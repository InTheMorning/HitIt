//! Moves Library files between `music_dir` and the audio cache (ADR 0100).

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::library_path::LibraryRelativePath;

#[derive(Clone, Debug)]
pub(crate) struct PreparedMove {
    relative_path: LibraryRelativePath,
    destination_root: PathBuf,
    source: PathBuf,
    destination: PathBuf,
}

impl PreparedMove {
    pub(crate) fn relative_path(&self) -> &LibraryRelativePath {
        &self.relative_path
    }

    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }
}

/// What `put_files` does when a file is already at the destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExistingDestination {
    /// Replace the file. Use this for the audio cache, where a lost file
    /// causes only a new download (ADR 0100 Decision 2).
    Replace,
    /// Give an error. Use this for `music_dir`.
    Fail,
}

/// Put each existing source file in the destination root.
pub(crate) fn put_files(
    source_root: &Path,
    destination_root: &Path,
    paths: &[LibraryRelativePath],
    existing: ExistingDestination,
) -> Result<Vec<PreparedMove>> {
    let mut moves = Vec::new();
    let result = (|| -> Result<()> {
        for relative_path in paths {
            let source = relative_path.resolve(source_root);
            if !source.is_file() {
                continue;
            }
            let destination = relative_path.resolve(destination_root);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create audio-cache folder {}", parent.display()))?;
            }
            if existing == ExistingDestination::Replace && destination.is_file() {
                fs::remove_file(&destination)
                    .with_context(|| format!("replace existing file {}", destination.display()))?;
            }
            if let Err(link_error) = fs::hard_link(&source, &destination) {
                if link_error.kind() == io::ErrorKind::AlreadyExists {
                    return Err(link_error).with_context(|| {
                        format!("file already exists: {}", destination.display())
                    });
                }
                copy_file(&source, &destination)?;
            }
            moves.push(PreparedMove {
                relative_path: relative_path.clone(),
                destination_root: destination_root.to_path_buf(),
                source,
                destination,
            });
        }
        Ok(())
    })();
    if let Err(error) = result {
        undo_put(&moves);
        return Err(error);
    }
    Ok(moves)
}

/// Copy one file without requiring a second file system.
pub(crate) fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    let source_len = fs::metadata(source)
        .with_context(|| format!("read source file {}", source.display()))?
        .len();
    let mut input =
        fs::File::open(source).with_context(|| format!("open source file {}", source.display()))?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .with_context(|| format!("create destination file {}", destination.display()))?;
    let copied = io::copy(&mut input, &mut output)
        .with_context(|| format!("copy {} to {}", source.display(), destination.display()));
    if let Err(error) = copied {
        let _ = fs::remove_file(destination);
        return Err(error);
    }
    output.flush()?;
    output.sync_all()?;
    let destination_len = output.metadata()?.len();
    if destination_len != source_len {
        let _ = fs::remove_file(destination);
        return Err(anyhow!("copied file length differs from its source"));
    }
    Ok(())
}

/// Undo destination files created by `put_files`.
pub(crate) fn undo_put(moves: &[PreparedMove]) {
    for moved in moves.iter().rev() {
        let _ = fs::remove_file(&moved.destination);
        cleanup_empty_parents(&moved.destination_root, &moved.destination);
    }
}

/// Remove source files after the caller commits its database transaction.
pub(crate) fn remove_sources(moves: &[PreparedMove], source_root: &Path) -> Result<()> {
    for moved in moves {
        match fs::remove_file(&moved.source) {
            Ok(()) => cleanup_empty_parents(source_root, &moved.source),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("remove {}", moved.source.display()))
            }
        }
    }
    Ok(())
}

/// Remove empty parent folders below `root`.
pub(crate) fn cleanup_empty_parents(root: &Path, path: &Path) {
    let mut directory = path.parent();
    while let Some(current) = directory {
        if current == root || !current.starts_with(root) {
            break;
        }
        if fs::read_dir(current)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
        {
            let _ = fs::remove_dir(current);
            directory = current.parent();
        } else {
            break;
        }
    }
}

/// Repair files left in `music_dir` by removals made before ADR 0100.
pub(crate) fn repair_start_step(
    conn: &rusqlite::Connection,
    music_dir: &Path,
    audio_cache_dir: &Path,
) -> Result<()> {
    if !music_dir.is_dir() {
        return Ok(());
    }
    let stale_track_ids = {
        let mut statement = conn.prepare(
            "SELECT DISTINCT lf.track_id FROM local_files lf
             JOIN tracks t ON t.id = lf.track_id WHERE t.is_in_library = 0 ORDER BY lf.track_id",
        )?;
        let rows = statement
            .query_map([], |row| row.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };
    for track_id in stale_track_ids {
        let rows = crate::db::local_audio_files_for_track(conn, track_id)?;
        let paths = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
        let moves = match put_files(
            music_dir,
            audio_cache_dir,
            &paths,
            ExistingDestination::Replace,
        ) {
            Ok(moves) => moves,
            Err(error) => {
                eprintln!("ADR 0100 start repair could not put track {track_id} in the audio cache: {error:#}");
                continue;
            }
        };
        let result = (|| -> Result<()> {
            let transaction = conn.unchecked_transaction()?;
            crate::db::move_local_file_rows_to_cache(&transaction, &rows, |row| {
                moves.iter().any(|moved| moved.relative_path() == &row.path)
            })?;
            transaction.commit()?;
            Ok(())
        })();
        if let Err(error) = result {
            undo_put(&moves);
            eprintln!("ADR 0100 start repair could not update track {track_id}: {error:#}");
            continue;
        }
        if let Err(error) = remove_sources(&moves, music_dir) {
            eprintln!("ADR 0100 start repair could not remove track {track_id} sources: {error:#}");
        }
    }

    for row in crate::db::cached_audio_files(conn)? {
        let cache_path = row.path.resolve(audio_cache_dir);
        let music_path = row.path.resolve(music_dir);
        let local_row_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM local_files WHERE path = ?1)",
            [row.path.as_stored()],
            |result| result.get(0),
        )?;
        if !local_row_exists
            && cache_path.is_file()
            && music_path.is_file()
            && fs::metadata(&cache_path)?.len() == fs::metadata(&music_path)?.len()
        {
            if let Err(error) = fs::remove_file(&music_path) {
                eprintln!(
                    "ADR 0100 start repair could not remove duplicate {}: {error}",
                    music_path.display()
                );
            } else {
                cleanup_empty_parents(music_dir, &music_path);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_0100_copy_file_copies_without_a_second_file_system() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let source = temp.path().join("source");
        let destination = temp.path().join("nested/destination");
        fs::create_dir_all(destination.parent().unwrap())?;
        fs::write(&source, b"audio bytes")?;

        copy_file(&source, &destination)?;

        assert_eq!(fs::read(destination)?, b"audio bytes");
        Ok(())
    }

    #[test]
    fn adr_0100_put_replaces_a_cache_file_and_keeps_a_music_file() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let source_root = temp.path().join("source");
        let destination_root = temp.path().join("destination");
        let relative = LibraryRelativePath::from_stored("a/track.mp3")?;
        for root in [&source_root, &destination_root] {
            fs::create_dir_all(relative.resolve(root).parent().unwrap())?;
        }
        fs::write(relative.resolve(&source_root), b"new")?;
        fs::write(relative.resolve(&destination_root), b"orphan")?;
        let paths = std::slice::from_ref(&relative);

        let failed = put_files(
            &source_root,
            &destination_root,
            paths,
            ExistingDestination::Fail,
        );
        assert!(failed.is_err(), "ADR 0100 keeps an existing music_dir file");
        assert_eq!(fs::read(relative.resolve(&destination_root))?, b"orphan");

        let moves = put_files(
            &source_root,
            &destination_root,
            paths,
            ExistingDestination::Replace,
        )?;
        assert_eq!(moves.len(), 1);
        assert_eq!(
            fs::read(relative.resolve(&destination_root))?,
            b"new",
            "ADR 0100 replaces an orphan cache file"
        );
        Ok(())
    }

    fn database() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::init_schema(&conn).unwrap();
        crate::db::migrate_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn adr_0100_start_moves_preexisting_removed_track() -> Result<()> {
        let conn = database();
        let temp = tempfile::tempdir()?;
        let music = temp.path().join("music");
        let cache = temp.path().join("audio");
        let relative = LibraryRelativePath::from_stored("artist/release/track.mp3")?;
        let source = relative.resolve(&music);
        fs::create_dir_all(source.parent().unwrap())?;
        fs::write(&source, b"old removal")?;
        conn.execute(
            "INSERT INTO feeds (id, feed_url) VALUES (1, 'https://example.test/adr-0100')",
            [],
        )?;
        conn.execute(
            "INSERT INTO tracks (id, feed_id, item_guid, is_in_library) VALUES (1, 1, 'old', 0)",
            [],
        )?;
        conn.execute(
            "INSERT INTO local_files (path, track_id, file_size_bytes) VALUES (?1, 1, 11)",
            [relative.as_stored()],
        )?;

        repair_start_step(&conn, &music, &cache)?;

        assert!(
            !source.exists(),
            "ADR 0100 moves the earlier removal out of music_dir"
        );
        assert_eq!(fs::read(relative.resolve(&cache))?, b"old removal");
        let local_count: i64 =
            conn.query_row("SELECT count(*) FROM local_files", [], |row| row.get(0))?;
        let cached_count: i64 =
            conn.query_row("SELECT count(*) FROM cached_files", [], |row| row.get(0))?;
        assert_eq!(local_count, 0);
        assert_eq!(cached_count, 1);
        Ok(())
    }

    #[test]
    fn adr_0100_start_removes_matching_duplicate_and_keeps_different_length() -> Result<()> {
        let conn = database();
        let temp = tempfile::tempdir()?;
        let music = temp.path().join("music");
        let cache = temp.path().join("audio");
        let same = LibraryRelativePath::from_stored("a/same.mp3")?;
        let different = LibraryRelativePath::from_stored("a/different.mp3")?;
        for (path, cache_bytes, music_bytes) in [
            (&same, b"equal".as_slice(), b"equal".as_slice()),
            (&different, b"cache".as_slice(), b"music differs".as_slice()),
        ] {
            for root in [&cache, &music] {
                fs::create_dir_all(path.resolve(root).parent().unwrap())?;
            }
            fs::write(path.resolve(&cache), cache_bytes)?;
            fs::write(path.resolve(&music), music_bytes)?;
            conn.execute(
                "INSERT INTO cached_files (path) VALUES (?1)",
                [path.as_stored()],
            )?;
        }

        repair_start_step(&conn, &music, &cache)?;

        assert!(
            !same.resolve(&music).exists(),
            "ADR 0100 removes a matching duplicate"
        );
        assert!(
            different.resolve(&music).exists(),
            "ADR 0100 keeps a different-length file"
        );
        Ok(())
    }

    #[test]
    fn adr_0100_start_does_nothing_when_music_path_is_not_a_directory() -> Result<()> {
        let conn = database();
        let temp = tempfile::tempdir()?;
        let music = temp.path().join("music-file");
        fs::write(&music, b"not a folder")?;

        repair_start_step(&conn, &music, &temp.path().join("audio"))?;

        assert_eq!(fs::read(music)?, b"not a folder");
        Ok(())
    }
}

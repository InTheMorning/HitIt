//! The one-time move of the v4vmm user data to the HitIt paths (ADR 0096
//! Decision 5). The app renames the directories and files. It does not make a
//! second set of the data. If a step is not successful, the app stops. Thus
//! the app does not start with an empty library next to the v4vmm library.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;

use super::{default_config_temporary, APP_ID};

const PREVIOUS_APP_ID: &str = "v4vmm";
const PREVIOUS_DATABASE: &str = "v4vmm.sqlite";
const DATABASE: &str = "hitit.sqlite";
// SQLite keeps these files beside the database. Each one moves with it.
const DATABASE_SIDECARS: [&str; 3] = ["-wal", "-shm", "-journal"];

struct UserDataPaths {
    previous_config_dir: PathBuf,
    config_dir: PathBuf,
    previous_data_dir: PathBuf,
    data_dir: PathBuf,
}

/// Move the v4vmm configuration and data directories to the HitIt paths.
///
/// # Errors
/// Reports the first failed move with its source and destination. Nothing
/// after that step runs.
pub fn move_previous_user_data() -> Result<()> {
    let (Some(previous), Some(current)) = (
        ProjectDirs::from("xyz", "HeyCitizen", PREVIOUS_APP_ID),
        ProjectDirs::from("xyz", "HeyCitizen", APP_ID),
    ) else {
        return Ok(());
    };
    move_user_data(&UserDataPaths {
        previous_config_dir: previous.config_dir().to_path_buf(),
        config_dir: current.config_dir().to_path_buf(),
        previous_data_dir: previous.data_dir().to_path_buf(),
        data_dir: current.data_dir().to_path_buf(),
    })
}

fn move_user_data(paths: &UserDataPaths) -> Result<()> {
    move_directory(&paths.previous_config_dir, &paths.config_dir)?;
    move_directory(&paths.previous_data_dir, &paths.data_dir)?;
    rename_database(&paths.data_dir)?;
    replace_default_db_path(
        &paths.config_dir.join("config.toml"),
        &paths.previous_data_dir.join(PREVIOUS_DATABASE),
        &paths.data_dir.join(DATABASE),
    )
}

fn exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn rename(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to).with_context(|| {
        format!(
            "HitIt could not move {} to {}. Move it by hand, then start HitIt again",
            from.display(),
            to.display()
        )
    })
}

fn move_directory(previous: &Path, current: &Path) -> Result<()> {
    // On macOS the config and data directories are one path, so the second
    // call finds the first move done.
    if exists(current) || !exists(previous) {
        return Ok(());
    }
    if let Some(parent) = current.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "HitIt could not create {} to move {} into it",
                parent.display(),
                previous.display()
            )
        })?;
    }
    rename(previous, current)
}

fn rename_database(data_dir: &Path) -> Result<()> {
    let previous = data_dir.join(PREVIOUS_DATABASE);
    let current = data_dir.join(DATABASE);
    if exists(&current) || !exists(&previous) {
        return Ok(());
    }
    // The main file moves last. After a failure, the next start finds the
    // main file at its v4vmm name and continues with the remaining files.
    for suffix in DATABASE_SIDECARS {
        let sidecar = data_dir.join(format!("{PREVIOUS_DATABASE}{suffix}"));
        if exists(&sidecar) {
            rename(&sidecar, &data_dir.join(format!("{DATABASE}{suffix}")))?;
        }
    }
    rename(&previous, &current)
}

fn replace_default_db_path(cfg_path: &Path, previous: &Path, current: &Path) -> Result<()> {
    let text = match fs::read_to_string(cfg_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("HitIt could not read {}", cfg_path.display()))
        }
    };
    // A file that does not parse goes to the ADR 0066 recovery unchanged.
    let Ok(table) = text.parse::<toml::Table>() else {
        return Ok(());
    };
    let previous_text = previous.display().to_string();
    if table.get("db_path").and_then(toml::Value::as_str) != Some(previous_text.as_str()) {
        return Ok(());
    }
    let Some(updated) = replace_top_level_line(&text, "db_path", current) else {
        return Ok(());
    };
    write_config(cfg_path, &updated)
}

/// Replace the one top-level `key = ...` line. Return `None` when the file
/// has no such line before its first table header.
fn replace_top_level_line(text: &str, key: &str, value: &Path) -> Option<String> {
    let replacement = format!(
        "{key} = {}",
        toml::Value::String(value.display().to_string())
    );
    let mut found = false;
    let mut lines = Vec::new();
    let mut top_level = true;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            top_level = false;
        }
        let is_key = top_level
            && !found
            && trimmed
                .strip_prefix(key)
                .is_some_and(|rest| rest.trim_start().starts_with('='));
        if is_key {
            found = true;
            let ending = &line[line.trim_end_matches(['\r', '\n']).len()..];
            lines.push(format!("{replacement}{ending}"));
        } else {
            lines.push(line.to_owned());
        }
    }
    found.then(|| lines.concat())
}

fn write_config(cfg_path: &Path, text: &str) -> Result<()> {
    let (temporary, mut file) = default_config_temporary(cfg_path)?;
    let result = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .with_context(|| format!("HitIt could not write {}", temporary.display()))
        .and_then(|()| rename(&temporary, cfg_path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(root: &Path) -> UserDataPaths {
        UserDataPaths {
            previous_config_dir: root.join("config/v4vmm"),
            config_dir: root.join("config/hitit"),
            previous_data_dir: root.join("data/v4vmm"),
            data_dir: root.join("data/hitit"),
        }
    }

    fn write_previous(root: &Path, db_path: &Path) {
        let paths = paths(root);
        fs::create_dir_all(&paths.previous_config_dir).unwrap();
        fs::create_dir_all(&paths.previous_data_dir).unwrap();
        fs::write(
            paths.previous_config_dir.join("config.toml"),
            format!(
                "# config\nmusic_dir = \"/music\"\ndb_path = \"{}\"\n\n[playback]\ndriver = \"mpv\"\n",
                db_path.display()
            ),
        )
        .unwrap();
        fs::write(paths.previous_data_dir.join("v4vmm.sqlite"), b"main").unwrap();
        fs::write(paths.previous_data_dir.join("v4vmm.sqlite-wal"), b"wal").unwrap();
    }

    #[test]
    fn adr_0096_moves_directories_database_and_default_db_path() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(temp.path());
        write_previous(temp.path(), &paths.previous_data_dir.join("v4vmm.sqlite"));

        move_user_data(&paths).unwrap();

        assert!(!paths.previous_config_dir.exists());
        assert!(!paths.previous_data_dir.exists());
        assert_eq!(
            fs::read(paths.data_dir.join("hitit.sqlite")).unwrap(),
            b"main"
        );
        assert_eq!(
            fs::read(paths.data_dir.join("hitit.sqlite-wal")).unwrap(),
            b"wal"
        );
        let config = fs::read_to_string(paths.config_dir.join("config.toml")).unwrap();
        assert_eq!(
            config,
            format!(
                "# config\nmusic_dir = \"/music\"\ndb_path = \"{}\"\n\n[playback]\ndriver = \"mpv\"\n",
                paths.data_dir.join("hitit.sqlite").display()
            )
        );
    }

    #[test]
    fn adr_0096_keeps_a_db_path_that_the_operator_chose() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(temp.path());
        write_previous(temp.path(), Path::new("/srv/library.sqlite"));

        move_user_data(&paths).unwrap();

        let config = fs::read_to_string(paths.config_dir.join("config.toml")).unwrap();
        assert!(config.contains("db_path = \"/srv/library.sqlite\"\n"));
    }

    #[test]
    fn adr_0096_changes_nothing_when_the_hitit_directories_exist() {
        let temp = tempfile::tempdir().unwrap();
        let paths = paths(temp.path());
        write_previous(temp.path(), &paths.previous_data_dir.join("v4vmm.sqlite"));
        fs::create_dir_all(&paths.config_dir).unwrap();
        fs::create_dir_all(&paths.data_dir).unwrap();

        move_user_data(&paths).unwrap();

        assert!(paths.previous_config_dir.join("config.toml").exists());
        assert!(paths.previous_data_dir.join("v4vmm.sqlite").exists());
        assert!(!paths.config_dir.join("config.toml").exists());
        assert!(!paths.data_dir.join("hitit.sqlite").exists());
    }

    #[test]
    fn adr_0096_keeps_a_hitit_database_that_exists() {
        let temp = tempfile::tempdir().unwrap();
        let data_dir = temp.path();
        fs::write(data_dir.join("v4vmm.sqlite"), b"previous").unwrap();
        fs::write(data_dir.join("hitit.sqlite"), b"current").unwrap();

        rename_database(data_dir).unwrap();

        assert_eq!(fs::read(data_dir.join("hitit.sqlite")).unwrap(), b"current");
        assert_eq!(
            fs::read(data_dir.join("v4vmm.sqlite")).unwrap(),
            b"previous"
        );
    }

    #[test]
    fn adr_0096_replaces_only_the_top_level_db_path_line() {
        let text = "db_path = \"/a\"\r\n[table]\ndb_path = \"/a\"\n";
        let updated = replace_top_level_line(text, "db_path", Path::new("/b")).unwrap();
        assert_eq!(updated, "db_path = \"/b\"\r\n[table]\ndb_path = \"/a\"\n");
        assert!(replace_top_level_line("[t]\ndb_path = 1\n", "db_path", Path::new("/b")).is_none());
    }

    #[test]
    fn adr_0096_names_both_paths_when_a_move_fails() {
        let temp = tempfile::tempdir().unwrap();
        let previous = temp.path().join("previous");
        fs::create_dir(&previous).unwrap();
        // A file in the parent position makes the directory creation fail.
        fs::write(temp.path().join("blocked"), b"").unwrap();
        let current = temp.path().join("blocked/current");

        let error = move_directory(&previous, &current).unwrap_err();

        let message = format!("{error:#}");
        assert!(message.contains("blocked") && message.contains("previous"));
        assert!(previous.exists());
    }
}

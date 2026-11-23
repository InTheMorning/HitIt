//! The retag of the library to the frame names of musicindex ADR 0093
//! R-TAG-2.
//!
//! The command reads each library file. A file that has an old
//! `MusicIndex ...` frame gets the value of that frame under its new name.
//! The crate write removes each old frame (R-TAG-2 item 5). A file with no old
//! frame does not change, so a second run writes nothing.
//!
//! A file also needs a write when its ID3 tag is not a correct ID3v2.4 tag:
//! a bad frame, a tag of an older version or an ID3v1 tag (musicindex ADR
//! 0090 R-TAG-8 and R-TAG-10 item 2). The crate write repairs it. A file with
//! a bad tag header stays unreadable (R-TAG-7 item 5).
//!
//! The dry run reads only. The write copies each file into the backup folder
//! before its write. The route frame uses the stored route of the track (ADR
//! 0076 Decision 9).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use musicindex_tags::format::AudioFormat;
use musicindex_tags::read_id3v2;
use rusqlite::Connection;

use crate::audio_tags::{
    frames, read_audio_tags, BadTag, Id3Field, Id3v24Edit, ReferenceSource, TagWriteResult,
    RENAMED_FRAMES,
};
use crate::db::{self, TrackRow};
use crate::library_path::LibraryRelativePath;
use crate::metadata_service::{with_stored_route_frame, RouteFrameWrite};

/// The start of the value of a transcript edit. The value after it is the
/// index of the transcript text in [`FilePlan::transcripts`].
const TRANSCRIPT_REFERENCE: &str = "retag-transcript:";

/// The state of one library file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FileState {
    /// The file has old frames or an ID3 tag to repair. The write gives the
    /// old frames their new names and repairs the tag.
    Rewrite,
    /// The file has no old frame and no ID3 tag to repair.
    Current,
    /// The track has no local path, or the file is not on the disk.
    Missing,
    /// The tags of the file cannot be read.
    Unreadable(String),
}

/// What happens to the route frame of one file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RouteChange {
    /// The file has no old route frame.
    NoFrame,
    /// The stored route gives the same value.
    Same,
    /// The stored route gives a different value.
    Stored,
    /// The track has no stored route. The write removes the old frame and
    /// writes no new frame.
    Removed,
}

/// The plan of one library file.
#[derive(Clone, Debug)]
pub(crate) struct FilePlan {
    pub(crate) track_id: i64,
    /// The stored path of the file in the library (ADR 0064).
    pub(crate) relative: Option<LibraryRelativePath>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) state: FileState,
    /// The label of each old frame in the file.
    pub(crate) old_frames: Vec<String>,
    /// Each repair of the ID3 tag that the write makes (musicindex ADR 0093
    /// R-TAG-8), for example `ID3v2.3 to ID3v2.4`.
    pub(crate) repairs: Vec<String>,
    /// The edits, before the route frame takes the stored route.
    edits: Vec<Id3v24Edit>,
    /// The text of each transcript edit.
    transcripts: Vec<String>,
}

impl FilePlan {
    /// The label of each new frame that the plan writes, before the route
    /// frame takes the stored route.
    pub(crate) fn new_frames(&self) -> Vec<&str> {
        self.edits
            .iter()
            .map(|edit| edit.frame_label.as_str())
            .collect()
    }
}

/// The dry run of the whole library.
#[derive(Clone, Debug, Default)]
pub(crate) struct RetagPlan {
    pub(crate) files: Vec<(FilePlan, RouteChange)>,
}

impl RetagPlan {
    fn count(&self, state: &FileState) -> usize {
        self.files
            .iter()
            .filter(|(file, _)| match (state, &file.state) {
                (FileState::Unreadable(_), FileState::Unreadable(_)) => true,
                (expected, actual) => expected == actual,
            })
            .count()
    }

    /// The number of old frames of each label in the files to rewrite.
    pub(crate) fn old_frame_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for (file, _) in &self.files {
            for label in &file.old_frames {
                *counts.entry(label.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    fn route_count(&self, change: RouteChange) -> usize {
        self.files
            .iter()
            .filter(|(file, route)| file.state == FileState::Rewrite && *route == change)
            .count()
    }
}

/// The result of the write of one file.
#[derive(Clone, Debug)]
pub(crate) enum FileResult {
    Written(TagWriteResult),
    Failed(String),
}

/// The result of the write of the library.
#[derive(Clone, Debug, Default)]
pub(crate) struct RetagReport {
    pub(crate) plan: RetagPlan,
    /// The stored path of each written file and its result.
    pub(crate) results: Vec<(String, FileResult)>,
}

/// Reads each library file and makes its plan. The function writes nothing.
///
/// # Errors
///
/// Returns an error when the library or a stored route cannot be read.
pub(crate) fn plan_library(conn: &Connection, music_dir: &Path) -> Result<RetagPlan> {
    let mut plan = RetagPlan::default();
    for track in db::library_tracks(conn)? {
        let file = plan_track(&track, music_dir);
        let route = route_change(conn, &file)?;
        plan.files.push((file, route));
    }
    Ok(plan)
}

/// Rewrites each library file that has old frames. Each file goes into
/// `backup_dir` before its write. An existing backup does not change, so it
/// keeps the file of the first run.
///
/// # Errors
///
/// Returns an error when `backup_dir` is in `music_dir`, or when the library
/// cannot be read. The failure of one file goes into the report.
pub(crate) fn write_library(
    conn: &Connection,
    music_dir: &Path,
    backup_dir: &Path,
) -> Result<RetagReport> {
    if !backup_dir.is_absolute() {
        return Err(anyhow!(
            "the backup folder must be an absolute path: {}",
            backup_dir.display()
        ));
    }
    if backup_dir.starts_with(music_dir) {
        return Err(anyhow!(
            "the backup folder {} is in the music folder {}; the library scan would read the copies",
            backup_dir.display(),
            music_dir.display()
        ));
    }
    let plan = plan_library(conn, music_dir)?;
    let mut results = Vec::new();
    for (file, _) in &plan.files {
        if file.state != FileState::Rewrite {
            continue;
        }
        let (Some(relative), Some(path)) = (&file.relative, &file.path) else {
            continue;
        };
        let backup = relative.resolve(backup_dir);
        let result = match retag_file(conn, file, path, &backup) {
            Ok(write) => FileResult::Written(write),
            Err(error) => FileResult::Failed(format!("{error:#}")),
        };
        results.push((relative.as_stored().to_owned(), result));
    }
    Ok(RetagReport { plan, results })
}

/// Writes the new frames of one file. The route frame takes the stored route
/// before the write (ADR 0076 Decision 9).
fn retag_file(
    conn: &Connection,
    file: &FilePlan,
    path: &Path,
    backup: &Path,
) -> Result<TagWriteResult> {
    let edits = with_stored_route_frame(
        conn,
        file.track_id,
        None,
        file.edits.clone(),
        RouteFrameWrite::WhenSelected,
    )?;
    // An empty edit list still removes the old frames: a file whose only old
    // frame gets no new frame, for example a route frame of a track with no
    // stored route.
    if !backup.exists() {
        if let Some(parent) = backup.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("make backup folder {}", parent.display()))?;
        }
        fs::copy(path, backup).with_context(|| format!("make backup {}", backup.display()))?;
    }
    let source = RetagSource {
        transcripts: &file.transcripts,
    };
    musicindex_tags::write_id3v24_edits(path, &edits, &source)
}

fn route_change(conn: &Connection, file: &FilePlan) -> Result<RouteChange> {
    let Some(old) = file
        .edits
        .iter()
        .find(|edit| edit.frame_label == frames::VALUE_ROUTES)
    else {
        return Ok(RouteChange::NoFrame);
    };
    let stored = db::payment_routes::stored_route(conn, file.track_id)?;
    let Some(value) = stored
        .as_deref()
        .and_then(crate::metadata::summarize_value_routes)
    else {
        return Ok(RouteChange::Removed);
    };
    Ok(if value == old.value {
        RouteChange::Same
    } else {
        RouteChange::Stored
    })
}

fn plan_track(track: &TrackRow, music_dir: &Path) -> FilePlan {
    let relative = track.local_path.clone();
    let path = relative
        .as_ref()
        .map(|relative| relative.resolve(music_dir));
    let mut plan = FilePlan {
        track_id: track.id,
        relative,
        path: path.clone(),
        state: FileState::Missing,
        old_frames: Vec::new(),
        repairs: Vec::new(),
        edits: Vec::new(),
        transcripts: Vec::new(),
    };
    let Some(path) = path.filter(|path| path.is_file()) else {
        return plan;
    };
    match id3_repairs(&path) {
        Ok(repairs) => plan.repairs = repairs,
        Err(error) => {
            plan.state = FileState::Unreadable(format!("{error:#}"));
            return plan;
        }
    }
    match read_audio_tags(&path) {
        Ok(tags) => plan_fields(&mut plan, &tags.fields),
        Err(error) => plan.state = FileState::Unreadable(format!("{error:#}")),
    }
    plan
}

/// Each repair that a write makes to the ID3 tag of an MP3 file (musicindex
/// ADR 0093 R-TAG-8). Another format gives no repair.
///
/// # Errors
///
/// Returns an error when the file cannot be read, or when its ID3v2 tag
/// header is bad. The write does not change such a file (R-TAG-7 item 5).
fn id3_repairs(path: &Path) -> Result<Vec<String>> {
    if !matches!(AudioFormat::detect_from_file(path), Ok(AudioFormat::Mp3)) {
        return Ok(Vec::new());
    }
    let read = read_id3v2(path)?;
    if let Some(cause) = read.problems.bad_tag {
        return Err(BadTag { cause }.into());
    }
    let mut repairs = Vec::new();
    if let Some(major @ (2 | 3)) = read.version {
        repairs.push(format!("ID3v2.{major} to ID3v2.4"));
    }
    for frame in &read.problems.bad_frames {
        repairs.push(format!(
            "bad frame {} at byte {}: {}",
            frame.id.as_deref().unwrap_or("(no ID)"),
            frame.position,
            frame.cause
        ));
    }
    if read.plain_frame_sizes {
        repairs.push("ID3v2.4 frame sizes that are not syncsafe".to_owned());
    }
    if read.has_id3v1 {
        repairs.push("remove the ID3v1 tag".to_owned());
    }
    Ok(repairs)
}

/// Fills the plan from the fields of one file.
fn plan_fields(plan: &mut FilePlan, fields: &[Id3Field]) {
    let old = fields
        .iter()
        .filter(|field| frames::is_old_description(frames::descriptor(&field.frame_id)))
        .collect::<Vec<_>>();
    plan.state = if old.is_empty() && plan.repairs.is_empty() {
        FileState::Current
    } else {
        FileState::Rewrite
    };
    for field in old {
        plan.old_frames.push(field.frame_id.clone());
        let Some(new_label) = new_label(&field.frame_id) else {
            continue;
        };
        if plan.edits.iter().any(|edit| edit.frame_label == new_label) {
            continue;
        }
        // A value under the new name is newer than the old value.
        let value = fields
            .iter()
            .find(|existing| existing.frame_id == new_label)
            .map_or(field.value.as_str(), |existing| existing.value.as_str());
        if value.trim().is_empty() {
            continue;
        }
        let value = if new_label == frames::TRANSCRIPT_TEXT || new_label == frames::TRANSCRIPT_TIMED
        {
            let text = if new_label == frames::TRANSCRIPT_TIMED {
                timed_table_to_srt(value).unwrap_or_else(|| value.to_owned())
            } else {
                value.to_owned()
            };
            plan.transcripts.push(text);
            format!("{TRANSCRIPT_REFERENCE}{}", plan.transcripts.len() - 1)
        } else {
            value.to_owned()
        };
        plan.edits.push(Id3v24Edit {
            frame_label: new_label.to_owned(),
            value,
        });
    }
}

/// The new label of an old field label. An MP3 file gives the frame ID and
/// the descriptor, for example `SYLT:MusicIndex Transcript`. A Vorbis comment
/// or an MP4 atom gives a `TXXX` label with the stored key, for example
/// `TXXX:MUSICINDEX FEED GUID`. Thus the descriptor decides, with any letter
/// case, and the frame ID decides only between the two transcript frames.
fn new_label(old_label: &str) -> Option<&'static str> {
    let (frame_id, descriptor) = old_label.split_once(':')?;
    let same_descriptor = |old: &str| frames::descriptor(old).eq_ignore_ascii_case(descriptor);
    RENAMED_FRAMES
        .iter()
        .find(|(old, _)| same_descriptor(old) && old.starts_with(&format!("{frame_id}:")))
        .or_else(|| RENAMED_FRAMES.iter().find(|(old, _)| same_descriptor(old)))
        .map(|(_, new)| *new)
}

/// Converts the display value of a `SYLT` frame into SRT text. The display
/// value is the table of the `id3` crate: a header line, then one line with
/// `HH:MM:SS.mmm`, a tab and the text for each time. The crate parses the SRT
/// text back into the same times. The function gives `None` when the value has
/// no timed line.
fn timed_table_to_srt(table: &str) -> Option<String> {
    let mut srt = String::new();
    let mut count = 0_usize;
    for line in table.lines() {
        let Some((time, text)) = line.split_once('\t') else {
            continue;
        };
        let time = time.trim();
        if !time.contains(':') || !time.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        count += 1;
        let _ = write!(srt, "{count}\n{time} --> {time}\n{}\n\n", text.trim());
    }
    (count > 0).then_some(srt)
}

/// The reference source of the retag. A transcript edit names the text that
/// the plan read from the file. The retag writes no picture.
struct RetagSource<'a> {
    transcripts: &'a [String],
}

impl ReferenceSource for RetagSource<'_> {
    fn text(&self, reference: &str) -> Result<String> {
        reference
            .strip_prefix(TRANSCRIPT_REFERENCE)
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| self.transcripts.get(index))
            .cloned()
            .ok_or_else(|| anyhow!("unknown transcript reference {reference}"))
    }

    fn picture(&self, reference: &str) -> Result<(String, Vec<u8>)> {
        Err(anyhow!("the retag writes no picture: {reference}"))
    }
}

/// The text of the dry run: each file to rewrite, then the counts.
pub(crate) fn plan_text(plan: &RetagPlan) -> String {
    let mut text = String::new();
    for (file, route) in &plan.files {
        if file.state != FileState::Rewrite {
            continue;
        }
        let _ = writeln!(text, "{}", shown_path(file.relative.as_ref()));
        if !file.old_frames.is_empty() {
            let _ = writeln!(
                text,
                "  old: {}\n  new: {}",
                file.old_frames.join(", "),
                file.new_frames().join(", ")
            );
        }
        for repair in &file.repairs {
            let _ = writeln!(text, "  repair: {repair}");
        }
        text.push_str(match route {
            RouteChange::NoFrame | RouteChange::Same => "",
            RouteChange::Stored => "  route: the stored route replaces the file value\n",
            RouteChange::Removed => "  route: no stored route; the write removes the route frame\n",
        });
    }
    for (file, _) in &plan.files {
        if let FileState::Unreadable(error) = &file.state {
            let _ = writeln!(
                text,
                "unreadable: {}: {error}",
                shown_path(file.relative.as_ref())
            );
        }
    }
    let _ = writeln!(text, "\nLibrary tracks: {}", plan.files.len());
    let _ = writeln!(
        text,
        "Files to rewrite: {}",
        plan.count(&FileState::Rewrite)
    );
    let _ = writeln!(
        text,
        "Files with a tag repair: {}",
        plan.files
            .iter()
            .filter(|(file, _)| file.state == FileState::Rewrite && !file.repairs.is_empty())
            .count()
    );
    let _ = writeln!(
        text,
        "Files with a correct tag: {}",
        plan.count(&FileState::Current)
    );
    let _ = writeln!(
        text,
        "Tracks with no file: {}",
        plan.count(&FileState::Missing)
    );
    let _ = writeln!(
        text,
        "Unreadable files: {}",
        plan.count(&FileState::Unreadable(String::new()))
    );
    let _ = writeln!(text, "Old frames:");
    for (label, count) in plan.old_frame_counts() {
        let new = new_label(&label).unwrap_or("(no new name; the write removes it)");
        let _ = writeln!(text, "  {count:>6}  {label} -> {new}");
    }
    let _ = writeln!(
        text,
        "Route frames: {} same, {} from the stored route, {} removed (no stored route)",
        plan.route_count(RouteChange::Same),
        plan.route_count(RouteChange::Stored),
        plan.route_count(RouteChange::Removed)
    );
    text
}

/// The text of the write: each failed file and each skipped edit, then the
/// counts.
pub(crate) fn report_text(report: &RetagReport) -> String {
    let mut text = String::new();
    let mut written = 0_usize;
    let mut removed = 0_usize;
    let mut failed = 0_usize;
    let mut bad_frames = 0_usize;
    let mut id3v1 = 0_usize;
    for (shown, result) in &report.results {
        match result {
            FileResult::Written(write) => {
                written += 1;
                removed += write.removed_old_frames.len();
                bad_frames += write.bad_frames.len();
                id3v1 += usize::from(write.removed_id3v1);
                if !write.skipped.is_empty() {
                    let _ = writeln!(text, "{shown}: skipped: {}", write.skipped.join("; "));
                }
            }
            FileResult::Failed(error) => {
                failed += 1;
                let _ = writeln!(text, "{shown}: failed: {error}");
            }
        }
    }
    let _ = writeln!(
        text,
        "\nFiles with a correct tag before the write: {}",
        report.plan.count(&FileState::Current)
    );
    let _ = writeln!(text, "Files written: {written}");
    let _ = writeln!(text, "Files failed: {failed}");
    let _ = writeln!(text, "Old frames removed: {removed}");
    let _ = writeln!(text, "Bad frames removed or repaired: {bad_frames}");
    let _ = writeln!(text, "ID3v1 tags removed: {id3v1}");
    text
}

fn shown_path(relative: Option<&LibraryRelativePath>) -> &str {
    relative.map_or("(no local path)", LibraryRelativePath::as_stored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_tags::read_audio_tags;
    use crate::library_path::LibraryRelativePath;
    use id3::frame::{
        Comment, ExtendedText, SynchronisedLyrics, SynchronisedLyricsType, TimestampFormat,
    };
    use id3::{Tag, TagLike, Version};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::init_schema(&conn).unwrap();
        db::migrate_schema(&conn).unwrap();
        conn.execute(
            "INSERT INTO feeds (feed_url, feed_guid, title) VALUES ('https://example.test/feed.xml', 'feed-guid', 'Feed')",
            [],
        )
        .unwrap();
        conn
    }

    fn add_track(conn: &Connection, music_dir: &Path, file_name: &str, bytes: &[u8]) -> PathBuf {
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, track_title, is_in_library)
             VALUES (1, ?1, 'Song', 1)",
            [file_name],
        )
        .unwrap();
        let track_id = conn.last_insert_rowid();
        let path = music_dir.join(file_name);
        fs::write(&path, bytes).unwrap();
        let relative = LibraryRelativePath::from_absolute(music_dir, &path).unwrap();
        db::mark_track_downloaded(conn, track_id, &relative, None).unwrap();
        path
    }

    fn set_stored_route(conn: &Connection, json: &str) {
        conn.execute("UPDATE tracks SET payment_routes_json = ?1", [json])
            .unwrap();
    }

    const ROUTES: &str =
        r#"[{"recipient_name":"Artist","route_type":"node","address":"03abc","split":100.0}]"#;

    fn old_mp3(path: &Path) {
        let mut tag = Tag::new();
        tag.set_title("Song");
        for (description, value) in [
            ("MusicIndex Feed Guid", "feed-guid"),
            ("MusicIndex Track Guid", "item-guid"),
            ("MusicIndex Value Routes", "old routes"),
            ("MusicIndex Image", "https://example.test/cover.jpg"),
        ] {
            tag.add_frame(ExtendedText {
                description: description.into(),
                value: value.into(),
            });
        }
        tag.add_frame(Comment {
            lang: "eng".into(),
            description: "MusicIndex Album Description".into(),
            text: "The album.".into(),
        });
        tag.add_frame(SynchronisedLyrics {
            lang: "eng".into(),
            timestamp_format: TimestampFormat::Ms,
            content_type: SynchronisedLyricsType::Transcription,
            description: "MusicIndex Transcript".into(),
            content: vec![(1_500, "First line".into()), (62_250, "Second line".into())],
        });
        tag.write_to_path(path, Version::Id3v24).unwrap();
    }

    fn frame_labels(path: &Path) -> Vec<String> {
        read_audio_tags(path)
            .unwrap()
            .fields
            .into_iter()
            .map(|field| field.frame_id)
            .collect()
    }

    /// R-TAG-2 item 5 and the step 3 request: the write gives each old frame
    /// its new name, removes the old frames and keeps a backup.
    #[test]
    fn adr_0086_retag_renames_the_old_frames_and_keeps_a_backup() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let path = add_track(&conn, music.path(), "song.mp3", b"");
        old_mp3(&path);
        set_stored_route(&conn, ROUTES);
        let original = fs::read(&path).unwrap();

        let report = write_library(&conn, music.path(), backups.path()).unwrap();

        assert!(matches!(
            report.results.as_slice(),
            [(_, FileResult::Written(write))] if write.removed_old_frames.len() == 6 && write.skipped.is_empty()
        ));
        let labels = frame_labels(&path);
        for label in [
            frames::FEED_GUID,
            frames::ITEM_GUID,
            frames::VALUE_ROUTES,
            frames::IMAGE,
            frames::FEED_DESCRIPTION,
            frames::TRANSCRIPT_TIMED,
        ] {
            assert!(
                labels.iter().any(|existing| existing == label),
                "{label}: {labels:?}"
            );
        }
        assert!(
            !labels.iter().any(|label| label.contains("MusicIndex")),
            "{labels:?}"
        );
        let tags = read_audio_tags(&path).unwrap();
        let value = |label: &str| {
            tags.fields
                .iter()
                .find(|field| field.frame_id == label)
                .map(|field| field.value.clone())
        };
        assert_eq!(value(frames::FEED_GUID).as_deref(), Some("feed-guid"));
        assert_eq!(value(frames::ITEM_GUID).as_deref(), Some("item-guid"));
        assert_ne!(value(frames::VALUE_ROUTES).as_deref(), Some("old routes"));
        assert_eq!(fs::read(backups.path().join("song.mp3")).unwrap(), original);
    }

    /// The `SYLT` times survive the rewrite, to the millisecond.
    #[test]
    fn adr_0086_retag_keeps_the_transcript_times() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let path = add_track(&conn, music.path(), "song.mp3", b"");
        old_mp3(&path);

        write_library(&conn, music.path(), backups.path()).unwrap();

        let tag = Tag::read_from_path(&path).unwrap();
        let lyrics = tag.synchronised_lyrics().collect::<Vec<_>>();
        assert_eq!(lyrics.len(), 1);
        assert_eq!(lyrics[0].description, "Podcast Transcript");
        assert_eq!(
            lyrics[0].content,
            vec![
                (1_500, "First line".to_owned()),
                (62_250, "Second line".to_owned())
            ]
        );
    }

    /// An ID3v2.3 tag with `frames`, then audio bytes. The frame sizes of
    /// ID3v2.3 are plain numbers.
    fn v23_file(frames: &[(&[u8; 4], &[u8])], id3v1: bool) -> Vec<u8> {
        let mut body = Vec::new();
        for (id, data) in frames {
            body.extend_from_slice(*id);
            body.extend_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
            body.extend_from_slice(&[0, 0]);
            body.extend_from_slice(data);
        }
        body.extend_from_slice(&[0; 16]);
        let size = u32::try_from(body.len()).unwrap();
        let mut bytes = b"ID3\x03\x00\x00".to_vec();
        bytes.extend_from_slice(&[
            u8::try_from((size >> 21) & 0x7f).unwrap(),
            u8::try_from((size >> 14) & 0x7f).unwrap(),
            u8::try_from((size >> 7) & 0x7f).unwrap(),
            u8::try_from(size & 0x7f).unwrap(),
        ]);
        bytes.extend_from_slice(&body);
        bytes.extend_from_slice(b"\xff\xfbmp3data");
        if id3v1 {
            let mut v1 = b"TAG".to_vec();
            v1.extend_from_slice(b"Old Title");
            v1.resize(128, 0);
            bytes.extend_from_slice(&v1);
        }
        bytes
    }

    /// Musicindex ADR 0093 R-TAG-10 item 2: the two Strange Love files have an
    /// ID3v2.3 tag with two `PRIV` frames of size 0. The retag rewrites them,
    /// and then a strict `id3` read gives an ID3v2.4 tag with no `PRIV` frame.
    #[test]
    fn adr_0090_retag_rewrites_a_v23_file_with_empty_priv_frames() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let bytes = v23_file(
            &[
                (b"TIT2", b"\x00Dirt and Sand"),
                (b"PRIV", b""),
                (b"PRIV", b""),
                (b"TPE1", b"\x00Strange Love"),
            ],
            false,
        );
        let path = add_track(&conn, music.path(), "song.mp3", &bytes);
        assert!(Tag::read_from_path(&path).is_err(), "the fixture is bad");

        let plan = plan_library(&conn, music.path()).unwrap();
        assert_eq!(plan.count(&FileState::Rewrite), 1);
        assert!(plan_text(&plan).contains("bad frame PRIV"));
        let report = write_library(&conn, music.path(), backups.path()).unwrap();
        let FileResult::Written(write) = &report.results[0].1 else {
            panic!("the write failed: {:?}", report.results);
        };
        assert_eq!(write.bad_frames.len(), 2);

        let tag = Tag::read_from_path(&path).expect("a strict read accepts the tag");
        assert_eq!(tag.version(), Version::Id3v24);
        assert_eq!(tag.title(), Some("Dirt and Sand"));
        assert_eq!(tag.artist(), Some("Strange Love"));
        assert!(tag.get("PRIV").is_none());
        assert_eq!(
            plan_library(&conn, music.path())
                .unwrap()
                .count(&FileState::Current),
            1
        );
    }

    /// Musicindex ADR 0093 R-TAG-8: a correct ID3v2.3 file with no podcast
    /// frame and with an ID3v1 tag is a rewrite. The write gives ID3v2.4 and
    /// removes the ID3v1 tag.
    #[test]
    fn adr_0090_retag_converts_a_v23_file_with_no_podcast_frame() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let bytes = v23_file(&[(b"TIT2", b"\x00Song")], true);
        let path = add_track(&conn, music.path(), "song.mp3", &bytes);

        let plan = plan_library(&conn, music.path()).unwrap();
        assert_eq!(plan.count(&FileState::Rewrite), 1);
        assert_eq!(
            plan.files[0].0.repairs,
            vec![
                "ID3v2.3 to ID3v2.4".to_owned(),
                "remove the ID3v1 tag".to_owned()
            ]
        );
        write_library(&conn, music.path(), backups.path()).unwrap();

        let tag = Tag::read_from_path(&path).unwrap();
        assert_eq!(tag.version(), Version::Id3v24);
        assert_eq!(tag.title(), Some("Song"));
        let written = fs::read(&path).unwrap();
        assert!(
            !written.windows(12).any(|bytes| bytes == b"TAGOld Title"),
            "the write must remove the ID3v1 tag"
        );
        assert_eq!(
            plan_library(&conn, music.path())
                .unwrap()
                .count(&FileState::Current),
            1
        );
    }

    /// Musicindex ADR 0093 R-TAG-7 item 5: a file with a bad tag header stays
    /// unreadable, and the write does not change it.
    #[test]
    fn adr_0090_retag_reports_a_bad_tag_header_and_keeps_the_file() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        // The size byte 0x80 is not syncsafe.
        let bytes = b"ID3\x03\x00\x00\x00\x00\x00\x80TIT2\xff\xfbmp3data".to_vec();
        let path = add_track(&conn, music.path(), "song.mp3", &bytes);

        let plan = plan_library(&conn, music.path()).unwrap();
        assert_eq!(plan.count(&FileState::Unreadable(String::new())), 1);
        assert!(plan_text(&plan).contains("unreadable: song.mp3"));
        let report = write_library(&conn, music.path(), backups.path()).unwrap();
        assert!(report.results.is_empty());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    /// A second run writes nothing: the file has only new frames.
    #[test]
    fn adr_0086_retag_second_run_changes_nothing() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let path = add_track(&conn, music.path(), "song.mp3", b"");
        old_mp3(&path);
        set_stored_route(&conn, ROUTES);
        write_library(&conn, music.path(), backups.path()).unwrap();
        let after_first = fs::read(&path).unwrap();

        let second = write_library(&conn, music.path(), backups.path()).unwrap();

        assert!(second.results.is_empty());
        assert_eq!(second.plan.count(&FileState::Current), 1);
        assert_eq!(fs::read(&path).unwrap(), after_first);
    }

    /// The dry run reads only, and it counts each old frame and the route
    /// frame that has no stored route.
    #[test]
    fn adr_0086_retag_dry_run_writes_nothing() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let path = add_track(&conn, music.path(), "song.mp3", b"");
        old_mp3(&path);
        let before = fs::read(&path).unwrap();

        let plan = plan_library(&conn, music.path()).unwrap();

        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(plan.count(&FileState::Rewrite), 1);
        assert_eq!(plan.old_frame_counts().len(), 6);
        assert_eq!(plan.route_count(RouteChange::Removed), 1);
        let text = plan_text(&plan);
        assert!(text.contains("Files to rewrite: 1"), "{text}");
        assert!(
            text.contains("TXXX:MusicIndex Track Guid -> TXXX:Podcast Item Guid"),
            "{text}"
        );
    }

    /// A backup folder in the music folder would put copies into the library
    /// scan, so the write stops.
    #[test]
    fn adr_0086_retag_refuses_a_backup_folder_in_the_music_folder() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();

        let error = write_library(&conn, music.path(), &music.path().join("backup")).unwrap_err();

        assert!(error.to_string().contains("music folder"), "{error}");
    }

    /// A FLAC file holds each old frame as a Vorbis comment in capital
    /// letters. The retag finds them with any letter case.
    #[test]
    fn adr_0086_retag_renames_old_vorbis_comments() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let path = add_track(
            &conn,
            music.path(),
            "song.flac",
            &flac_with_comments(&[
                "TITLE=Song",
                "MUSICINDEX FEED GUID=feed-guid",
                "MUSICINDEX TRACK GUID=item-guid",
            ]),
        );

        let plan = plan_library(&conn, music.path()).unwrap();
        assert_eq!(plan.count(&FileState::Rewrite), 1);
        write_library(&conn, music.path(), backups.path()).unwrap();

        let tags = read_audio_tags(&path).unwrap();
        let labels = tags
            .fields
            .iter()
            .map(|field| field.frame_id.to_ascii_lowercase())
            .collect::<Vec<_>>();
        assert!(
            labels.contains(&"txxx:podcast feed guid".to_owned()),
            "{labels:?}"
        );
        assert!(
            labels.contains(&"txxx:podcast item guid".to_owned()),
            "{labels:?}"
        );
        assert!(
            !labels.iter().any(|label| label.contains("musicindex")),
            "{labels:?}"
        );
        let second = plan_library(&conn, music.path()).unwrap();
        assert_eq!(second.count(&FileState::Current), 1);
    }

    /// A track with no stored route: the write removes the old route frame
    /// and writes no new route frame (ADR 0076 Decision 9). The edit list is
    /// empty, and the crate still removes the old frame.
    #[test]
    fn adr_0086_retag_removes_a_route_frame_with_no_stored_route() {
        let conn = setup_db();
        let music = tempfile::tempdir().unwrap();
        let backups = tempfile::tempdir().unwrap();
        let path = add_track(&conn, music.path(), "song.mp3", b"");
        let mut tag = Tag::new();
        tag.add_frame(ExtendedText {
            description: "MusicIndex Value Routes".into(),
            value: "old routes".into(),
        });
        tag.write_to_path(&path, Version::Id3v24).unwrap();

        let report = write_library(&conn, music.path(), backups.path()).unwrap();

        assert!(
            matches!(
                report.results.as_slice(),
                [(_, FileResult::Written(write))]
                    if write.removed_old_frames == ["TXXX:MusicIndex Value Routes"]
            ),
            "{:?}",
            report.results
        );
        let labels = frame_labels(&path);
        assert!(
            !labels
                .iter()
                .any(|label| label.contains("MusicIndex") || label == frames::VALUE_ROUTES),
            "{labels:?}"
        );
        let second = plan_library(&conn, music.path()).unwrap();
        assert!(plan_text(&second).contains("Files to rewrite: 0"));
    }

    #[test]
    fn timed_table_gives_srt_with_the_same_times() {
        let srt = timed_table_to_srt(
            "Timecode\tTranscription\n00:00:01.500\tFirst\n01:02:03.004\tSecond\n",
        )
        .unwrap();
        assert_eq!(
            srt,
            "1\n00:00:01.500 --> 00:00:01.500\nFirst\n\n2\n01:02:03.004 --> 01:02:03.004\nSecond\n\n"
        );
        assert_eq!(timed_table_to_srt("Plain text"), None);
    }

    #[test]
    fn new_label_uses_the_descriptor_with_any_case() {
        assert_eq!(
            new_label("SYLT:MusicIndex Transcript"),
            Some(frames::TRANSCRIPT_TIMED)
        );
        assert_eq!(
            new_label("USLT:MusicIndex Transcript"),
            Some(frames::TRANSCRIPT_TEXT)
        );
        assert_eq!(
            new_label("TXXX:MUSICINDEX FEED GUID"),
            Some(frames::FEED_GUID)
        );
        assert_eq!(
            new_label("TXXX:MUSICINDEX ALBUM DESCRIPTION"),
            Some(frames::FEED_DESCRIPTION)
        );
        assert_eq!(new_label("TXXX:MusicIndex Something"), None);
    }

    /// A FLAC file with a STREAMINFO block and a Vorbis comment block.
    fn flac_with_comments(comments: &[&str]) -> Vec<u8> {
        let mut bytes = b"fLaC".to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 34]);
        bytes.extend_from_slice(&4096_u16.to_be_bytes());
        bytes.extend_from_slice(&4096_u16.to_be_bytes());
        bytes.extend_from_slice(&[0; 6]);
        let packed: u64 = (44_100_u64 << 44) | (1_u64 << 41) | (15_u64 << 36);
        bytes.extend_from_slice(&packed.to_be_bytes());
        bytes.extend_from_slice(&[0; 16]);
        let mut block = Vec::new();
        let vendor = b"hitit test";
        block.extend_from_slice(&u32::try_from(vendor.len()).unwrap().to_le_bytes());
        block.extend_from_slice(vendor);
        block.extend_from_slice(&u32::try_from(comments.len()).unwrap().to_le_bytes());
        for comment in comments {
            block.extend_from_slice(&u32::try_from(comment.len()).unwrap().to_le_bytes());
            block.extend_from_slice(comment.as_bytes());
        }
        let length = u32::try_from(block.len()).unwrap().to_be_bytes();
        bytes.extend_from_slice(&[0x04, length[1], length[2], length[3]]);
        bytes.extend_from_slice(&block);
        // A last PADDING block and the start of a frame.
        bytes.extend_from_slice(&[0x81, 0x00, 0x00, 16]);
        bytes.extend_from_slice(&[0; 16]);
        bytes.extend_from_slice(&[0xFF, 0xF8, 0x69, 0x08, 0x00, 0x00]);
        bytes
    }
}

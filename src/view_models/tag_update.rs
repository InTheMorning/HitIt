//! View model of the "Update n file(s)" button, its popup and its report
//! (ADR 0076 Decision 8, packet 004).
//!
//! The Music section renders these displays. The view model decides the
//! count, the labels, the in-use mark and each report sentence. No renderer
//! decides which file is in use or if the button is available.

#![warn(clippy::pedantic)]

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::application::commands::tag_update::{TagUpdateWriteReport, TagUpdateWriteStatus};
use crate::application::queries::tag_update::{
    TagFrameChange, TagUpdateFile, TagUpdateFileContent, TagUpdateScan,
};
use crate::db::tag_update_skips::TagUpdateSkip;
use crate::metadata::PODCAST_VALUE_ROUTES_FRAME;
use crate::runtime::TagUpdateSnapshot;

/// The longest file or stored value that a popup line shows, in
/// characters. A longer value ends with an ellipsis.
const VALUE_PREVIEW_CHARS: usize = 80;

/// The mark of a file that the show plays or holds.
pub(crate) const IN_USE_MARK: &str = "In use by the show";
/// The mark of a file that the scan could not read.
pub(crate) const UNREADABLE_MARK: &str = "Cannot read";

/// Availability of the "Update n file(s)" button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagUpdateAvailability {
    /// The button opens the popup.
    Available,
    /// A confirm writes files. The button waits for its result.
    Writing,
}

/// Display contract of the "Update n file(s)" button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateButtonDisplay {
    pub(crate) button_id: &'static str,
    pub(crate) label: String,
    pub(crate) a11y_label: String,
    pub(crate) availability: TagUpdateAvailability,
    pub(crate) enabled: bool,
}

/// The number of files that the scan lists, or zero with no scan.
#[must_use]
pub(crate) fn file_count(snapshot: Option<&TagUpdateSnapshot>) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.scan.as_ref())
        .map_or(0, TagUpdateScan::count)
}

/// The number of skipped changes that the scan lists, or zero with no scan.
#[must_use]
pub(crate) fn skip_count(snapshot: Option<&TagUpdateSnapshot>) -> usize {
    snapshot
        .and_then(|snapshot| snapshot.scan.as_ref())
        .map_or(0, TagUpdateScan::skip_count)
}

/// The button, or `None` when the scan lists no file and no skip.
///
/// With no file and one or more skips, the button is the control for skips
/// only (ADR 0108 Decision 4). It has the label "Skipped tag changes (n)".
#[must_use]
pub(crate) fn button(snapshot: Option<&TagUpdateSnapshot>) -> Option<TagUpdateButtonDisplay> {
    let count = file_count(snapshot);
    let snapshot_ref = snapshot?;
    let availability = if snapshot_ref.writing {
        TagUpdateAvailability::Writing
    } else {
        TagUpdateAvailability::Available
    };
    if count == 0 {
        let skips = skip_count(snapshot);
        if skips == 0 {
            return None;
        }
        let a11y_label = match availability {
            TagUpdateAvailability::Available => {
                "Show the skipped tag changes. Select a change to write it".to_owned()
            }
            TagUpdateAvailability::Writing => {
                "The app writes the tags. Wait for the result".to_owned()
            }
        };
        return Some(TagUpdateButtonDisplay {
            button_id: "library-tag-update",
            label: format!("Skipped tag changes ({skips})"),
            a11y_label,
            availability,
            enabled: availability == TagUpdateAvailability::Available,
        });
    }
    let files = files_text(count);
    let (label, a11y_label) = match availability {
        TagUpdateAvailability::Available => (
            format!("Update {files}"),
            format!("Show the {files} whose tags differ from the stored metadata"),
        ),
        TagUpdateAvailability::Writing => (
            format!("Updating {files}"),
            "The app writes the tags. Wait for the result".to_owned(),
        ),
    };
    Some(TagUpdateButtonDisplay {
        button_id: "library-tag-update",
        label,
        a11y_label,
        availability,
        enabled: availability == TagUpdateAvailability::Available,
    })
}

/// The state of a popup check box. The renderer maps it to the check box
/// primitive (ADR 0084 Decision 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagUpdateCheckState {
    Checked,
    /// Some, but not each, group of the file is selected.
    Mixed,
    NotChecked,
}

/// The target of a popup check box. The id string of a box names its target,
/// so the click handler needs no other data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagUpdateCheck {
    /// All groups of a file.
    File(i64),
    /// One change group of a file.
    Group(i64, usize),
    /// One skipped change group of a file.
    Skip(i64, usize),
}

impl TagUpdateCheck {
    /// The element id of the check box.
    #[must_use]
    pub(crate) fn id(self) -> String {
        match self {
            Self::File(track) => format!("tag-update-file-{track}"),
            Self::Group(track, group) => format!("tag-update-group-{track}-{group}"),
            Self::Skip(track, group) => format!("tag-update-skip-{track}-{group}"),
        }
    }

    /// The target that an element id names, or `None` for another id.
    #[must_use]
    pub(crate) fn parse(id: &str) -> Option<Self> {
        let pair = |rest: &str| -> Option<(i64, usize)> {
            let (track, group) = rest.split_once('-')?;
            Some((track.parse().ok()?, group.parse().ok()?))
        };
        if let Some(rest) = id.strip_prefix("tag-update-file-") {
            return rest.parse().ok().map(Self::File);
        }
        if let Some(rest) = id.strip_prefix("tag-update-group-") {
            return pair(rest).map(|(track, group)| Self::Group(track, group));
        }
        id.strip_prefix("tag-update-skip-")
            .and_then(pair)
            .map(|(track, group)| Self::Skip(track, group))
    }
}

/// The frames of a file that one stored key shares, in the order of the
/// scan. The scan gives each such frame the same group (ADR 0108).
fn frame_groups(frames: &[TagFrameChange]) -> Vec<(usize, Vec<&TagFrameChange>)> {
    let mut groups: Vec<(usize, Vec<&TagFrameChange>)> = Vec::new();
    for frame in frames {
        match groups.iter_mut().find(|(group, _)| *group == frame.group) {
            Some((_, members)) => members.push(frame),
            None => groups.push((frame.group, vec![frame])),
        }
    }
    groups
}

/// A change group that the host can select, and its skipped mark. A file in
/// use or unreadable has no group.
fn selectable_groups(file: &TagUpdateFile) -> Vec<(usize, bool)> {
    if file.in_use {
        return Vec::new();
    }
    match &file.content {
        TagUpdateFileContent::Differs { frames, .. } => frame_groups(frames)
            .into_iter()
            .map(|(group, members)| (group, members.iter().any(|frame| frame.skipped)))
            .collect(),
        TagUpdateFileContent::Unreadable { .. } => Vec::new(),
    }
}

/// The choice of the host in the popup (ADR 0108 Decisions 1, 2 and 4).
///
/// The selection is made from one scan and keeps it. The popup, the plan
/// and the confirm all read this scan, so a later scan does not change a
/// popup that is open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateSelection {
    scan: TagUpdateScan,
    /// Each (track, group) that is not skipped and that the host did not
    /// clear.
    selected: BTreeSet<(i64, usize)>,
    /// Each (track, group) that is skipped and that the host selected.
    selected_skips: BTreeSet<(i64, usize)>,
    skipped_folded: bool,
}

impl TagUpdateSelection {
    /// Selects each group that is not skipped. Clears each skip. The section
    /// of the skips is folded, unless the scan lists no file to update.
    #[must_use]
    pub(crate) fn new(scan: &TagUpdateScan) -> Self {
        let selected = scan
            .files
            .iter()
            .flat_map(|file| {
                selectable_groups(file)
                    .into_iter()
                    .filter(|(_, skipped)| !skipped)
                    .map(|(group, _)| (file.track_id, group))
            })
            .collect();
        Self {
            skipped_folded: scan.count() > 0,
            scan: scan.clone(),
            selected,
            selected_skips: BTreeSet::new(),
        }
    }

    /// `true` when the section "Skipped (n)" is folded.
    #[must_use]
    pub(crate) fn skipped_folded(&self) -> bool {
        self.skipped_folded
    }

    pub(crate) fn toggle_skipped_folded(&mut self) {
        self.skipped_folded = !self.skipped_folded;
    }

    fn file(&self, track_id: i64) -> Option<&TagUpdateFile> {
        self.scan
            .files
            .iter()
            .find(|file| file.track_id == track_id)
    }

    /// The groups of a file that are not skipped.
    fn open_groups(&self, track_id: i64) -> Vec<usize> {
        self.file(track_id)
            .map(|file| {
                selectable_groups(file)
                    .into_iter()
                    .filter(|(_, skipped)| !skipped)
                    .map(|(group, _)| group)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Applies the click on a check box.
    pub(crate) fn toggle(&mut self, check: TagUpdateCheck) {
        match check {
            TagUpdateCheck::File(track_id) => self.toggle_file(track_id),
            TagUpdateCheck::Group(track_id, group) => self.toggle_group(track_id, group),
            TagUpdateCheck::Skip(track_id, group) => self.toggle_skip(track_id, group),
        }
    }

    pub(crate) fn toggle_group(&mut self, track_id: i64, group: usize) {
        if !self.open_groups(track_id).contains(&group) {
            return;
        }
        if !self.selected.remove(&(track_id, group)) {
            self.selected.insert((track_id, group));
        }
    }

    /// Selects each group of the file when one or more groups are cleared.
    /// In the other condition, it clears each group.
    pub(crate) fn toggle_file(&mut self, track_id: i64) {
        let groups = self.open_groups(track_id);
        if self.file_state(track_id) == TagUpdateCheckState::Checked {
            for group in groups {
                self.selected.remove(&(track_id, group));
            }
        } else {
            for group in groups {
                self.selected.insert((track_id, group));
            }
        }
    }

    pub(crate) fn toggle_skip(&mut self, track_id: i64, group: usize) {
        let skipped = self
            .file(track_id)
            .is_some_and(|file| selectable_groups(file).contains(&(group, true)));
        if !skipped {
            return;
        }
        if !self.selected_skips.remove(&(track_id, group)) {
            self.selected_skips.insert((track_id, group));
        }
    }

    /// The state of the box of a file. It covers only the groups that are
    /// not skipped.
    #[must_use]
    pub(crate) fn file_state(&self, track_id: i64) -> TagUpdateCheckState {
        let groups = self.open_groups(track_id);
        let chosen = groups
            .iter()
            .filter(|group| self.selected.contains(&(track_id, **group)))
            .count();
        if chosen == 0 {
            TagUpdateCheckState::NotChecked
        } else if chosen == groups.len() {
            TagUpdateCheckState::Checked
        } else {
            TagUpdateCheckState::Mixed
        }
    }

    #[must_use]
    pub(crate) fn group_state(&self, track_id: i64, group: usize) -> TagUpdateCheckState {
        if self.selected.contains(&(track_id, group)) {
            TagUpdateCheckState::Checked
        } else {
            TagUpdateCheckState::NotChecked
        }
    }

    #[must_use]
    pub(crate) fn skip_state(&self, track_id: i64, group: usize) -> TagUpdateCheckState {
        if self.selected_skips.contains(&(track_id, group)) {
            TagUpdateCheckState::Checked
        } else {
            TagUpdateCheckState::NotChecked
        }
    }

    /// `true` when the file has one or more groups that are not skipped, or
    /// when one or more skips are selected (ADR 0108 Decision 2.2).
    #[must_use]
    pub(crate) fn confirm_enabled(&self) -> bool {
        !self.selected_skips.is_empty()
            || self
                .scan
                .files
                .iter()
                .any(|file| selectable_groups(file).iter().any(|(_, skipped)| !skipped))
    }

    /// The files to write and the skips to record. A file in use or
    /// unreadable is not in the plan.
    #[must_use]
    pub(crate) fn confirm_plan(&self) -> (Vec<TagUpdateFile>, Vec<TagUpdateSkip>) {
        let mut files = Vec::new();
        let mut skips = Vec::new();
        for file in &self.scan.files {
            let track_id = file.track_id;
            let mut keep = BTreeSet::new();
            let mut cleared = BTreeSet::new();
            for (group, skipped) in selectable_groups(file) {
                if skipped {
                    if self.selected_skips.contains(&(track_id, group)) {
                        keep.insert(group);
                    }
                } else if self.selected.contains(&(track_id, group)) {
                    keep.insert(group);
                } else {
                    cleared.insert(group);
                }
            }
            files.extend(file.with_groups(&keep));
            skips.extend(file.skips_for_groups(&cleared));
        }
        (files, skips)
    }
}

/// Role of the mark of one popup file. The mark text states the reason, so
/// the role is never the only signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagUpdateMarkRole {
    InUse,
    Unreadable,
}

/// A check box of the popup. The accessible label names the file and the
/// frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateCheckDisplay {
    /// The element id. [`TagUpdateCheck::parse`] reads it.
    pub(crate) id: String,
    pub(crate) state: TagUpdateCheckState,
    pub(crate) a11y_label: String,
}

/// One line of a popup file, with an optional check box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateLineDisplay {
    pub(crate) text: String,
    pub(crate) check: Option<TagUpdateCheckDisplay>,
}

/// One file of the popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateFileDisplay {
    pub(crate) title: String,
    pub(crate) album: Option<String>,
    /// One line for each change group to write, or the read error.
    pub(crate) lines: Vec<TagUpdateLineDisplay>,
    pub(crate) mark: Option<(&'static str, TagUpdateMarkRole)>,
    /// The box of the file. A file in use or unreadable has none.
    pub(crate) check: Option<TagUpdateCheckDisplay>,
}

/// The section "Skipped (n)" of the popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateSkippedDisplay {
    pub(crate) id: &'static str,
    pub(crate) label: String,
    pub(crate) a11y_label: String,
    pub(crate) folded: bool,
    /// The count of skipped change groups.
    pub(crate) count: usize,
    /// One item for each file. One line for each skipped group.
    pub(crate) files: Vec<TagUpdateFileDisplay>,
}

/// Display contract of the popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdatePopupDisplay {
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) cancel_button_id: &'static str,
    pub(crate) cancel_label: &'static str,
    pub(crate) cancel_a11y_label: &'static str,
    pub(crate) confirm_button_id: &'static str,
    pub(crate) confirm_label: &'static str,
    pub(crate) confirm_a11y_label: String,
    pub(crate) confirm_enabled: bool,
    pub(crate) files: Vec<TagUpdateFileDisplay>,
    pub(crate) skipped: Option<TagUpdateSkippedDisplay>,
}

/// The popup of the selection, or `None` when the scan lists no file and no
/// skip. The popup reads the scan that the selection keeps.
#[must_use]
pub(crate) fn popup(selection: &TagUpdateSelection) -> Option<TagUpdatePopupDisplay> {
    let scan = &selection.scan;
    let skip_total = scan.skip_count();
    if scan.count() == 0 && skip_total == 0 {
        return None;
    }
    let visible = scan
        .files
        .iter()
        .filter(|file| match &file.content {
            TagUpdateFileContent::Unreadable { .. } => true,
            TagUpdateFileContent::Differs { frames, .. } => {
                frames.iter().any(|frame| !frame.skipped)
            }
        })
        .collect::<Vec<_>>();
    let writable = visible
        .iter()
        .filter(|file| file.has_write_action() && !file.in_use)
        .count();
    let held = visible.len() - writable;
    let skips_only = scan.count() == 0;
    let (title, mut message) = if skips_only {
        (
            "Skipped tag changes".to_owned(),
            "Each change below is skipped. Select a change to write its stored value to the tag."
                .to_owned(),
        )
    } else {
        (
            format!("Update {}?", files_text(scan.count())),
            format!(
                "The app writes the stored metadata to the tags of {}. Each file below shows the frames to write.",
                files_text(writable)
            ),
        )
    };
    if held > 0 {
        let _ = write!(
            message,
            " The app does not write {}: each one is in use by the show or cannot be read.",
            files_text(held)
        );
    }
    let planned = selection.confirm_plan().0.len();
    let skipped = (skip_total > 0).then(|| skipped_display(selection, skip_total));
    Some(TagUpdatePopupDisplay {
        title,
        message,
        cancel_button_id: "library-tag-update-cancel",
        cancel_label: "Cancel",
        cancel_a11y_label: "Close the list and write no file",
        confirm_button_id: "library-tag-update-confirm",
        confirm_label: "Write Tags",
        confirm_a11y_label: format!(
            "Write the stored metadata to the tags of {}",
            files_text(planned)
        ),
        confirm_enabled: selection.confirm_enabled(),
        files: visible
            .into_iter()
            .map(|file| file_display(selection, file))
            .collect(),
        skipped,
    })
}

fn frame_names(members: &[&TagFrameChange]) -> String {
    members
        .iter()
        .map(|frame| frame.frame_label.as_str())
        .collect::<Vec<_>>()
        .join(" and ")
}

fn check_display(
    check: TagUpdateCheck,
    state: TagUpdateCheckState,
    a11y_label: String,
) -> TagUpdateCheckDisplay {
    TagUpdateCheckDisplay {
        id: check.id(),
        state,
        a11y_label,
    }
}

fn mark_of(file: &TagUpdateFile) -> Option<(&'static str, TagUpdateMarkRole)> {
    if file.in_use {
        Some((IN_USE_MARK, TagUpdateMarkRole::InUse))
    } else if matches!(file.content, TagUpdateFileContent::Unreadable { .. }) {
        Some((UNREADABLE_MARK, TagUpdateMarkRole::Unreadable))
    } else {
        None
    }
}

fn file_display(selection: &TagUpdateSelection, file: &TagUpdateFile) -> TagUpdateFileDisplay {
    let track_id = file.track_id;
    let selectable = !selectable_groups(file).is_empty();
    let lines = match &file.content {
        TagUpdateFileContent::Differs { frames, .. } => frame_groups(frames)
            .into_iter()
            .filter(|(_, members)| members.iter().any(|frame| !frame.skipped))
            .map(|(group, members)| TagUpdateLineDisplay {
                text: members
                    .iter()
                    .map(|frame| frame_line(frame))
                    .collect::<Vec<_>>()
                    .join("\n"),
                check: selectable.then(|| {
                    check_display(
                        TagUpdateCheck::Group(track_id, group),
                        selection.group_state(track_id, group),
                        format!("Write {} of {}", frame_names(&members), file.title),
                    )
                }),
            })
            .collect(),
        TagUpdateFileContent::Unreadable { error } => vec![TagUpdateLineDisplay {
            text: format!("The app cannot read the tags of this file: {error}"),
            check: None,
        }],
    };
    TagUpdateFileDisplay {
        title: file.title.clone(),
        album: file.album.clone(),
        lines,
        mark: mark_of(file),
        check: selectable.then(|| {
            check_display(
                TagUpdateCheck::File(track_id),
                selection.file_state(track_id),
                format!("Write all changes of {}", file.title),
            )
        }),
    }
}

fn skipped_display(selection: &TagUpdateSelection, count: usize) -> TagUpdateSkippedDisplay {
    let files = selection
        .scan
        .files
        .iter()
        .filter_map(|file| {
            let TagUpdateFileContent::Differs { frames, .. } = &file.content else {
                return None;
            };
            let selectable = !selectable_groups(file).is_empty();
            let lines = frame_groups(frames)
                .into_iter()
                .filter(|(_, members)| members.iter().any(|frame| frame.skipped))
                .map(|(group, members)| TagUpdateLineDisplay {
                    text: members
                        .iter()
                        .map(|frame| frame_line(frame))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    check: selectable.then(|| {
                        check_display(
                            TagUpdateCheck::Skip(file.track_id, group),
                            selection.skip_state(file.track_id, group),
                            format!(
                                "Write the skipped {} of {}",
                                frame_names(&members),
                                file.title
                            ),
                        )
                    }),
                })
                .collect::<Vec<_>>();
            (!lines.is_empty()).then(|| TagUpdateFileDisplay {
                title: file.title.clone(),
                album: file.album.clone(),
                lines,
                mark: mark_of(file),
                check: None,
            })
        })
        .collect();
    TagUpdateSkippedDisplay {
        id: "library-tag-update-skipped",
        label: format!("Skipped ({count})"),
        a11y_label: format!(
            "Show or hide the {count} skipped tag changes. Select a change to write it"
        ),
        folded: selection.skipped_folded(),
        count,
        files,
    }
}

fn frame_line(frame: &TagFrameChange) -> String {
    let name = if frame.frame_label == PODCAST_VALUE_ROUTES_FRAME {
        format!("Payment route ({})", frame.frame_label)
    } else {
        frame.frame_label.clone()
    };
    let file_value = frame
        .file_value
        .as_deref()
        .map_or_else(|| "no value".to_owned(), preview);
    format!(
        "{name}: {} (file: {file_value})",
        preview(&frame.expected_value)
    )
}

fn preview(value: &str) -> String {
    let value = value.trim();
    if value.chars().count() <= VALUE_PREVIEW_CHARS {
        return value.to_owned();
    }
    let mut short = value.chars().take(VALUE_PREVIEW_CHARS).collect::<String>();
    short.push('\u{2026}');
    short
}

/// Role of one report line. The line text states the result, so the role is
/// never the only signal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TagUpdateReportRole {
    Written,
    NotWritten,
    Failed,
}

/// One line of the report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateReportLine {
    pub(crate) text: String,
    pub(crate) role: TagUpdateReportRole,
}

/// The report of the latest confirm and the latest scan error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TagUpdateReportDisplay {
    pub(crate) summary: String,
    pub(crate) lines: Vec<TagUpdateReportLine>,
}

/// The report, or `None` when no confirm ran and the scan had no error.
/// `request_failed` is `true` when the runtime did not accept the latest
/// confirm.
#[must_use]
pub(crate) fn report(
    snapshot: Option<&TagUpdateSnapshot>,
    request_failed: bool,
) -> Option<TagUpdateReportDisplay> {
    if request_failed {
        return Some(TagUpdateReportDisplay {
            summary:
                "The tag update wrote no file. The background runtime did not accept the request."
                    .to_owned(),
            lines: Vec::new(),
        });
    }
    let snapshot = snapshot?;
    if let Some(error) = &snapshot.write_error {
        return Some(TagUpdateReportDisplay {
            summary: format!("The tag update wrote no file. {error}"),
            lines: Vec::new(),
        });
    }
    if let Some(write) = &snapshot.last_write {
        return Some(write_report(write));
    }
    snapshot
        .scan_error
        .as_ref()
        .map(|error| TagUpdateReportDisplay {
            summary: format!(
                "The app could not compare the file tags with the stored metadata. {error}"
            ),
            lines: Vec::new(),
        })
}

/// The words of each tag write that failed in the latest confirm: the error
/// of a confirm that wrote no file, and each file with a write error. The
/// status bar event list records them (ADR 0101 Decision 7). The words are
/// the report words of [`report`] and the report lines of the file.
#[must_use]
pub(crate) fn failure_words(snapshot: &TagUpdateSnapshot) -> Vec<String> {
    if let Some(error) = &snapshot.write_error {
        return vec![format!("The tag update wrote no file. {error}")];
    }
    snapshot
        .last_write
        .iter()
        .flat_map(|write| write_report(write).lines)
        .filter(|line| line.role == TagUpdateReportRole::Failed)
        .map(|line| line.text)
        .collect()
}

/// The words of a confirm that the runtime did not accept.
#[must_use]
pub(crate) fn request_failure_words() -> String {
    report(None, true).map_or_else(String::new, |report| report.summary)
}

fn write_report(write: &TagUpdateWriteReport) -> TagUpdateReportDisplay {
    let at = chrono::DateTime::from_timestamp_micros(write.finished_at_us).map_or_else(
        || "an unknown time".to_owned(),
        |at| at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    );
    let written = write.written_count();
    let not_written = write.results.len() - written;
    let mut summary = format!("Tag update at {at}: the app wrote {}.", files_text(written));
    if not_written > 0 {
        let _ = write!(
            summary,
            " It did not write {}. The button keeps their count.",
            files_text(not_written)
        );
    }
    let lines = write
        .results
        .iter()
        .map(|result| {
            let title = &result.title;
            match &result.status {
                TagUpdateWriteStatus::Written { frames, skipped } => TagUpdateReportLine {
                    text: if skipped.is_empty() {
                        format!("Wrote {} to {title}.", frames_text(*frames))
                    } else {
                        format!(
                            "Wrote {} to {title}. Skipped: {}.",
                            frames_text(*frames),
                            skipped.join("; ")
                        )
                    },
                    role: TagUpdateReportRole::Written,
                },
                TagUpdateWriteStatus::InUse => TagUpdateReportLine {
                    text: format!("Did not write {title}. The file is in use by the show."),
                    role: TagUpdateReportRole::NotWritten,
                },
                TagUpdateWriteStatus::Unreadable => TagUpdateReportLine {
                    text: format!("Did not write {title}. The app cannot read its tags."),
                    role: TagUpdateReportRole::NotWritten,
                },
                TagUpdateWriteStatus::Failed { error } => TagUpdateReportLine {
                    text: format!("Could not write {title}. {error}"),
                    role: TagUpdateReportRole::Failed,
                },
            }
        })
        .collect();
    TagUpdateReportDisplay { summary, lines }
}

pub(crate) fn files_text(count: usize) -> String {
    if count == 1 {
        "1 file".to_owned()
    } else {
        format!("{count} files")
    }
}

fn frames_text(count: usize) -> String {
    if count == 1 {
        "1 frame".to_owned()
    } else {
        format!("{count} frames")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::application::commands::tag_update::TagUpdateWriteResult;

    fn differs(track_id: i64, in_use: bool) -> TagUpdateFile {
        TagUpdateFile {
            track_id,
            title: format!("Song {track_id}"),
            album: Some("Album One".into()),
            path: PathBuf::from(format!("/music/song-{track_id}.mp3")),
            in_use,
            content: TagUpdateFileContent::Differs {
                frames: vec![TagFrameChange {
                    group: 0,
                    skipped: false,
                    frame_label: "TIT2".into(),
                    file_value: Some("Old title".into()),
                    expected_value: format!("Song {track_id}"),
                }],
                shared_key_edits: Vec::new(),
            },
        }
    }

    fn snapshot(files: Vec<TagUpdateFile>) -> TagUpdateSnapshot {
        TagUpdateSnapshot {
            scan: Some(TagUpdateScan {
                files,
                stale_skips: Vec::new(),
            }),
            ..TagUpdateSnapshot::default()
        }
    }

    fn selection_of(files: Vec<TagUpdateFile>) -> TagUpdateSelection {
        TagUpdateSelection::new(&TagUpdateScan {
            files,
            stale_skips: Vec::new(),
        })
    }

    fn frame(group: usize, label: &str, skipped: bool) -> TagFrameChange {
        TagFrameChange {
            group,
            skipped,
            frame_label: label.into(),
            file_value: Some("Old".into()),
            expected_value: format!("New {label}"),
        }
    }

    /// A file with one frame in each given (group, label, skipped) entry.
    fn grouped(track_id: i64, frames: &[(usize, &str, bool)]) -> TagUpdateFile {
        let mut file = differs(track_id, false);
        file.content = TagUpdateFileContent::Differs {
            frames: frames
                .iter()
                .map(|(group, label, skipped)| frame(*group, label, *skipped))
                .collect(),
            shared_key_edits: Vec::new(),
        };
        file
    }

    fn written_labels(plan: &(Vec<TagUpdateFile>, Vec<TagUpdateSkip>)) -> Vec<(i64, Vec<String>)> {
        plan.0
            .iter()
            .map(|file| {
                let TagUpdateFileContent::Differs { frames, .. } = &file.content else {
                    panic!("a planned file has frames");
                };
                (
                    file.track_id,
                    frames
                        .iter()
                        .map(|frame| frame.frame_label.clone())
                        .collect(),
                )
            })
            .collect()
    }

    /// R4-05: the button exists only when the count is above 0. Its label
    /// carries the count, and it has an accessibility label.
    #[test]
    fn adr_0076_tag_update_button_shows_only_above_zero_with_count() {
        assert_eq!(button(None), None);
        assert_eq!(button(Some(&TagUpdateSnapshot::default())), None);
        assert_eq!(button(Some(&snapshot(Vec::new()))), None);

        let one = button(Some(&snapshot(vec![differs(1, false)]))).unwrap();
        assert_eq!(one.label, "Update 1 file");
        assert!(one.enabled);

        let three = button(Some(&snapshot(vec![
            differs(1, false),
            differs(2, true),
            differs(3, false),
        ])))
        .unwrap();
        assert_eq!(three.label, "Update 3 files");
        assert_eq!(
            three.a11y_label,
            "Show the 3 files whose tags differ from the stored metadata"
        );
        assert_eq!(three.availability, TagUpdateAvailability::Available);

        let mut writing = snapshot(vec![differs(1, false)]);
        writing.writing = true;
        let busy = button(Some(&writing)).unwrap();
        assert_eq!(busy.availability, TagUpdateAvailability::Writing);
        assert!(!busy.enabled);
    }

    /// The popup lists each file with its title, album, frames and in-use
    /// mark. It has one confirm and one cancel button.
    #[test]
    fn adr_0076_tag_update_popup_lists_files_with_frames_and_marks() {
        let mut unreadable = differs(3, false);
        unreadable.content = TagUpdateFileContent::Unreadable {
            error: "probe failed".into(),
        };
        let popup = popup(&selection_of(vec![
            differs(1, false),
            differs(2, true),
            unreadable,
        ]))
        .unwrap();

        assert_eq!(popup.title, "Update 3 files?");
        assert_eq!(popup.confirm_label, "Write Tags");
        assert_eq!(popup.cancel_label, "Cancel");
        assert_eq!(
            popup.confirm_a11y_label,
            "Write the stored metadata to the tags of 1 file"
        );
        assert_eq!(popup.files.len(), 3);
        assert_eq!(popup.files[0].title, "Song 1");
        assert_eq!(popup.files[0].album.as_deref(), Some("Album One"));
        assert_eq!(
            popup.files[0].lines[0].text,
            "TIT2: Song 1 (file: Old title)"
        );
        assert_eq!(popup.files[0].mark, None);
        assert_eq!(
            popup.files[1].mark,
            Some((IN_USE_MARK, TagUpdateMarkRole::InUse))
        );
        assert_eq!(
            popup.files[2].mark,
            Some((UNREADABLE_MARK, TagUpdateMarkRole::Unreadable))
        );
        assert!(popup.files[2].lines[0].text.contains("probe failed"));
    }

    #[test]
    fn adr_0108_tag_update_popup_omits_a_file_with_only_skipped_changes() {
        let mut skipped = differs(2, false);
        if let TagUpdateFileContent::Differs { frames, .. } = &mut skipped.content {
            frames[0].skipped = true;
        }
        let state = snapshot(vec![differs(1, false), skipped]);
        let display = popup(&TagUpdateSelection::new(state.scan.as_ref().unwrap())).unwrap();
        assert_eq!(file_count(Some(&state)), 1);
        assert_eq!(display.title, "Update 1 file?");
        assert_eq!(display.files.len(), 1);
        assert_eq!(display.files[0].title, "Song 1");
    }

    /// ADR 0108 task 002 criterion 1.
    #[test]
    fn adr_0108_selection_starts_with_each_open_group_selected_and_each_skip_cleared() {
        let selection = selection_of(vec![grouped(
            1,
            &[(0, "TIT2", false), (1, "TPE1", false), (2, "TALB", true)],
        )]);
        assert_eq!(selection.file_state(1), TagUpdateCheckState::Checked);
        assert_eq!(selection.group_state(1, 0), TagUpdateCheckState::Checked);
        assert_eq!(selection.group_state(1, 1), TagUpdateCheckState::Checked);
        assert_eq!(selection.skip_state(1, 2), TagUpdateCheckState::NotChecked);
        assert!(selection.skipped_folded());
    }

    /// ADR 0108 task 002 criterion 2.
    #[test]
    fn adr_0108_selection_file_box_is_mixed_when_one_group_is_cleared() {
        let mut selection =
            selection_of(vec![grouped(1, &[(0, "TIT2", false), (1, "TPE1", false)])]);
        selection.toggle_group(1, 0);
        assert_eq!(selection.file_state(1), TagUpdateCheckState::Mixed);
        selection.toggle_group(1, 1);
        assert_eq!(selection.file_state(1), TagUpdateCheckState::NotChecked);
        selection.toggle_file(1);
        assert_eq!(selection.file_state(1), TagUpdateCheckState::Checked);
        assert_eq!(selection.group_state(1, 0), TagUpdateCheckState::Checked);
        assert_eq!(selection.group_state(1, 1), TagUpdateCheckState::Checked);
        selection.toggle_group(1, 0);
        selection.toggle_file(1);
        assert_eq!(selection.file_state(1), TagUpdateCheckState::Checked);
        selection.toggle_file(1);
        assert_eq!(selection.file_state(1), TagUpdateCheckState::NotChecked);
    }

    /// ADR 0108 task 002 criterion 3.
    #[test]
    fn adr_0108_selection_plan_writes_selected_groups_and_skips_cleared_ones() {
        let mut selection = selection_of(vec![
            grouped(1, &[(0, "TIT2", false), (1, "TPE1", false)]),
            grouped(2, &[(0, "TIT2", false)]),
        ]);
        selection.toggle_group(1, 1);
        let plan = selection.confirm_plan();
        assert_eq!(
            written_labels(&plan),
            vec![(1, vec!["TIT2".to_owned()]), (2, vec!["TIT2".to_owned()])]
        );
        assert_eq!(plan.1.len(), 1);
        assert_eq!(plan.1[0].track_id, 1);
        assert_eq!(plan.1[0].frame_label, "TPE1");
        assert_eq!(plan.1[0].stored_value, "New TPE1");
    }

    /// ADR 0108 task 002 criterion 4.
    #[test]
    fn adr_0108_selection_with_all_groups_cleared_confirms_only_the_skips() {
        let mut selection =
            selection_of(vec![grouped(1, &[(0, "TIT2", false), (1, "TPE1", false)])]);
        selection.toggle_file(1);
        assert!(selection.confirm_enabled());
        let plan = selection.confirm_plan();
        assert!(plan.0.is_empty());
        assert_eq!(plan.1.len(), 2);
    }

    /// ADR 0108 task 002 criterion 5.
    #[test]
    fn adr_0108_selection_with_only_skips_needs_a_selected_skip() {
        let mut selection = selection_of(vec![grouped(1, &[(0, "TIT2", true)])]);
        assert!(!selection.confirm_enabled());
        assert!(!selection.skipped_folded());
        selection.toggle_skip(1, 0);
        assert!(selection.confirm_enabled());
        let plan = selection.confirm_plan();
        assert_eq!(written_labels(&plan), vec![(1, vec!["TIT2".to_owned()])]);
        assert!(plan.1.is_empty());
    }

    /// ADR 0108 task 002 criterion 6.
    #[test]
    fn adr_0108_selection_two_changes_of_one_group_are_one_line() {
        let selection = selection_of(vec![grouped(1, &[(0, "TCOP", false), (0, "WCOP", false)])]);
        let display = popup(&selection).unwrap();
        assert_eq!(display.files[0].lines.len(), 1);
        let line = &display.files[0].lines[0];
        assert!(line.text.contains("TCOP") && line.text.contains("WCOP"));
        assert_eq!(
            line.check.as_ref().unwrap().a11y_label,
            "Write TCOP and WCOP of Song 1"
        );
        assert_eq!(selection.confirm_plan().0.len(), 1);
    }

    /// ADR 0108 task 002 criterion 7.
    #[test]
    fn adr_0108_popup_gives_no_check_box_to_a_file_in_use_or_unreadable() {
        let mut unreadable = differs(3, false);
        unreadable.content = TagUpdateFileContent::Unreadable {
            error: "probe failed".into(),
        };
        let selection = selection_of(vec![differs(1, false), differs(2, true), unreadable]);
        let display = popup(&selection).unwrap();
        assert!(display.files[0].check.is_some());
        assert!(display.files[0].lines[0].check.is_some());
        for held in &display.files[1..] {
            assert!(held.check.is_none());
            assert!(held.lines.iter().all(|line| line.check.is_none()));
        }
        let plan = selection.confirm_plan();
        assert_eq!(plan.0.len(), 1);
        assert_eq!(plan.0[0].track_id, 1);
    }

    /// ADR 0108 task 002 criterion 8.
    #[test]
    fn adr_0108_popup_skipped_section_counts_groups_and_shows_values() {
        let mut selection = selection_of(vec![
            differs(1, false),
            grouped(2, &[(0, "TIT2", true), (1, "TPE1", true)]),
        ]);
        let display = popup(&selection).unwrap();
        let skipped = display.skipped.unwrap();
        assert_eq!(skipped.label, "Skipped (2)");
        assert_eq!(skipped.count, 2);
        assert!(skipped.folded);
        assert_eq!(skipped.files.len(), 1);
        assert_eq!(skipped.files[0].lines.len(), 2);
        assert_eq!(skipped.files[0].lines[0].text, "TIT2: New TIT2 (file: Old)");
        assert_eq!(
            skipped.files[0].lines[0].check.as_ref().unwrap().a11y_label,
            "Write the skipped TIT2 of Song 2"
        );
        selection.toggle_skipped_folded();
        assert!(!popup(&selection).unwrap().skipped.unwrap().folded);
        let none = popup(&selection_of(vec![differs(1, false)])).unwrap();
        assert!(none.skipped.is_none());
    }

    #[test]
    fn adr_0108_check_id_names_its_target() {
        for check in [
            TagUpdateCheck::File(7),
            TagUpdateCheck::Group(7, 2),
            TagUpdateCheck::Skip(7, 3),
        ] {
            assert_eq!(TagUpdateCheck::parse(&check.id()), Some(check));
        }
        assert_eq!(TagUpdateCheck::parse("other"), None);
    }

    /// ADR 0108 task 002 criterion 9.
    #[test]
    fn adr_0108_button_for_skips_only_shows_the_skip_count() {
        let skips_only = snapshot(vec![grouped(1, &[(0, "TIT2", true), (1, "TPE1", true)])]);
        let button = button(Some(&skips_only)).unwrap();
        assert_eq!(button.label, "Skipped tag changes (2)");
        assert!(button.enabled);
        let display = popup(&TagUpdateSelection::new(skips_only.scan.as_ref().unwrap())).unwrap();
        assert_eq!(display.title, "Skipped tag changes");
        assert!(display.files.is_empty());
        assert!(!display.skipped.unwrap().folded);
        assert!(!display.confirm_enabled);
        assert!(super::button(Some(&snapshot(vec![differs(1, false)])))
            .unwrap()
            .label
            .starts_with("Update"));
        assert_eq!(super::button(Some(&snapshot(Vec::new()))), None);
    }

    /// Each result has its own report line, and the summary gives the
    /// recorded time.
    #[test]
    fn adr_0076_tag_update_report_names_each_result_and_its_time() {
        let state = TagUpdateSnapshot {
            last_write: Some(TagUpdateWriteReport {
                results: vec![
                    TagUpdateWriteResult {
                        track_id: 1,
                        title: "Song 1".into(),
                        status: TagUpdateWriteStatus::Written {
                            frames: 2,
                            skipped: Vec::new(),
                        },
                    },
                    TagUpdateWriteResult {
                        track_id: 2,
                        title: "Song 2".into(),
                        status: TagUpdateWriteStatus::InUse,
                    },
                    TagUpdateWriteResult {
                        track_id: 3,
                        title: "Song 3".into(),
                        status: TagUpdateWriteStatus::Failed {
                            error: "disk full".into(),
                        },
                    },
                ],
                finished_at_us: 1_000_000_000,
            }),
            ..TagUpdateSnapshot::default()
        };

        let report = report(Some(&state), false).unwrap();

        assert_eq!(
            report.summary,
            "Tag update at 1970-01-01 00:16:40 UTC: the app wrote 1 file. It did not write 2 files. The button keeps their count."
        );
        let lines = report
            .lines
            .iter()
            .map(|line| (line.text.as_str(), line.role))
            .collect::<Vec<_>>();
        assert_eq!(
            lines,
            vec![
                ("Wrote 2 frames to Song 1.", TagUpdateReportRole::Written),
                (
                    "Did not write Song 2. The file is in use by the show.",
                    TagUpdateReportRole::NotWritten
                ),
                (
                    "Could not write Song 3. disk full",
                    TagUpdateReportRole::Failed
                ),
            ]
        );
    }
}

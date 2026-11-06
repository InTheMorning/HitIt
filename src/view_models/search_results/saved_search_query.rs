//! The query field of a saved-search page (ADR 0089 Decision 4, ADR 0087
//! Decisions 4 and 6).
//!
//! A click on the query makes it a field. Each typed text is the live query
//! of the page, and its results change while the curator types. "Save
//! Query" or Enter keeps the text through the writer of the Library. Escape
//! cancels it. A saved search keeps its text until a keep saves a new one.
//!
//! [`SavedSearchQueryEdit`] is a pure state machine. It holds no GPUI type
//! and calls no writer. The host gives it the outcome of each keep.

#![warn(clippy::pedantic)]

use super::{SavedSearchId, SearchPageActionDisplay, SearchPageActionStyle};

/// The text that shows when another saved search has the typed text.
pub(crate) const SAVED_SEARCH_QUERY_CONFLICT_TEXT: &str =
    "Another saved search has this text. Change the text, or press Escape to cancel.";

/// The text that shows when the typed text is blank.
pub(crate) const SAVED_SEARCH_QUERY_BLANK_TEXT: &str =
    "A saved search needs a query. Type a word, a phrase or a field.";

/// The placeholder of the query field.
pub(crate) const SAVED_SEARCH_QUERY_PLACEHOLDER: &str = "Saved search query";

/// The open query field of a saved search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearchQueryEdit {
    id: SavedSearchId,
    /// The stored text. It changes only when a keep saves a new text.
    stored: String,
    /// The text in the field, as the last change gave it.
    typed: String,
    /// The message of the last keep, if it did not close the field.
    message: Option<String>,
}

/// The outcome of a keep, as the writer of the Library gives it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchQueryKeep {
    /// The text is saved. The field closes, and the stored text is this
    /// trimmed text.
    Saved(String),
    /// Another saved search has the text. The field stays open.
    Conflict,
    /// The text is blank. The field stays open.
    Blank,
    /// The saved search no longer exists. The field closes.
    Missing,
    /// The database refused the write. The field stays open, with the error.
    Failed(String),
}

impl SavedSearchQueryEdit {
    /// Opens the field of the saved search `id` with its stored `query`. The
    /// field shows the stored text and no message.
    #[must_use]
    pub(crate) fn open(id: SavedSearchId, query: &str) -> Self {
        Self {
            id,
            stored: query.to_owned(),
            typed: query.to_owned(),
            message: None,
        }
    }

    /// The saved search that the field edits.
    #[must_use]
    pub(crate) const fn id(&self) -> SavedSearchId {
        self.id
    }

    /// The stored text. Cancel restores it.
    #[must_use]
    pub(crate) fn stored(&self) -> &str {
        &self.stored
    }

    /// The text in the field, as the last change gave it. The page reads
    /// the live query instead, so only the tests read it.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn typed(&self) -> &str {
        &self.typed
    }

    /// The live query: the typed text without its outer spaces. The live run
    /// of the page uses this text.
    #[must_use]
    pub(crate) fn live_query(&self) -> &str {
        self.typed.trim()
    }

    /// The message of the last keep that did not close the field.
    #[must_use]
    pub(crate) fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Records a change of the field text. A change clears the message of
    /// the last keep. The stored text does not change.
    pub(crate) fn set_typed(&mut self, text: &str) {
        text.clone_into(&mut self.typed);
        self.message = None;
    }

    /// Applies the outcome of a keep. Gives `true` when the field closes, so
    /// the caller drops this state. A `Saved` outcome makes its text the
    /// stored text. Each other outcome that keeps the field open sets its
    /// message, and the stored text does not change.
    pub(crate) fn apply_keep(&mut self, outcome: SavedSearchQueryKeep) -> bool {
        match outcome {
            SavedSearchQueryKeep::Saved(text) => {
                self.stored = text;
                self.typed.clone_from(&self.stored);
                self.message = None;
                true
            }
            SavedSearchQueryKeep::Missing => {
                self.message = None;
                true
            }
            SavedSearchQueryKeep::Conflict => {
                self.message = Some(SAVED_SEARCH_QUERY_CONFLICT_TEXT.to_owned());
                false
            }
            SavedSearchQueryKeep::Blank => {
                self.message = Some(SAVED_SEARCH_QUERY_BLANK_TEXT.to_owned());
                false
            }
            SavedSearchQueryKeep::Failed(error) => {
                self.message = Some(format!("The app could not save the query: {error}"));
                false
            }
        }
    }
}

/// The controls of the open query field: "Save Query" and "Cancel", and the
/// message of the last keep.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SavedSearchQueryFieldDisplay {
    pub(crate) placeholder: &'static str,
    pub(crate) save: SearchPageActionDisplay,
    pub(crate) cancel: SearchPageActionDisplay,
    pub(crate) message: Option<String>,
}

/// The display of the open query field. "Save Query" is available when the
/// typed text is not blank.
#[must_use]
pub(crate) fn saved_search_query_field(
    edit: &SavedSearchQueryEdit,
) -> SavedSearchQueryFieldDisplay {
    let typed = edit.live_query();
    let available = !typed.is_empty();
    SavedSearchQueryFieldDisplay {
        placeholder: SAVED_SEARCH_QUERY_PLACEHOLDER,
        save: SearchPageActionDisplay {
            id: "music-saved-search-query-save",
            label: "Save Query",
            a11y_label: format!("Save the query \"{typed}\" of this saved search"),
            available,
            style: SearchPageActionStyle::Action,
        },
        cancel: SearchPageActionDisplay {
            id: "music-saved-search-query-cancel",
            label: "Cancel",
            a11y_label: format!("Cancel the query change, and keep \"{}\"", edit.stored()),
            available: true,
            style: SearchPageActionStyle::Plain,
        },
        message: edit.message().map(str::to_owned),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> SavedSearchId {
        SavedSearchId(7)
    }

    /// ADR 0089 Decision 4: opening the field keeps the stored text, shows
    /// no message, and makes the typed text the stored text.
    #[test]
    fn adr_0089_query_edit_open_keeps_the_stored_text() {
        let edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        assert_eq!(edit.id(), id());
        assert_eq!(edit.stored(), "bluegrass");
        assert_eq!(edit.typed(), "bluegrass");
        assert_eq!(edit.message(), None);
    }

    /// ADR 0087 Decision 4: a typed text is the live query. The stored text
    /// does not change while the curator types.
    #[test]
    fn adr_0089_query_edit_typing_changes_the_live_query_only() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        edit.set_typed("  bluegrass banjo  ");
        assert_eq!(edit.live_query(), "bluegrass banjo");
        assert_eq!(edit.stored(), "bluegrass");
    }

    /// ADR 0089 Decision 4: "Save Query" with a new text closes the field,
    /// and the stored text becomes the saved text.
    #[test]
    fn adr_0089_query_edit_keep_saves_and_closes() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        edit.set_typed("bluegrass banjo");
        let closed = edit.apply_keep(SavedSearchQueryKeep::Saved("bluegrass banjo".to_owned()));
        assert!(closed);
        assert_eq!(edit.stored(), "bluegrass banjo");
        assert_eq!(edit.typed(), "bluegrass banjo");
        assert_eq!(edit.message(), None);
    }

    /// ADR 0087 Decision 6: a unique conflict keeps the field open with the
    /// typed text and the conflict message. The stored text stays.
    #[test]
    fn adr_0089_query_edit_conflict_keeps_the_field_open() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        edit.set_typed("ambient");
        let closed = edit.apply_keep(SavedSearchQueryKeep::Conflict);
        assert!(!closed, "the field stays open");
        assert_eq!(edit.message(), Some(SAVED_SEARCH_QUERY_CONFLICT_TEXT));
        assert_eq!(edit.typed(), "ambient", "the typed text stays");
        assert_eq!(
            edit.stored(),
            "bluegrass",
            "the stored text does not change"
        );
    }

    /// ADR 0089 Decision 4: a blank text and a database error keep the field
    /// open with a message. Neither changes the stored text.
    #[test]
    fn adr_0089_query_edit_blank_and_failure_keep_the_field_open() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        edit.set_typed("   ");
        assert!(!edit.apply_keep(SavedSearchQueryKeep::Blank));
        assert_eq!(edit.message(), Some(SAVED_SEARCH_QUERY_BLANK_TEXT));
        assert!(!edit.apply_keep(SavedSearchQueryKeep::Failed("disk full".to_owned())));
        assert_eq!(
            edit.message(),
            Some("The app could not save the query: disk full")
        );
        assert_eq!(edit.stored(), "bluegrass");
    }

    /// ADR 0089 Decision 4: a saved search that no longer exists closes the
    /// field. The stored text is the text of the open, unchanged.
    #[test]
    fn adr_0089_query_edit_missing_closes_the_field() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        edit.set_typed("jazz");
        assert!(edit.apply_keep(SavedSearchQueryKeep::Missing));
        assert_eq!(edit.stored(), "bluegrass");
    }

    /// ADR 0089 Decision 4: a change of the text after a refused keep clears
    /// its message.
    #[test]
    fn adr_0089_query_edit_typing_clears_the_message() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        edit.apply_keep(SavedSearchQueryKeep::Conflict);
        edit.set_typed("ambient banjo");
        assert_eq!(edit.message(), None);
        assert_eq!(edit.typed(), "ambient banjo");
    }

    /// ADR 0089 Decision 4: "Save Query" is available for a text that is not
    /// blank, and it is not available for a blank text. The conflict message
    /// reaches the display.
    #[test]
    fn adr_0089_query_edit_display_gives_typed_controls() {
        let mut edit = SavedSearchQueryEdit::open(id(), "bluegrass");
        let display = saved_search_query_field(&edit);
        assert_eq!(display.save.label, "Save Query");
        assert!(display.save.available);
        assert_eq!(display.cancel.label, "Cancel");
        assert_eq!(display.message, None);

        edit.set_typed("  ");
        assert!(!saved_search_query_field(&edit).save.available);

        edit.set_typed("ambient");
        edit.apply_keep(SavedSearchQueryKeep::Conflict);
        assert_eq!(
            saved_search_query_field(&edit).message.as_deref(),
            Some(SAVED_SEARCH_QUERY_CONFLICT_TEXT)
        );
    }
}

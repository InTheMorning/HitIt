//! The field list under the toolbar search field (ADR 0087 Decision 3).
//!
//! The list shows the query fields, the fixed values of a field, the error
//! of the field under the text cursor and the scope conflict of the query.
//! This module gives each row, its text, its accessibility label, its typed
//! action and its availability. It also gives the open state of the list,
//! the outcome of each list key and the text edit of each row.
//!
//! After `artist:`, `release:` and `playlist:`, the list shows the values of
//! the field from the Library and, for `artist:`, from `MusicIndex` (ADR
//! 0087 packet 005). Each value row has a source mark in words. A loading
//! row and an error row tell the state of the `MusicIndex` request.
//!
//! The crate `musicindex-search` gives the field rows, the fixed value rows,
//! the range of each edit, the field and prefix of the name rows and the
//! field error (musicindex ADR 0052 R-QUERY-7, ADR 0091). This module has no
//! GPUI type and no I/O. The renderer only composes the rows.

#![warn(clippy::pedantic)]

use std::ops::Range;

use musicindex_search::field_list::{field_list_rows, FieldListRow, Library};

use crate::view_models::index_name_list::name_sort_key;
use crate::view_models::search_field_values::{
    artist_value_request_key, FieldValueRows, FieldValueSource, IndexArtistValues,
};
use crate::view_models::search_query::{
    cursor_term, quote_field_value, FixedValueInfo, QueryFieldInfo, QueryFieldName, QueryScope,
    SearchQuery,
};

/// The note of a row that `in:index` makes unavailable.
const NOTE_INDEX_SCOPE: &str = "Not available with in:index";

/// The note of the `in:index` row in a Library-only query.
const NOTE_LIBRARY_SCOPE: &str = "Not available with a Library-only field or in:library";

/// The label of the row that waits for the `MusicIndex` names.
const LOADING_LABEL: &str = "Loading names from Music Index";

/// The accessibility label of the loading row.
const LOADING_A11Y_LABEL: &str = "Loading artist names from Music Index";

/// The label of the row of a failed `MusicIndex` name request.
const INDEX_ERROR_LABEL: &str = "Names from Music Index did not load";

/// The note of the error row.
const INDEX_ERROR_NOTE: &str = "The Library names show. The next change of the text tries again.";

/// The accessibility label of the error row.
const INDEX_ERROR_A11Y_LABEL: &str =
    "Artist names from Music Index did not load. The Library names show.";

/// The text of the scope conflict notice.
const SCOPE_CONFLICT_TEXT: &str =
    "This query asks for the Library only and for Music Index only. It shows no rows.";

/// The open state of the field list.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FieldListOpening {
    /// The list does not show.
    #[default]
    Closed,
    /// The list shows.
    Open,
}

/// One event that changes the open state of the field list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldListEvent {
    /// The search field got the keyboard focus.
    Focus,
    /// The search field lost the keyboard focus.
    Blur,
    /// A left mouse button went down in the search field.
    Click,
    /// The text of the search field changed.
    Change,
    /// The cursor or the selection moved, and the text did not change.
    CursorMoved,
    /// The Escape key closed the list.
    Escape,
    /// A mouse button went down outside the search field and the list.
    OutsideClick,
    /// The search ran.
    Submit,
    /// The view put the edit of a row into the text.
    Inserted,
}

/// The open state, the focus and the selected row of the field list.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FieldListState {
    /// The open state.
    pub opening: FieldListOpening,
    /// `true` while the search field has the keyboard focus.
    pub focused: bool,
    /// The index of the selected row.
    pub selected: Option<usize>,
    /// The count of openings. It seeds the runnable example of each text
    /// field, so each visit can show another value (musicindex ADR 0058
    /// R-QUERY-12 item 2). The app sets the first value.
    pub visit: u64,
}

impl FieldListState {
    /// Changes the state for one event.
    pub fn apply(&mut self, event: FieldListEvent) {
        let was_open = self.opening == FieldListOpening::Open;
        self.apply_event(event);
        if !was_open && self.opening == FieldListOpening::Open {
            self.visit = self.visit.wrapping_add(1);
        }
    }

    fn apply_event(&mut self, event: FieldListEvent) {
        match event {
            FieldListEvent::Focus => {
                // ADR 0087 Decision 3: only a click or typing opens the list.
                // The field gets the focus again when the window gets the
                // focus, and from each shortcut.
                self.focused = true;
                self.selected = None;
            }
            FieldListEvent::Click | FieldListEvent::Inserted => {
                // After a row goes into the text, the list stays open for
                // the next term. Thus the state does not depend on the order
                // of this event and the `Change` of the edit.
                self.opening = FieldListOpening::Open;
                self.selected = None;
            }
            FieldListEvent::Change => {
                if self.focused {
                    self.opening = FieldListOpening::Open;
                }
                self.selected = None;
            }
            FieldListEvent::CursorMoved => self.selected = None,
            FieldListEvent::Escape | FieldListEvent::Submit | FieldListEvent::OutsideClick => {
                self.opening = FieldListOpening::Closed;
            }
            FieldListEvent::Blur => {
                self.focused = false;
                self.opening = FieldListOpening::Closed;
            }
        }
    }

    /// Sets the selected row.
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }
}

/// The kind of one row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldListRowKind {
    /// One query field.
    Field { field: QueryFieldName },
    /// One fixed value of a field.
    FixedValue {
        field: QueryFieldName,
        word: &'static str,
    },
    /// One value of the Library or of `MusicIndex` (packet 005).
    Value {
        field: QueryFieldName,
        text: String,
        source: FieldValueSource,
    },
    /// The row waits for the `MusicIndex` names of the field.
    Loading { field: QueryFieldName },
    /// The `MusicIndex` name request of the field failed.
    IndexError { field: QueryFieldName },
}

/// The kind of one example text of a field row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldExampleKind {
    /// A fixed example of a field that names no person or release.
    Fixed,
    /// The placeholder of a text field. It runs no search (musicindex ADR
    /// 0058 R-QUERY-11).
    Placeholder,
    /// An example that the field list builds from loaded data (musicindex
    /// ADR 0058 R-QUERY-12).
    Runnable,
}

/// The click on an example text. It puts the example into the search field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldExampleClick {
    /// The text edit of the click.
    pub edit: FieldListEdit,
    /// The byte range to select after the edit, in the changed text. For a
    /// placeholder it is the placeholder text. For a runnable example it is
    /// the empty range at the end.
    pub select: Range<usize>,
}

/// One example text of a field row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldListExample {
    /// The element id of the example text.
    pub element_id: String,
    /// The text of the example, with its field token.
    pub text: String,
    /// The kind of the example.
    pub kind: FieldExampleKind,
    /// The click on the example. `None` for a fixed example: the click goes
    /// to the row.
    pub click: Option<FieldExampleClick>,
}

/// The typed action of one row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldListRowAction {
    /// Puts the field token into the text.
    InsertField(QueryFieldName),
    /// Puts the value word into the text.
    InsertValue {
        field: QueryFieldName,
        word: &'static str,
    },
    /// Puts the value text into the text, in quotes when it needs them.
    InsertText { field: QueryFieldName, text: String },
    /// The row has no action.
    Inert,
}

/// The reason that a row is not available.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldListRowUnavailable {
    /// The row conflicts with the scope of the query.
    ScopeConflict,
    /// The row waits for the `MusicIndex` names.
    Loading,
    /// The `MusicIndex` name request failed.
    IndexError,
}

/// The availability of one row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldListRowAvailability {
    Available,
    Unavailable(FieldListRowUnavailable),
}

impl FieldListRowAvailability {
    /// `true` when the row is available.
    #[must_use]
    pub fn is_available(self) -> bool {
        self == Self::Available
    }
}

/// The mark of one row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldListMark {
    /// Only the Library can apply the field.
    LibraryOnly,
    /// The source of a value row. The mark is a word, so the color is not
    /// the only cue.
    Source(FieldValueSource),
}

impl FieldListMark {
    /// The word of the mark.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::LibraryOnly => "Library only",
            Self::Source(source) => source.label(),
        }
    }
}

/// The text edit of one row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldListEdit {
    /// The byte range of the old text that the edit replaces.
    pub range: Range<usize>,
    /// The new text for that range.
    pub text: String,
    /// The cursor offset in the changed text.
    pub cursor: usize,
}

/// One row of the field list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldListRowVm {
    /// The element id of the row.
    pub element_id: String,
    /// The field or the fixed value of the row.
    pub kind: FieldListRowKind,
    /// The field token or the value word.
    pub label: String,
    /// The values of the field, or the description of the value.
    pub values_text: Option<String>,
    /// The example texts of the field, in the order that the row shows them.
    pub examples: Vec<FieldListExample>,
    /// The mark of the row.
    pub mark: Option<FieldListMark>,
    /// The reason that the row is not available.
    pub note: Option<String>,
    /// The accessibility label of the row.
    pub a11y_label: String,
    /// The typed action of the row.
    pub action: FieldListRowAction,
    /// The availability of the row.
    pub availability: FieldListRowAvailability,
    /// The text edit that the row puts into the text.
    pub edit: FieldListEdit,
    /// `true` for the selected row.
    pub selected: bool,
}

/// The kind of one notice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldListNoticeKind {
    /// The field under the cursor has a value that is not correct.
    FieldError,
    /// The query asks for the Library only and for the Index only.
    ScopeConflict,
}

/// One notice of the field list. A notice is not a row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldListNoticeVm {
    /// The kind of the notice.
    pub kind: FieldListNoticeKind,
    /// The text of the notice.
    pub text: String,
    /// The accessibility label of the notice.
    pub a11y_label: String,
}

/// One key that the field list reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldListKey {
    Up,
    Down,
    Tab,
    Enter,
    Escape,
}

/// The outcome of one key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldListKeyOutcome {
    /// Select the row with this index.
    Select(usize),
    /// Put this edit into the text.
    Apply(FieldListEdit),
    /// Close the list.
    Close,
    /// The list does not use the key. The search field gets it.
    Pass,
}

/// The field list under the toolbar search field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldListVm {
    /// The rows, in the order of the field table or of the fixed values.
    pub rows: Vec<FieldListRowVm>,
    /// The notices. The field error comes first.
    pub notices: Vec<FieldListNoticeVm>,
    /// The index of the selected row.
    pub selected: Option<usize>,
    /// The accessibility label of the list.
    pub a11y_label: String,
}

impl FieldListVm {
    /// The element id of the scrolling row list.
    pub const LIST_ID: &'static str = "search-field-list";

    /// Builds the field list for the text, the selection and the state.
    ///
    /// `selection` is the byte range of the selection. An empty range is
    /// the cursor. `values` gives the Library values and the state of the
    /// `MusicIndex` name request. The result is `None` when the list is
    /// closed, when the search field has no focus, when the selection is not
    /// empty, or when the list has no row and no notice.
    #[must_use]
    pub(crate) fn build(
        text: &str,
        selection: Range<usize>,
        state: &FieldListState,
        values: &FieldValueRows<'_>,
    ) -> Option<Self> {
        if state.opening == FieldListOpening::Closed || !state.focused {
            return None;
        }
        if !selection.is_empty() {
            return None;
        }
        let cursor = selection.start.min(text.len());
        let query = SearchQuery::parse(text);
        let scope = query.scope();
        // The crate decides the field rows, the fixed value rows, the range
        // that an edit replaces, the name request and the field error
        // (musicindex ADR 0052 R-QUERY-7, ADR 0091). This module adds the
        // names, the marks, the availability and the edits.
        let list = field_list_rows(text, cursor, Library::Present);
        let mut rows: Vec<FieldListRowVm> = Vec::new();
        if let Some(range) = list.edit.clone() {
            let add_space = range.end == cursor_term(text, cursor).range.end
                && !text[range.end..]
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace);
            let edit = ValueEdit { range, add_space };
            rows.extend(list.rows.iter().map(|row| match row {
                FieldListRow::Field(info) => {
                    field_row(info, edit.range.clone(), scope, values, state.visit)
                }
                FieldListRow::Value { field, value } => value_row(*field, value, &edit, scope),
            }));
            if let Some(request) = &list.names {
                let library = values.library.matching(request.field, &request.prefix);
                rows.extend(library.iter().enumerate().map(|(position, entry)| {
                    text_value_row(
                        request.field,
                        &entry.text,
                        FieldValueSource::Library,
                        position,
                        &edit,
                        scope,
                    )
                }));
                if request.field == QueryFieldName::Artist {
                    let request_key = artist_value_request_key(text, cursor);
                    if request_key.is_some() && request_key.as_deref() == values.index.key() {
                        let library_keys: Vec<&str> =
                            library.iter().map(|entry| entry.key.as_str()).collect();
                        rows.extend(index_rows(
                            request.field,
                            values.index,
                            &library_keys,
                            &edit,
                            scope,
                            cursor,
                        ));
                    }
                }
            }
        }
        let notices = notices(list.error, scope);
        if rows.is_empty() && notices.is_empty() {
            return None;
        }
        let selected = state.selected.filter(|index| *index < rows.len());
        if let Some(index) = selected {
            rows[index].selected = true;
        }
        let a11y_label = list_a11y_label(rows.len());
        Some(Self {
            rows,
            notices,
            selected,
            a11y_label,
        })
    }

    /// Gives the outcome of one key.
    #[must_use]
    pub fn key(&self, key: FieldListKey) -> FieldListKeyOutcome {
        let available: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.availability.is_available())
            .map(|(index, _)| index)
            .collect();
        let selected = self.selected.filter(|index| available.contains(index));
        match key {
            FieldListKey::Down => match (available.first(), selected) {
                (None, _) => FieldListKeyOutcome::Pass,
                (Some(first), None) => FieldListKeyOutcome::Select(*first),
                (Some(first), Some(current)) => FieldListKeyOutcome::Select(
                    available
                        .iter()
                        .copied()
                        .find(|index| *index > current)
                        .unwrap_or(*first),
                ),
            },
            FieldListKey::Up => match (available.last(), selected) {
                (None, _) => FieldListKeyOutcome::Pass,
                (Some(last), None) => FieldListKeyOutcome::Select(*last),
                (Some(last), Some(current)) => FieldListKeyOutcome::Select(
                    available
                        .iter()
                        .rev()
                        .copied()
                        .find(|index| *index < current)
                        .unwrap_or(*last),
                ),
            },
            FieldListKey::Tab => selected
                .or_else(|| available.first().copied())
                .map_or(FieldListKeyOutcome::Pass, |index| {
                    FieldListKeyOutcome::Apply(self.rows[index].edit.clone())
                }),
            FieldListKey::Enter => selected.map_or(FieldListKeyOutcome::Pass, |index| {
                FieldListKeyOutcome::Apply(self.rows[index].edit.clone())
            }),
            FieldListKey::Escape => FieldListKeyOutcome::Close,
        }
    }
}

/// Builds the row of one field. `edit_range` is the typed field name.
fn field_row(
    info: &'static QueryFieldInfo,
    edit_range: Range<usize>,
    scope: QueryScope,
    values: &FieldValueRows<'_>,
    seed: u64,
) -> FieldListRowVm {
    let availability = if scope == QueryScope::IndexOnly && info.library_only {
        FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict)
    } else {
        FieldListRowAvailability::Available
    };
    let note = unavailable_note(availability, scope);
    let mark = info.library_only.then_some(FieldListMark::LibraryOnly);
    let element_id = format!("search-field-list-field-{}", bare_token(info.token));
    let examples = field_examples(info, &element_id, &edit_range, values, seed);
    let mut a11y_label = format!("Field {}, {}", info.token, info.values_text);
    for example in &examples {
        match example.kind {
            FieldExampleKind::Fixed | FieldExampleKind::Runnable => {
                a11y_label.push_str(", for example ");
            }
            FieldExampleKind::Placeholder => a11y_label.push_str(", placeholder "),
        }
        a11y_label.push_str(&example.text);
    }
    if let Some(mark) = mark {
        a11y_label.push_str(", ");
        a11y_label.push_str(mark.label());
    }
    push_note(&mut a11y_label, note.as_deref());
    let cursor = edit_range.start + info.token.len();
    FieldListRowVm {
        element_id,
        kind: FieldListRowKind::Field { field: info.name },
        label: info.token.to_owned(),
        values_text: Some(info.values_text.to_owned()),
        examples,
        mark,
        note,
        a11y_label,
        action: FieldListRowAction::InsertField(info.name),
        availability,
        edit: FieldListEdit {
            range: edit_range,
            text: info.token.to_owned(),
            cursor,
        },
        selected: false,
    }
}

/// The example texts of one field (musicindex ADR 0058).
///
/// A field with a placeholder gives the placeholder first. When the loaded
/// data has a value for the field, a runnable example follows it. The
/// runnable example is the token and `quote_field_value` of that value, and
/// the seed picks the value. A field with no placeholder gives its fixed
/// example.
fn field_examples(
    info: &'static QueryFieldInfo,
    row_id: &str,
    edit_range: &Range<usize>,
    values: &FieldValueRows<'_>,
    seed: u64,
) -> Vec<FieldListExample> {
    let Some(placeholder) = info.placeholder else {
        return vec![FieldListExample {
            element_id: format!("{row_id}-example-0"),
            text: info.example.to_owned(),
            kind: FieldExampleKind::Fixed,
            click: None,
        }];
    };
    let start = edit_range.start + info.token.len();
    let quoted = quote_field_value(placeholder);
    // The selection holds the placeholder text, without its quotes.
    let inner = usize::from(quoted.len() != placeholder.len());
    let text = format!("{}{quoted}", info.token);
    let cursor = edit_range.start + text.len();
    let mut examples = vec![FieldListExample {
        element_id: format!("{row_id}-example-0"),
        text: text.clone(),
        kind: FieldExampleKind::Placeholder,
        click: Some(FieldExampleClick {
            edit: FieldListEdit {
                range: edit_range.clone(),
                text,
                cursor,
            },
            select: start + inner..start + quoted.len() - inner,
        }),
    }];
    if let Some(value) = values.library.example_value(info.name, seed) {
        let text = format!("{}{}", info.token, quote_field_value(value));
        let cursor = edit_range.start + text.len();
        examples.push(FieldListExample {
            element_id: format!("{row_id}-example-1"),
            text: text.clone(),
            kind: FieldExampleKind::Runnable,
            click: Some(FieldExampleClick {
                edit: FieldListEdit {
                    range: edit_range.clone(),
                    text,
                    cursor,
                },
                select: cursor..cursor,
            }),
        });
    }
    examples
}

/// The value item that a value row replaces, and whether a space follows
/// the new value.
struct ValueEdit {
    /// The byte range of the value item, with its quotes.
    range: Range<usize>,
    /// `true` when the edit adds one space after the value.
    add_space: bool,
}

impl ValueEdit {
    /// The edit that puts `value` into the value item.
    fn edit(&self, value: &str) -> FieldListEdit {
        let mut text = value.to_owned();
        if self.add_space {
            text.push(' ');
        }
        let cursor = self.range.start + text.len();
        FieldListEdit {
            range: self.range.clone(),
            text,
            cursor,
        }
    }
}

/// Builds the row of one fixed value.
fn value_row(
    field: QueryFieldName,
    value: &'static FixedValueInfo,
    value_edit: &ValueEdit,
    scope: QueryScope,
) -> FieldListRowVm {
    let token = field.info().token;
    let conflict = match scope {
        QueryScope::IndexOnly => field.is_library_only(),
        QueryScope::LibraryOnly => {
            SearchQuery::parse(&format!("{token}{}", value.word)).scope() == QueryScope::IndexOnly
        }
        QueryScope::Everything | QueryScope::Conflict => false,
    };
    let availability = if conflict {
        FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict)
    } else {
        FieldListRowAvailability::Available
    };
    let note = unavailable_note(availability, scope);
    let mut a11y_label = format!("Value {} for {token}, {}", value.word, value.description);
    push_note(&mut a11y_label, note.as_deref());
    FieldListRowVm {
        element_id: format!(
            "search-field-list-value-{}-{}",
            bare_token(token),
            value.word
        ),
        kind: FieldListRowKind::FixedValue {
            field,
            word: value.word,
        },
        label: value.word.to_owned(),
        values_text: Some(value.description.to_owned()),
        examples: Vec::new(),
        mark: None,
        note,
        a11y_label,
        action: FieldListRowAction::InsertValue {
            field,
            word: value.word,
        },
        availability,
        edit: value_edit.edit(value.word),
        selected: false,
    }
}

/// Builds the row of one Library value or one `MusicIndex` name.
/// `position` is the position of the row in its source.
fn text_value_row(
    field: QueryFieldName,
    text: &str,
    source: FieldValueSource,
    position: usize,
    value_edit: &ValueEdit,
    scope: QueryScope,
) -> FieldListRowVm {
    let token = field.info().token;
    // The value of a Library-only field under the cursor is itself a
    // Library-only field. With `in:index`, the scope is then `Conflict`, so
    // both scopes make the row unavailable. Each such conflict comes from
    // `in:index`.
    let conflict =
        field.is_library_only() && matches!(scope, QueryScope::IndexOnly | QueryScope::Conflict);
    let (availability, note) = if conflict {
        (
            FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict),
            Some(NOTE_INDEX_SCOPE.to_owned()),
        )
    } else {
        (FieldListRowAvailability::Available, None)
    };
    let source_words = match source {
        FieldValueSource::Library => "from the Library",
        FieldValueSource::Index => "from Music Index",
    };
    let source_id = match source {
        FieldValueSource::Library => "library",
        FieldValueSource::Index => "index",
    };
    let mut a11y_label = format!("Value {text} for {token}, {source_words}");
    push_note(&mut a11y_label, note.as_deref());
    FieldListRowVm {
        element_id: format!(
            "search-field-list-value-{}-{source_id}-{position}",
            bare_token(token)
        ),
        kind: FieldListRowKind::Value {
            field,
            text: text.to_owned(),
            source,
        },
        label: text.to_owned(),
        values_text: None,
        examples: Vec::new(),
        mark: Some(FieldListMark::Source(source)),
        note,
        a11y_label,
        action: FieldListRowAction::InsertText {
            field,
            text: text.to_owned(),
        },
        availability,
        edit: value_edit.edit(&quote_field_value(text)),
        selected: false,
    }
}

/// The `MusicIndex` rows of `artist:` for the state of the request. A name
/// whose key is the key of a Library row gives no row (packet 005 decision
/// D4).
fn index_rows(
    field: QueryFieldName,
    index: &IndexArtistValues,
    library_keys: &[&str],
    value_edit: &ValueEdit,
    scope: QueryScope,
    cursor: usize,
) -> Vec<FieldListRowVm> {
    let token = bare_token(field.info().token);
    let empty_edit = FieldListEdit {
        range: cursor..cursor,
        text: String::new(),
        cursor,
    };
    match index {
        IndexArtistValues::NotRequested => Vec::new(),
        IndexArtistValues::Loaded { names, .. } => names
            .iter()
            .filter(|name| !library_keys.contains(&name_sort_key(name).as_str()))
            .enumerate()
            .map(|(position, name)| {
                text_value_row(
                    field,
                    name,
                    FieldValueSource::Index,
                    position,
                    value_edit,
                    scope,
                )
            })
            .collect(),
        IndexArtistValues::Loading { .. } => vec![FieldListRowVm {
            element_id: format!("search-field-list-loading-{token}"),
            kind: FieldListRowKind::Loading { field },
            label: LOADING_LABEL.to_owned(),
            values_text: None,
            examples: Vec::new(),
            mark: None,
            note: None,
            a11y_label: LOADING_A11Y_LABEL.to_owned(),
            action: FieldListRowAction::Inert,
            availability: FieldListRowAvailability::Unavailable(FieldListRowUnavailable::Loading),
            edit: empty_edit,
            selected: false,
        }],
        IndexArtistValues::Failed { .. } => vec![FieldListRowVm {
            element_id: format!("search-field-list-index-error-{token}"),
            kind: FieldListRowKind::IndexError { field },
            label: INDEX_ERROR_LABEL.to_owned(),
            values_text: None,
            examples: Vec::new(),
            mark: None,
            note: Some(INDEX_ERROR_NOTE.to_owned()),
            a11y_label: INDEX_ERROR_A11Y_LABEL.to_owned(),
            action: FieldListRowAction::Inert,
            availability: FieldListRowAvailability::Unavailable(
                FieldListRowUnavailable::IndexError,
            ),
            edit: empty_edit,
            selected: false,
        }],
    }
}

/// The token without its colon.
fn bare_token(token: &str) -> &str {
    token.strip_suffix(':').unwrap_or(token)
}

/// The note of an unavailable row in this scope.
fn unavailable_note(availability: FieldListRowAvailability, scope: QueryScope) -> Option<String> {
    match availability {
        FieldListRowAvailability::Available
        | FieldListRowAvailability::Unavailable(
            FieldListRowUnavailable::Loading | FieldListRowUnavailable::IndexError,
        ) => None,
        FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict) => {
            let note = if scope == QueryScope::IndexOnly {
                NOTE_INDEX_SCOPE
            } else {
                NOTE_LIBRARY_SCOPE
            };
            Some(note.to_owned())
        }
    }
}

/// Adds the note to an accessibility label.
fn push_note(label: &mut String, note: Option<&str>) {
    if let Some(note) = note {
        label.push_str(", ");
        label.push_str(note);
    }
}

/// The notices for the field error under the cursor and the scope. The field
/// error comes first.
fn notices(error: Option<String>, scope: QueryScope) -> Vec<FieldListNoticeVm> {
    let mut notices = Vec::new();
    if let Some(text) = error {
        notices.push(FieldListNoticeVm {
            kind: FieldListNoticeKind::FieldError,
            a11y_label: format!("Field error, {text}"),
            text,
        });
    }
    if scope == QueryScope::Conflict {
        notices.push(FieldListNoticeVm {
            kind: FieldListNoticeKind::ScopeConflict,
            text: SCOPE_CONFLICT_TEXT.to_owned(),
            a11y_label: format!("Scope conflict, {SCOPE_CONFLICT_TEXT}"),
        });
    }
    notices
}

/// The accessibility label of the list.
fn list_a11y_label(row_count: usize) -> String {
    if row_count == 1 {
        "Search field list, 1 row".to_owned()
    } else {
        format!("Search field list, {row_count} rows")
    }
}

impl FieldListEdit {
    /// Gives the changed text. The click on a placeholder sets this text
    /// with no change event (musicindex ADR 0058 R-QUERY-11 item 3).
    #[must_use]
    pub fn apply(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len() + self.text.len());
        out.push_str(&text[..self.range.start]);
        out.push_str(&self.text);
        out.push_str(&text[self.range.end..]);
        out
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::view_models::search_field_values::LibraryFieldValues;
    use crate::view_models::search_query::QUERY_FIELDS;

    fn state(opening: FieldListOpening) -> FieldListState {
        FieldListState {
            opening,
            focused: true,
            selected: None,
            visit: 0,
        }
    }

    fn build_at(text: &str, cursor: usize, opening: FieldListOpening) -> Option<FieldListVm> {
        FieldListVm::build(
            text,
            cursor..cursor,
            &state(opening),
            &FieldValueRows::EMPTY,
        )
    }

    fn build_end(text: &str, opening: FieldListOpening) -> Option<FieldListVm> {
        build_at(text, text.len(), opening)
    }

    fn labels(vm: &FieldListVm) -> Vec<&str> {
        vm.rows.iter().map(|row| row.label.as_str()).collect()
    }

    fn row<'a>(vm: &'a FieldListVm, label: &str) -> &'a FieldListRowVm {
        vm.rows
            .iter()
            .find(|row| row.label == label)
            .expect("the row exists")
    }

    fn with_selected(mut vm: FieldListVm, index: Option<usize>) -> FieldListVm {
        vm.selected = index;
        for (i, row) in vm.rows.iter_mut().enumerate() {
            row.selected = Some(i) == index;
        }
        vm
    }

    fn tab_text(text: &str, cursor: usize) -> (String, usize) {
        let vm = with_selected(
            build_at(text, cursor, FieldListOpening::Open).expect("the list shows"),
            Some(0),
        );
        let FieldListKeyOutcome::Apply(edit) = vm.key(FieldListKey::Tab) else {
            panic!("Tab applies the selected row");
        };
        (edit.apply(text), edit.cursor)
    }

    #[test]
    fn adr_0087_field_list_r87_4_01_empty_text_shows_each_field() {
        let vm = build_end("", FieldListOpening::Open).expect("the list shows");
        assert_eq!(vm.rows.len(), 12);
        for (row, info) in vm.rows.iter().zip(QUERY_FIELDS.iter()) {
            assert_eq!(row.kind, FieldListRowKind::Field { field: info.name });
            assert_eq!(row.label, info.token);
            assert_eq!(row.values_text.as_deref(), Some(info.values_text));
            let first = row.examples.first().expect("each field row has an example");
            assert_eq!(first.text, info.example);
            assert!(!row.selected);
        }
        assert!(vm.notices.is_empty());
        assert_eq!(vm.selected, None);
    }

    fn field_row_of<'a>(vm: &'a FieldListVm, token: &str) -> &'a FieldListRowVm {
        vm.rows
            .iter()
            .find(|row| row.label == token)
            .expect("the row exists")
    }

    fn build_visit(text: &str, library: &LibraryFieldValues, visit: u64) -> FieldListVm {
        let mut open = state(FieldListOpening::Open);
        open.visit = visit;
        FieldListVm::build(
            text,
            text.len()..text.len(),
            &open,
            &FieldValueRows {
                library,
                index: &IndexArtistValues::NotRequested,
            },
        )
        .expect("the list shows")
    }

    /// musicindex ADR 0058 R-QUERY-11: a text field shows its placeholder,
    /// the click selects the placeholder text and runs no search, and the
    /// renderer draws it muted.
    #[test]
    fn adr_0058_placeholder_row_selects_its_text_and_runs_no_search() {
        let vm = build_end("", FieldListOpening::Open).expect("the list shows");
        for info in QUERY_FIELDS
            .iter()
            .filter(|info| info.placeholder.is_some())
        {
            let placeholder = info.placeholder.expect("filtered");
            let row = field_row_of(&vm, info.token);
            let example = &row.examples[0];
            assert_eq!(example.kind, FieldExampleKind::Placeholder);
            let click = example.click.as_ref().expect("a placeholder has a click");
            let text = click.edit.apply("");
            assert_eq!(text, info.example);
            assert_eq!(&text[click.select.clone()], placeholder);
            // The edit puts text only. No action submits the search.
            assert_eq!(row.action, FieldListRowAction::InsertField(info.name));
        }
        // A field with no placeholder keeps its fixed example.
        let kind = field_row_of(&vm, "kind:");
        assert_eq!(kind.examples.len(), 1);
        assert_eq!(kind.examples[0].kind, FieldExampleKind::Fixed);
        assert!(kind.examples[0].click.is_none());
    }

    /// musicindex ADR 0058 R-QUERY-11: no row label calls a placeholder an
    /// example.
    #[test]
    fn adr_0058_no_label_calls_a_placeholder_an_example() {
        let vm = build_end("", FieldListOpening::Open).expect("the list shows");
        for info in QUERY_FIELDS
            .iter()
            .filter(|info| info.placeholder.is_some())
        {
            let row = field_row_of(&vm, info.token);
            assert!(!row.a11y_label.contains("example"), "{}", row.a11y_label);
            assert!(row.a11y_label.contains("placeholder"));
        }
    }

    /// musicindex ADR 0058 R-QUERY-12 items 1 and 3: a runnable example is
    /// the token and `quote_field_value` of a loaded value.
    #[test]
    fn adr_0058_runnable_example_quotes_a_loaded_value() {
        let library =
            LibraryFieldValues::from_texts(&["Haleen Moss"], &["Aged Friends"], &["Friday show"]);
        let vm = build_visit("", &library, 1);
        for (token, value) in [
            ("artist:", "Haleen Moss"),
            ("release:", "Aged Friends"),
            ("playlist:", "Friday show"),
        ] {
            let row = field_row_of(&vm, token);
            assert_eq!(row.examples.len(), 2, "{token}");
            let runnable = &row.examples[1];
            assert_eq!(runnable.kind, FieldExampleKind::Runnable);
            assert_eq!(
                runnable.text,
                format!("{token}{}", quote_field_value(value))
            );
            let click = runnable.click.as_ref().expect("a runnable example clicks");
            assert_eq!(click.edit.apply(""), runnable.text);
            assert_eq!(click.select, runnable.text.len()..runnable.text.len());
        }
        // The loaded data has no track title, so `title:` shows only its
        // placeholder.
        assert_eq!(field_row_of(&vm, "title:").examples.len(), 1);
    }

    /// musicindex ADR 0058 R-QUERY-12 item 2: two visits can give two
    /// examples. A visit is one opening of the list.
    #[test]
    fn adr_0058_two_visits_can_give_different_examples() {
        let library = LibraryFieldValues::from_texts(&["Ann", "Bob", "Cyd", "Dee"], &[], &[]);
        let mut list = state(FieldListOpening::Closed);
        list.apply(FieldListEvent::Focus);
        list.apply(FieldListEvent::Click);
        let first = list.visit;
        list.apply(FieldListEvent::Escape);
        list.apply(FieldListEvent::Click);
        assert_eq!(list.visit, first + 1);
        let example_of = |visit: u64| {
            let vm = build_visit("", &library, visit);
            field_row_of(&vm, "artist:").examples[1].text.clone()
        };
        let texts: BTreeSet<String> = (first..first + 4).map(example_of).collect();
        assert!(texts.len() > 1, "visits gave one example: {texts:?}");
        // The same visit gives the same example, so a redraw does not flicker.
        assert_eq!(example_of(first), example_of(first));
    }

    /// musicindex ADR 0058 R-QUERY-12 item 3: with no loaded data, the row
    /// shows only the placeholder.
    #[test]
    fn adr_0058_no_loaded_data_gives_only_the_placeholder() {
        let vm = build_visit("", &LibraryFieldValues::default(), 5);
        for token in ["artist:", "title:", "release:", "playlist:"] {
            let row = field_row_of(&vm, token);
            assert_eq!(row.examples.len(), 1, "{token}");
            assert_eq!(row.examples[0].kind, FieldExampleKind::Placeholder);
        }
    }

    #[test]
    fn adr_0087_field_list_r87_4_02_field_name_prefix_filters_rows() {
        let vm = build_at("ar", 2, FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["artist:"]);
        let vm = build_at("-ar", 3, FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["artist:"]);
        assert_eq!(build_at("zz", 2, FieldListOpening::Open), None);
    }

    #[test]
    fn adr_0087_field_list_r87_4_03_kind_shows_its_fixed_values() {
        let vm = build_at("kind:", 5, FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["album", "ep", "single"]);
        for (row, value) in vm
            .rows
            .iter()
            .zip(QueryFieldName::Kind.fixed_values().iter())
        {
            assert_eq!(row.values_text.as_deref(), Some(value.description));
            assert_eq!(
                row.kind,
                FieldListRowKind::FixedValue {
                    field: QueryFieldName::Kind,
                    word: value.word,
                }
            );
        }
        let vm = build_end("kind:s", FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["single"]);
        let vm = build_at("kind:album,", 11, FieldListOpening::Open).expect("the list shows");
        assert_eq!(vm.rows.len(), 3);
    }

    #[test]
    fn adr_0087_field_list_r87_4_04_fixed_values_of_in_is_and_has() {
        let count = |text: &str| {
            build_end(text, FieldListOpening::Open)
                .expect("the list shows")
                .rows
                .len()
        };
        assert_eq!(count("in:"), 2);
        assert_eq!(count("is:"), 4);
        assert_eq!(count("has:"), 2);
        assert_eq!(build_at("artist:", 7, FieldListOpening::Open), None);
    }

    #[test]
    fn adr_0087_field_list_r87_4_05_library_only_fields_have_a_mark() {
        let vm = build_end("", FieldListOpening::Open).expect("the list shows");
        for row in &vm.rows {
            let library_only = matches!(row.label.as_str(), "is:" | "playlist:" | "has:");
            let expected = library_only.then_some(FieldListMark::LibraryOnly);
            assert_eq!(row.mark, expected, "row {}", row.label);
        }
        assert_eq!(FieldListMark::LibraryOnly.label(), "Library only");
    }

    #[test]
    fn adr_0087_field_list_r87_4_06_field_error_under_the_cursor() {
        let vm = build_at("after:yesterday", 15, FieldListOpening::Open).expect("a notice");
        assert!(vm.rows.is_empty());
        assert_eq!(vm.notices.len(), 1);
        assert_eq!(vm.notices[0].kind, FieldListNoticeKind::FieldError);
        assert_eq!(
            vm.notices[0].text,
            "after: needs a date, for example 2026-10-01"
        );
        let vm =
            build_at("after:yesterday ar", 18, FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["artist:"]);
        assert!(vm.notices.is_empty());
    }

    #[test]
    fn adr_0087_field_list_r87_4_07_scope_conflict_makes_rows_unavailable() {
        let vm = build_at("in:index ", 9, FieldListOpening::Open).expect("the list shows");
        for row in &vm.rows {
            let library_only = matches!(row.label.as_str(), "is:" | "playlist:" | "has:");
            if library_only {
                assert_eq!(
                    row.availability,
                    FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict)
                );
                assert_eq!(row.note.as_deref(), Some("Not available with in:index"));
            } else {
                assert_eq!(row.availability, FieldListRowAvailability::Available);
                assert_eq!(row.note, None);
            }
        }
        let vm = build_at("is:ready in:", 12, FieldListOpening::Open).expect("the list shows");
        assert_eq!(
            row(&vm, "index").availability,
            FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict)
        );
        assert_eq!(
            row(&vm, "index").note.as_deref(),
            Some("Not available with a Library-only field or in:library")
        );
        assert_eq!(
            row(&vm, "library").availability,
            FieldListRowAvailability::Available
        );
        let vm = build_end("in:index is:ready ", FieldListOpening::Open).expect("a notice");
        let kinds: Vec<_> = vm.notices.iter().map(|notice| notice.kind).collect();
        assert_eq!(kinds, vec![FieldListNoticeKind::ScopeConflict]);
        assert_eq!(
            vm.notices[0].text,
            "This query asks for the Library only and for Music Index only. It shows no rows."
        );
    }

    #[test]
    fn adr_0087_field_list_r87_4_08_up_and_down_wrap_and_skip_unavailable_rows() {
        let vm = build_end("kind:", FieldListOpening::Open).expect("the list shows");
        assert_eq!(vm.rows.len(), 3);
        let at = |index| with_selected(vm.clone(), index);
        assert_eq!(
            at(None).key(FieldListKey::Down),
            FieldListKeyOutcome::Select(0)
        );
        assert_eq!(
            at(Some(2)).key(FieldListKey::Down),
            FieldListKeyOutcome::Select(0)
        );
        assert_eq!(
            at(None).key(FieldListKey::Up),
            FieldListKeyOutcome::Select(2)
        );
        assert_eq!(
            at(Some(0)).key(FieldListKey::Up),
            FieldListKeyOutcome::Select(2)
        );

        // In `in:index `, the rows 9 to 11 (`is:`, `playlist:`, `has:`)
        // are not available.
        let vm = build_at("in:index ", 9, FieldListOpening::Open).expect("the list shows");
        let at = |index| with_selected(vm.clone(), index);
        assert_eq!(
            at(Some(8)).key(FieldListKey::Down),
            FieldListKeyOutcome::Select(0)
        );
        assert_eq!(
            at(Some(0)).key(FieldListKey::Up),
            FieldListKeyOutcome::Select(8)
        );
        assert_eq!(
            at(None).key(FieldListKey::Up),
            FieldListKeyOutcome::Select(8)
        );

        let mut vm = build_end("kind:", FieldListOpening::Open).expect("the list shows");
        for row in &mut vm.rows {
            row.availability =
                FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict);
        }
        assert_eq!(vm.key(FieldListKey::Up), FieldListKeyOutcome::Pass);
        assert_eq!(vm.key(FieldListKey::Down), FieldListKeyOutcome::Pass);
    }

    #[test]
    fn adr_0087_field_list_r87_4_09_tab_puts_the_field_token_into_the_text() {
        let vm = with_selected(
            build_at("ar", 2, FieldListOpening::Open).expect("the list shows"),
            Some(0),
        );
        assert_eq!(
            vm.key(FieldListKey::Tab),
            FieldListKeyOutcome::Apply(FieldListEdit {
                range: 0..2,
                text: "artist:".to_owned(),
                cursor: 7,
            })
        );
        assert_eq!(tab_text("ar", 2), ("artist:".to_owned(), 7));
        assert_eq!(
            tab_text("haleen -ar", 10),
            ("haleen -artist:".to_owned(), 15)
        );
        assert_eq!(tab_text("arjsmovie", 2), ("artist:jsmovie".to_owned(), 7));
    }

    #[test]
    fn adr_0087_field_list_r87_4_10_tab_puts_the_value_into_the_text() {
        assert_eq!(tab_text("kind:s", 6), ("kind:single ".to_owned(), 12));
        assert_eq!(tab_text("kind:s,ep", 6), ("kind:single,ep".to_owned(), 11));

        let vm = build_end("kind:", FieldListOpening::Open).expect("the list shows");
        assert_eq!(
            vm.key(FieldListKey::Tab),
            FieldListKeyOutcome::Apply(vm.rows[0].edit.clone())
        );
        let mut vm = vm;
        vm.rows[0].availability =
            FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict);
        assert_eq!(
            vm.key(FieldListKey::Tab),
            FieldListKeyOutcome::Apply(vm.rows[1].edit.clone())
        );
        for row in &mut vm.rows {
            row.availability =
                FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict);
        }
        assert_eq!(vm.key(FieldListKey::Tab), FieldListKeyOutcome::Pass);
    }

    #[test]
    fn adr_0087_field_list_r87_4_11_enter_applies_only_a_selected_row() {
        let vm = build_end("kind:", FieldListOpening::Open).expect("the list shows");
        let selected = with_selected(vm.clone(), Some(2));
        assert_eq!(
            selected.key(FieldListKey::Enter),
            selected.key(FieldListKey::Tab)
        );
        assert_eq!(
            selected.key(FieldListKey::Enter),
            FieldListKeyOutcome::Apply(vm.rows[2].edit.clone())
        );
        assert_eq!(vm.key(FieldListKey::Enter), FieldListKeyOutcome::Pass);
    }

    #[test]
    fn adr_0087_field_list_r87_4_12_escape_closes_and_change_or_click_opens() {
        let vm = build_end("", FieldListOpening::Open).expect("the list shows");
        assert_eq!(vm.key(FieldListKey::Escape), FieldListKeyOutcome::Close);

        let mut state = state(FieldListOpening::Open);
        state.apply(FieldListEvent::Escape);
        assert_eq!(
            FieldListVm::build("ar", 2..2, &state, &FieldValueRows::EMPTY),
            None
        );
        state.apply(FieldListEvent::Change);
        assert!(FieldListVm::build("ar", 2..2, &state, &FieldValueRows::EMPTY).is_some());
        state.apply(FieldListEvent::Escape);
        state.apply(FieldListEvent::Click);
        assert_eq!(state.opening, FieldListOpening::Open);
        assert!(FieldListVm::build("ar", 2..2, &state, &FieldValueRows::EMPTY).is_some());
    }

    #[test]
    fn adr_0087_field_list_r87_4_13_events_change_the_state() {
        let mut state = state(FieldListOpening::Open);
        state.apply(FieldListEvent::Blur);
        assert_eq!(state.opening, FieldListOpening::Closed);
        assert!(!state.focused);
        state.apply(FieldListEvent::Change);
        assert_eq!(state.opening, FieldListOpening::Closed);

        let mut open = FieldListState {
            opening: FieldListOpening::Open,
            focused: true,
            selected: None,
            visit: 0,
        };
        open.apply(FieldListEvent::Submit);
        assert_eq!(open.opening, FieldListOpening::Closed);

        state.apply(FieldListEvent::Focus);
        assert_eq!(state.opening, FieldListOpening::Closed);
        assert!(state.focused);
        state.apply(FieldListEvent::Change);
        assert_eq!(state.opening, FieldListOpening::Open);
        state.apply(FieldListEvent::OutsideClick);
        assert_eq!(state.opening, FieldListOpening::Closed);
        assert!(state.focused);

        for event in [FieldListEvent::Change, FieldListEvent::CursorMoved] {
            state.select(Some(1));
            state.apply(event);
            assert_eq!(state.selected, None, "{event:?}");
        }
        state.select(Some(1));
        let opening = state.opening;
        state.apply(FieldListEvent::CursorMoved);
        assert_eq!(state.opening, opening);
    }

    #[test]
    fn adr_0087_field_list_r87_4_15_labels_and_actions() {
        let vm = build_end("", FieldListOpening::Open).expect("the list shows");
        assert_eq!(
            row(&vm, "artist:").a11y_label,
            "Field artist:, an artist name, placeholder artist:\"artist name\""
        );
        assert!(row(&vm, "is:").a11y_label.ends_with(", Library only"));
        for row in &vm.rows {
            let FieldListRowKind::Field { field } = &row.kind else {
                panic!("each row is a field row");
            };
            assert_eq!(row.action, FieldListRowAction::InsertField(*field));
            assert_eq!(row.availability, FieldListRowAvailability::Available);
        }

        let vm = build_end("kind:", FieldListOpening::Open).expect("the list shows");
        assert_eq!(
            row(&vm, "single").a11y_label,
            "Value single for kind:, A release with one track"
        );
        for row in &vm.rows {
            let FieldListRowKind::FixedValue { field, word } = &row.kind else {
                panic!("each row is a value row");
            };
            assert_eq!(
                row.action,
                FieldListRowAction::InsertValue {
                    field: *field,
                    word,
                }
            );
        }

        let vm = build_at("in:index ", 9, FieldListOpening::Open).expect("the list shows");
        assert_eq!(
            row(&vm, "is:").a11y_label,
            "Field is:, downloaded, partial, ready, removed, for example is:downloaded, Library only, \
             Not available with in:index"
        );
    }

    #[test]
    fn adr_0087_field_list_r87_4_16_edits_keep_character_boundaries() {
        let text = "café ar";
        let vm = with_selected(
            build_at(text, 8, FieldListOpening::Open).expect("the list shows"),
            Some(0),
        );
        let FieldListKeyOutcome::Apply(edit) = vm.key(FieldListKey::Tab) else {
            panic!("Tab applies the row");
        };
        assert_eq!(edit.range, 6..8);
        let changed = edit.apply(text);
        assert_eq!(changed, "café artist:");
        assert_eq!(edit.cursor, 13);

        for (text, opening) in [
            ("café ar", FieldListOpening::Open),
            ("é kind:s", FieldListOpening::Open),
            ("naïve ", FieldListOpening::Open),
            ("ü in:", FieldListOpening::Open),
        ] {
            let vm = build_end(text, opening).expect("the list shows");
            for row in &vm.rows {
                assert!(text.is_char_boundary(row.edit.range.start), "{text:?}");
                assert!(text.is_char_boundary(row.edit.range.end), "{text:?}");
                let changed = row.edit.apply(text);
                assert!(changed.is_char_boundary(row.edit.cursor), "{text:?}");
            }
        }
    }

    /// Asserts that the list for `text` at its end shows one `Field` row
    /// for each row of `QUERY_FIELDS`, in that order.
    fn assert_each_field(text: &str) -> FieldListVm {
        let vm = build_end(text, FieldListOpening::Open).expect("the list shows");
        assert_eq!(vm.rows.len(), QUERY_FIELDS.len(), "{text:?}");
        for (row, info) in vm.rows.iter().zip(QUERY_FIELDS.iter()) {
            assert_eq!(
                row.kind,
                FieldListRowKind::Field { field: info.name },
                "{text:?}"
            );
        }
        vm
    }

    /// R87-8-01 (ADR 0087 packet 008): after a plain word and a space, the
    /// list shows each field. A selection still gives no list.
    #[test]
    fn adr_0087_field_list_r87_8_01_each_field_after_a_word_and_a_space() {
        let vm = assert_each_field("haleen ");
        for row in &vm.rows {
            assert_eq!(row.edit.range, 7..7, "row {}", row.label);
        }
        for text in ["", "ar", "kind:", "haleen "] {
            assert_eq!(
                FieldListVm::build(
                    text,
                    0..2,
                    &state(FieldListOpening::Open),
                    &FieldValueRows::EMPTY
                ),
                None,
                "{text:?}"
            );
        }
    }

    /// R87-8-02 (ADR 0087 packet 008): after `in:index ` typed by hand, the
    /// list shows each field, and Down skips the three Library-only rows.
    #[test]
    fn adr_0087_field_list_r87_8_02_each_field_after_in_index_and_a_space() {
        let vm = build_at("in:index ", 9, FieldListOpening::Open).expect("the list shows");
        assert_eq!(vm.rows.len(), 12);
        for label in ["is:", "playlist:", "has:"] {
            assert_eq!(
                row(&vm, label).availability,
                FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict)
            );
            assert_eq!(
                row(&vm, label).note.as_deref(),
                Some("Not available with in:index")
            );
        }
        assert_eq!(vm.key(FieldListKey::Down), FieldListKeyOutcome::Select(0));
        let mut selected = None;
        for _ in 0..vm.rows.len() * 2 {
            let FieldListKeyOutcome::Select(next) =
                with_selected(vm.clone(), selected).key(FieldListKey::Down)
            else {
                panic!("Down selects a row");
            };
            assert!(vm.rows[next].availability.is_available(), "row {next}");
            selected = Some(next);
        }
    }

    /// R87-8-03 (ADR 0087 packet 008): after a field term and a space, and
    /// after a value row put a value and a space into the text, the list
    /// shows each field.
    #[test]
    fn adr_0087_field_list_r87_8_03_each_field_after_a_field_term() {
        let vm = assert_each_field("kind:single ");
        assert_each_field("artist:\"Haleen Moss\" ");

        let release = vm
            .rows
            .iter()
            .position(|row| row.label == "release:")
            .expect("the release: row");
        let FieldListKeyOutcome::Apply(edit) =
            with_selected(vm, Some(release)).key(FieldListKey::Tab)
        else {
            panic!("Tab applies the selected row");
        };
        assert_eq!(edit.apply("kind:single "), "kind:single release:");
        assert_eq!(edit.cursor, 20);
    }

    /// R87-8-04 (ADR 0087 packet 008): `Inserted` keeps the list open in
    /// each order with `Change`. Escape, the search and a blur close it.
    #[test]
    fn adr_0087_field_list_r87_8_04_inserted_keeps_the_list_open() {
        let open = FieldListState {
            selected: Some(1),
            ..state(FieldListOpening::Open)
        };

        let mut inserted = open;
        inserted.apply(FieldListEvent::Inserted);
        assert_eq!(inserted.opening, FieldListOpening::Open);
        assert_eq!(inserted.selected, None);

        for events in [
            [FieldListEvent::Inserted, FieldListEvent::Change],
            [FieldListEvent::Change, FieldListEvent::Inserted],
        ] {
            let mut state = open;
            for event in events {
                state.apply(event);
            }
            assert_eq!(state.opening, FieldListOpening::Open, "{events:?}");
        }

        for event in [
            FieldListEvent::Escape,
            FieldListEvent::Submit,
            FieldListEvent::OutsideClick,
            FieldListEvent::Blur,
        ] {
            let mut state = open;
            state.apply(event);
            assert_eq!(state.opening, FieldListOpening::Closed, "{event:?}");
        }
    }

    fn library_h() -> LibraryFieldValues {
        LibraryFieldValues::from_texts(
            &["Haleen", "Haleen Moss", "Shaleen", "jSmovie"],
            &["Aged Friends"],
            &["Friday show", "Monday mix"],
        )
    }

    fn build_values(
        text: &str,
        cursor: usize,
        library: &LibraryFieldValues,
        index: &IndexArtistValues,
    ) -> Option<FieldListVm> {
        FieldListVm::build(
            text,
            cursor..cursor,
            &state(FieldListOpening::Open),
            &FieldValueRows { library, index },
        )
    }

    fn loaded(key: &str, names: &[&str]) -> IndexArtistValues {
        IndexArtistValues::Loaded {
            key: key.to_owned(),
            names: names.iter().map(|&name| name.to_owned()).collect(),
        }
    }

    fn sources(vm: &FieldListVm) -> Vec<Option<FieldValueSource>> {
        vm.rows
            .iter()
            .map(|row| match &row.kind {
                FieldListRowKind::Value { source, .. } => Some(*source),
                _ => None,
            })
            .collect()
    }

    /// R87-5-03: the Library rows come first, then the `MusicIndex` rows,
    /// each with its source mark in words.
    #[test]
    fn adr_0087_field_values_r87_5_03_library_rows_then_index_rows() {
        let library = library_h();
        let index = loaded("hal", &["Halo Child"]);
        let vm = build_values("artist:hal", 10, &library, &index).expect("the list shows");
        assert_eq!(
            labels(&vm),
            vec!["Haleen", "Haleen Moss", "Shaleen", "Halo Child"]
        );
        assert_eq!(
            sources(&vm),
            vec![
                Some(FieldValueSource::Library),
                Some(FieldValueSource::Library),
                Some(FieldValueSource::Library),
                Some(FieldValueSource::Index),
            ]
        );
        let marks: Vec<&str> = vm
            .rows
            .iter()
            .map(|row| row.mark.expect("each value row has a mark").label())
            .collect();
        assert_eq!(marks, vec!["Library", "Library", "Library", "Music Index"]);
        assert_eq!(
            vm.rows[0].element_id,
            "search-field-list-value-artist-library-0"
        );
        assert_eq!(
            vm.rows[3].element_id,
            "search-field-list-value-artist-index-0"
        );
        assert_eq!(
            vm.rows[3].a11y_label,
            "Value Halo Child for artist:, from Music Index"
        );
        assert_eq!(
            vm.rows[0].a11y_label,
            "Value Haleen for artist:, from the Library"
        );
    }

    /// R87-5-04: a name in the Library and in `MusicIndex` gives one row,
    /// with the source `Library`.
    #[test]
    fn adr_0087_field_values_r87_5_04_shared_name_gives_one_library_row() {
        let library = library_h();
        let index = loaded("hal", &["haleen", "Halo Child"]);
        let vm = build_values("artist:hal", 10, &library, &index).expect("the list shows");
        let haleen: Vec<&FieldListRowVm> = vm
            .rows
            .iter()
            .filter(|row| name_sort_key(&row.label) == "haleen")
            .collect();
        assert_eq!(haleen.len(), 1);
        assert_eq!(
            haleen[0].mark,
            Some(FieldListMark::Source(FieldValueSource::Library))
        );
        assert_eq!(labels(&vm).last(), Some(&"Halo Child"));
    }

    /// R87-5-06: `release:` and `playlist:` show their Library values and
    /// send no request.
    #[test]
    fn adr_0087_field_values_r87_5_06_release_and_playlist_values() {
        let library = library_h();
        let none = IndexArtistValues::NotRequested;
        let vm = build_values("release:ag", 10, &library, &none).expect("the list shows");
        assert_eq!(labels(&vm), vec!["Aged Friends"]);
        assert_eq!(sources(&vm), vec![Some(FieldValueSource::Library)]);
        let vm = build_values("playlist:fri", 12, &library, &none).expect("the list shows");
        assert_eq!(labels(&vm), vec!["Friday show"]);
        assert_eq!(artist_value_request_key("release:ag", 10), None);
        assert_eq!(artist_value_request_key("playlist:fri", 12), None);
    }

    /// R87-5-10: Tab puts the quoted value and one space into the text. A
    /// value before more text gets no new space.
    #[test]
    fn adr_0087_field_values_r87_5_10_tab_puts_the_quoted_value_into_the_text() {
        let library = library_h();
        let none = IndexArtistValues::NotRequested;
        let vm = build_values("artist:hal", 10, &library, &none).expect("the list shows");
        let vm = with_selected(vm, Some(1));
        assert_eq!(
            vm.rows[1].action,
            FieldListRowAction::InsertText {
                field: QueryFieldName::Artist,
                text: "Haleen Moss".to_owned(),
            }
        );
        let FieldListKeyOutcome::Apply(edit) = vm.key(FieldListKey::Tab) else {
            panic!("Tab applies the row");
        };
        let changed = edit.apply("artist:hal");
        assert_eq!(changed, "artist:\"Haleen Moss\" ");
        assert_eq!(edit.cursor, changed.len());

        let text = "artist:hal kind:single";
        let vm = build_values(text, 10, &library, &none).expect("the list shows");
        let vm = with_selected(vm, Some(0));
        let FieldListKeyOutcome::Apply(edit) = vm.key(FieldListKey::Tab) else {
            panic!("Tab applies the row");
        };
        assert_eq!(edit.apply(text), "artist:Haleen kind:single");
        assert_eq!(edit.cursor, 13);

        // A quoted value is one item. The edit replaces the quotes too.
        let text = "artist:\"Haleen M";
        let vm = build_values(text, text.len(), &library, &none).expect("the list shows");
        assert_eq!(labels(&vm), vec!["Haleen Moss"]);
        let edit = &vm.rows[0].edit;
        assert_eq!(edit.apply(text), "artist:\"Haleen Moss\" ");
    }

    /// R87-5-11: the loading row follows the Library rows. It is not
    /// available, so Down skips it.
    #[test]
    fn adr_0087_field_values_r87_5_11_loading_row() {
        let library = library_h();
        let index = IndexArtistValues::Loading {
            key: "hal".to_owned(),
        };
        let vm = build_values("artist:hal", 10, &library, &index).expect("the list shows");
        assert_eq!(vm.rows.len(), 4);
        let row = &vm.rows[3];
        assert_eq!(
            row.kind,
            FieldListRowKind::Loading {
                field: QueryFieldName::Artist
            }
        );
        assert_eq!(
            row.availability,
            FieldListRowAvailability::Unavailable(FieldListRowUnavailable::Loading)
        );
        assert_eq!(row.action, FieldListRowAction::Inert);
        assert_eq!(row.label, "Loading names from Music Index");
        assert_eq!(row.element_id, "search-field-list-loading-artist");
        assert_eq!(row.edit.range, 10..10);
        assert!(row.edit.text.is_empty());
        let vm = with_selected(vm, Some(2));
        assert_eq!(vm.key(FieldListKey::Down), FieldListKeyOutcome::Select(0));
    }

    /// R87-5-12: a failed request gives one error row, and the Library rows
    /// stay.
    #[test]
    fn adr_0087_field_values_r87_5_12_error_row() {
        let library = library_h();
        let index = IndexArtistValues::Failed {
            key: "hal".to_owned(),
        };
        let vm = build_values("artist:hal", 10, &library, &index).expect("the list shows");
        assert_eq!(vm.rows.len(), 4);
        let row = &vm.rows[3];
        assert_eq!(
            row.kind,
            FieldListRowKind::IndexError {
                field: QueryFieldName::Artist
            }
        );
        assert_eq!(row.label, "Names from Music Index did not load");
        assert_eq!(
            row.note.as_deref(),
            Some("The Library names show. The next change of the text tries again.")
        );
        assert_eq!(
            row.availability,
            FieldListRowAvailability::Unavailable(FieldListRowUnavailable::IndexError)
        );
        assert_eq!(row.action, FieldListRowAction::Inert);
    }

    /// R87-5-13: a state of another key gives no `MusicIndex` row. An empty
    /// value gives the Library rows only.
    #[test]
    fn adr_0087_field_values_r87_5_13_state_of_another_key_gives_no_row() {
        let library = library_h();
        let index = loaded("hal", &["Halo Child"]);
        let vm = build_values("artist:halo", 11, &library, &index);
        assert!(vm.is_none_or(|vm| sources(&vm)
            .iter()
            .all(|source| *source == Some(FieldValueSource::Library))));
        let loading = IndexArtistValues::Loading {
            key: "hal".to_owned(),
        };
        let vm = build_values("artist:", 7, &library, &loading).expect("the list shows");
        assert_eq!(vm.rows.len(), 4);
        assert_eq!(sources(&vm), vec![Some(FieldValueSource::Library); 4]);
    }

    /// R87-5-15: with `in:index`, a `playlist:` value is not available.
    #[test]
    fn adr_0087_field_values_r87_5_15_playlist_value_conflicts_with_in_index() {
        let library = library_h();
        let none = IndexArtistValues::NotRequested;
        let text = "in:index playlist:fri";
        let vm = build_values(text, text.len(), &library, &none).expect("the list shows");
        let row = row(&vm, "Friday show");
        assert_eq!(
            row.availability,
            FieldListRowAvailability::Unavailable(FieldListRowUnavailable::ScopeConflict)
        );
        assert_eq!(row.note.as_deref(), Some("Not available with in:index"));
    }

    /// Packet 005 orchestrator decision: the fixed values read a quoted
    /// prefix, and a quoted `kind:` value is one item.
    #[test]
    fn adr_0087_field_values_quoted_prefix_matches_a_fixed_value() {
        let vm = build_end("kind:\"si", FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["single"]);
        assert_eq!(vm.rows[0].edit.range, 5..8);
        assert_eq!(tab_text("kind:\"si", 8), ("kind:single ".to_owned(), 12));
    }

    /// ADR 0091, crate pin 634e4cc: `ki` shows the field rows of the crate.
    #[test]
    fn adr_0091_pin_634e4cc_field_rows_for_a_name_prefix() {
        let vm = build_end("ki", FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["kind:"]);
        assert_eq!(vm.rows[0].edit.range, 0..2);
        assert_eq!(vm.rows[0].edit.text, "kind:");
    }

    /// ADR 0091, crate pin 634e4cc: `kind:s` shows the fixed value rows of the
    /// crate. The edit replaces the typed value.
    #[test]
    fn adr_0091_pin_634e4cc_value_rows_for_a_value_prefix() {
        let vm = build_end("kind:s", FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["single"]);
        assert_eq!(vm.rows[0].edit.range, 5..6);
        assert_eq!(vm.rows[0].edit.text, "single ");
    }

    /// ADR 0091, crate pin 634e4cc: the Library is present, so `in:` shows
    /// its two values and `is:` shows its rows with the mark "Library only".
    #[test]
    fn adr_0091_pin_634e4cc_library_present_shows_the_library_rows() {
        let vm = build_end("in:", FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["library", "index"]);
        let vm = build_end("i", FieldListOpening::Open).expect("the list shows");
        assert_eq!(labels(&vm), vec!["in:", "is:"]);
        assert_eq!(row(&vm, "is:").mark, Some(FieldListMark::LibraryOnly));
    }

    /// ADR 0091, crate pin 634e4cc: the name request of `playlist:` has the
    /// typed prefix with its quote read. The Library names match that prefix.
    #[test]
    fn adr_0091_pin_634e4cc_name_request_gives_the_library_names() {
        use crate::view_models::search_field_values::LibraryFieldValues;

        let library =
            LibraryFieldValues::from_texts(&[], &[], &["Friday Mix", "Night Drive", "Friends"]);
        let rows = FieldValueRows {
            library: &library,
            index: &IndexArtistValues::NotRequested,
        };
        for text in ["playlist:fri", "playlist:\"fri"] {
            let vm = FieldListVm::build(
                text,
                text.len()..text.len(),
                &state(FieldListOpening::Open),
                &rows,
            )
            .expect("the list shows");
            assert_eq!(labels(&vm), vec!["Friday Mix", "Friends"], "{text}");
            assert_eq!(
                vm.rows[0].mark,
                Some(FieldListMark::Source(FieldValueSource::Library))
            );
        }
    }

    /// ADR 0091, crate pin 634e4cc: the notice of the field error has the
    /// message of the crate.
    #[test]
    fn adr_0091_pin_634e4cc_field_error_gives_the_crate_message() {
        let text = "after:yesterday";
        let vm = build_end(text, FieldListOpening::Open).expect("the list shows");
        let expected = field_list_rows(text, text.len(), Library::Present)
            .error
            .expect("the crate gives the error");
        assert_eq!(vm.notices.len(), 1);
        assert_eq!(vm.notices[0].kind, FieldListNoticeKind::FieldError);
        assert_eq!(vm.notices[0].text, expected);
        assert!(expected.starts_with("after:"), "{expected}");
    }
}

//! The Tiles view of the Artists list (musicindex ADR 0053 R-LIST-10).
//!
//! Each Artists row of the scope has one tile. The tile has the art of the
//! row, its name as the first line, and its type word and library text as
//! the second line. This module is pure: the renderer only composes the
//! tile parts.

#![warn(clippy::pedantic)]

use crate::view_models::collection_art::CollectionArtVm;
use crate::view_models::library::{
    ArtistsListItemVm, DestinationListRowTarget, DestinationListRowVm,
};

/// The word between the type word and the library text of a tile.
const SECOND_LINE_SEPARATOR: &str = " \u{b7} ";

/// One tile of the Artists list (musicindex ADR 0053 R-LIST-10). A name
/// with no feed has the logo placeholder as its art, the same as its row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtistsTileVm {
    /// Stable element id. It is the element id of the row.
    pub(crate) element_id: String,
    /// The first line: the name of the artist.
    pub(crate) label: String,
    /// The second line: the type word and the library text. It is empty
    /// when the row has neither.
    pub(crate) second_line: String,
    /// The accessibility label of the tile. It is the label of the row.
    pub(crate) a11y_label: String,
    /// The art of the row (ADR 0084 Decision 12).
    pub(crate) art: Option<CollectionArtVm>,
    /// The entry that a click opens.
    pub(crate) open: DestinationListRowTarget,
    /// `true` when the tile opens the page that Back left open (R-NAV-2
    /// item 2).
    pub(crate) marked: bool,
}

impl ArtistsTileVm {
    /// Gives the tile of one Artists `row`.
    #[must_use]
    pub(crate) fn of(row: &DestinationListRowVm) -> Self {
        let mut parts: Vec<&str> = Vec::new();
        let word = row.type_tag().map(|tag| tag.word);
        if let Some(word) = word.as_deref() {
            parts.push(word);
        }
        if !row.secondary_text.is_empty() {
            parts.push(&row.secondary_text);
        }
        Self {
            element_id: row.element_id.clone(),
            label: row.label.clone(),
            second_line: parts.join(SECOND_LINE_SEPARATOR),
            a11y_label: row.a11y_label.clone(),
            art: row.art.clone(),
            open: row.open.clone(),
            marked: row.marked,
        }
    }
}

impl ArtistsListItemVm {
    /// The tile of this item, in the same sequence as the rows. A status row
    /// has none, and it keeps its row in the Tiles view.
    #[must_use]
    pub(crate) fn tile(&self) -> Option<ArtistsTileVm> {
        match self {
            Self::Row(row) => Some(ArtistsTileVm::of(row)),
            Self::Status(_) => None,
        }
    }
}

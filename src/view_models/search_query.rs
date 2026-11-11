//! The Music search query text of v4vmm (ADR 0087 Decisions 1 and 2, ADR 0091).
//!
//! The parser of the query text is the crate `musicindex-search`, module
//! `query` (musicindex ADR 0049 R-QUERY-1). This module re-exports its names,
//! so the other files of v4vmm do not change. The module keeps the two scope
//! rules of ADR 0089 Decision 4. They read and write the Library scope of a
//! query text, and each range is a byte range on a character boundary.
//!
//! The module has no GPUI type and no I/O.

#![warn(clippy::pedantic)]

use std::ops::Range;

#[cfg(test)]
use chrono::NaiveDate;
pub use musicindex_search::query::{
    cursor_term, fields_with_prefix, quote_field_value, unquote_field_value_prefix, ContentValue,
    CursorPartial, CursorTerm, DateValue, DurationRange, FieldTerm, FieldValue, FieldValueError,
    FieldValueKind, FieldValueState, FixedValueInfo, KindValue, LibraryStateValue, QueryFieldInfo,
    QueryFieldName, QueryScope, QueryTerm, ScopeValue, SearchQuery, TermKind, QUERY_FIELDS,
};

use crate::view_models::list_page::ListScope;

/// The spans of the correct `in:library` terms, without `-` (ADR 0089
/// Decision 4).
fn library_scope_spans(query: &SearchQuery) -> Vec<Range<usize>> {
    query
        .terms()
        .iter()
        .filter(|term| !term.excluded)
        .filter(|term| {
            matches!(
                &term.kind,
                TermKind::Field(FieldTerm {
                    name: QueryFieldName::In,
                    value: FieldValueState::Valid(FieldValue::Scope(ScopeValue::Library)),
                })
            )
        })
        .map(|term| term.span.clone())
        .collect()
}

/// Writes the scope of the "All | In Library" control into the query `text`
/// (ADR 0089 Decision 4).
///
/// `ListScope::InLibrary` adds `in:library` at the end. A text with the term
/// does not change. `ListScope::All` removes each correct `in:library` term.
/// Each other term keeps its text. A removed term takes the white space
/// before it, or the white space after it when no text is before it. A
/// `in:index` term does not change, so the conflict message of
/// [`QueryScope::Conflict`] shows when both terms are present.
#[must_use]
pub(crate) fn with_library_scope(text: &str, scope: ListScope) -> String {
    let query = SearchQuery::parse(text);
    let spans = library_scope_spans(&query);
    match scope {
        ListScope::InLibrary => {
            if !spans.is_empty() {
                text.to_owned()
            } else if text.is_empty() || text.ends_with(char::is_whitespace) {
                format!("{text}in:library")
            } else {
                format!("{text} in:library")
            }
        }
        ListScope::All => remove_spans(text, &spans),
    }
}

/// Removes each span of `spans` from `text`. The spans are in text order and
/// do not overlap. A removed span takes the white space before it, when the
/// text has white space before it. Else it takes the white space after it.
fn remove_spans(text: &str, spans: &[Range<usize>]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pos = 0;
    for span in spans {
        out.push_str(&text[pos..span.start]);
        pos = span.end;
        if out.ends_with(char::is_whitespace) {
            let kept = out.trim_end().len();
            out.truncate(kept);
        } else {
            let rest = &text[pos..];
            pos += rest.len() - rest.trim_start().len();
        }
    }
    out.push_str(&text[pos..]);
    out
}

/// The scope that the "All | In Library" control shows for the query `text`
/// (ADR 0089 Decision 4). The control shows "In Library" when the text has
/// an `in:library` term, or when a field of the Library only is in the text
/// (`QueryScope::LibraryOnly`). Each other text shows "All".
#[must_use]
pub(crate) fn library_scope_of(text: &str) -> ListScope {
    let query = SearchQuery::parse(text);
    if !library_scope_spans(&query).is_empty() || query.scope() == QueryScope::LibraryOnly {
        ListScope::InLibrary
    } else {
        ListScope::All
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::release_kind::ReleaseKind;

    const EXAMPLE: &str =
        r#"artist:jsmovie kind:single after:2026-10-01 -title:remix "night drive""#;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("test date is correct")
    }

    /// The one term of `text`.
    fn only_term(text: &str) -> QueryTerm {
        let query = SearchQuery::parse(text);
        assert_eq!(query.terms().len(), 1, "one term in {text:?}");
        query.terms()[0].clone()
    }

    /// The field of the one term of `text`.
    fn only_field(text: &str) -> FieldTerm {
        match only_term(text).kind {
            TermKind::Field(field) => field,
            other => panic!("{text:?} gives {other:?}, not a field"),
        }
    }

    fn valid(text: &str) -> FieldValue {
        match only_field(text).value {
            FieldValueState::Valid(value) => value,
            other => panic!("{text:?} gives {other:?}, not a correct value"),
        }
    }

    fn is_invalid(text: &str) -> bool {
        matches!(only_field(text).value, FieldValueState::Invalid(_))
    }

    fn duration(text: &str) -> DurationRange {
        match valid(text) {
            FieldValue::Duration(range) => range,
            other => panic!("{text:?} gives {other:?}"),
        }
    }

    fn date_value(text: &str) -> DateValue {
        match valid(text) {
            FieldValue::Date(value) => value,
            other => panic!("{text:?} gives {other:?}"),
        }
    }

    fn text_field(name: QueryFieldName, text: &str) -> TermKind {
        TermKind::Field(FieldTerm {
            name,
            value: FieldValueState::Valid(FieldValue::Text(text.to_owned())),
        })
    }

    #[test]
    fn adr_0087_query_parser_r87_1_01_example_gives_five_typed_terms() {
        let query = SearchQuery::parse(EXAMPLE);
        let kinds: Vec<&TermKind> = query.terms().iter().map(|term| &term.kind).collect();
        let day = date(2026, 10, 1);
        assert_eq!(
            kinds,
            vec![
                &text_field(QueryFieldName::Artist, "jsmovie"),
                &TermKind::Field(FieldTerm {
                    name: QueryFieldName::Kind,
                    value: FieldValueState::Valid(FieldValue::Kinds(vec![KindValue::Single])),
                }),
                &TermKind::Field(FieldTerm {
                    name: QueryFieldName::After,
                    value: FieldValueState::Valid(FieldValue::Date(DateValue {
                        first_day: day,
                        last_day: day,
                    })),
                }),
                &text_field(QueryFieldName::Title, "remix"),
                &TermKind::Phrase("night drive".to_owned()),
            ]
        );
        let excluded: Vec<bool> = query.terms().iter().map(|term| term.excluded).collect();
        assert_eq!(excluded, vec![false, false, false, true, false]);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_02_each_span_gives_its_source_text() {
        let query = SearchQuery::parse(EXAMPLE);
        let texts: Vec<&str> = query
            .terms()
            .iter()
            .map(|term| &EXAMPLE[term.span.clone()])
            .collect();
        assert_eq!(
            texts,
            vec![
                "artist:jsmovie",
                "kind:single",
                "after:2026-10-01",
                "-title:remix",
                "\"night drive\"",
            ]
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_03_unknown_colon_name_stays_one_word() {
        assert_eq!(
            only_term("Re:Union").kind,
            TermKind::Word("Re:Union".to_owned())
        );
        assert_eq!(
            only_term("foo:bar").kind,
            TermKind::Word("foo:bar".to_owned())
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_04_date_that_is_not_correct_is_invalid() {
        match only_field("after:yesterday").value {
            FieldValueState::Invalid(error) => assert_eq!(
                error.message(),
                "after: needs a date, for example 2026-10-01"
            ),
            other => panic!("after:yesterday gives {other:?}"),
        }
        assert!(is_invalid("after:2026-02-30"));
        assert!(is_invalid("after:2026-1-5"));
    }

    #[test]
    fn adr_0087_query_parser_r87_1_05_field_name_has_no_case() {
        for text in ["Artist:x", "ARTIST:x", "artist:x"] {
            assert_eq!(only_field(text).name, QueryFieldName::Artist, "{text}");
        }
        assert_eq!(
            valid("KIND:Single"),
            FieldValue::Kinds(vec![KindValue::Single])
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_06_text_value_keeps_case_and_reads_quotes() {
        assert_eq!(valid("title:Remix"), FieldValue::Text("Remix".to_owned()));
        assert_eq!(
            valid(r#"release:"aged friends""#),
            FieldValue::Text("aged friends".to_owned())
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_07_kind_reads_a_comma_list() {
        assert_eq!(
            valid("kind:album,ep"),
            FieldValue::Kinds(vec![KindValue::Album, KindValue::Ep])
        );
        assert_eq!(
            valid("kind:single,single,"),
            FieldValue::Kinds(vec![KindValue::Single])
        );
        match only_field("kind:lp").value {
            FieldValueState::Invalid(error) => assert!(error.message().contains("lp")),
            other => panic!("kind:lp gives {other:?}"),
        }
    }

    #[test]
    fn adr_0087_query_parser_r87_1_08_in_reads_one_scope() {
        assert_eq!(valid("in:library"), FieldValue::Scope(ScopeValue::Library));
        assert_eq!(valid("in:index"), FieldValue::Scope(ScopeValue::Index));
        assert!(is_invalid("in:both"));
        assert!(is_invalid("in:library,index"));
        assert_eq!(
            only_field("-in:index").value,
            FieldValueState::Invalid(FieldValueError::ExcludedScope)
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_09_month_gives_its_first_and_last_day() {
        let october = date_value("after:2026-10");
        assert_eq!(october.first_day, date(2026, 10, 1));
        assert_eq!(october.last_day, date(2026, 10, 31));
        assert_eq!(date_value("before:2026-02").last_day, date(2026, 2, 28));
        assert_eq!(date_value("after:2026-10-01").start_unix(), 1_790_812_800);
        assert_eq!(date_value("before:2026-10-31").end_unix(), 1_793_491_199);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_10_duration_gives_inclusive_limits() {
        let range = |min, max| DurationRange {
            min_secs: min,
            max_secs: max,
        };
        assert_eq!(duration("duration:>5m"), range(Some(301), None));
        assert_eq!(duration("duration:>=5m"), range(Some(300), None));
        assert_eq!(duration("duration:<3:30"), range(None, Some(209)));
        assert_eq!(duration("duration:<=3:30"), range(None, Some(210)));
        assert_eq!(duration("duration:3m..5m"), range(Some(180), Some(300)));
        assert_eq!(duration("duration:>=1h30m").min_secs, Some(5400));
        assert_eq!(duration("duration:>=90s").min_secs, Some(90));
        assert_eq!(duration("duration:>=1:02:03").min_secs, Some(3723));
        let middle = range(Some(180), Some(300));
        assert!(middle.contains(180) && middle.contains(300));
        assert!(!middle.contains(179) && !middle.contains(301));
    }

    #[test]
    fn adr_0087_query_parser_r87_1_11_duration_that_is_not_correct_is_invalid() {
        for text in [
            "duration:5m",
            "duration:5",
            "duration:2.5m",
            "duration:3m..",
            "duration:5m..3m",
            "duration:30s3m",
            "duration:3:5",
            "duration:<0s",
            "duration:>99999999999s",
        ] {
            assert!(is_invalid(text), "{text}");
        }
    }

    #[test]
    fn adr_0087_query_parser_r87_1_12_is_and_has_read_their_fixed_values() {
        assert_eq!(
            valid("is:downloaded"),
            FieldValue::LibraryState(LibraryStateValue::Downloaded)
        );
        assert_eq!(
            valid("is:partial"),
            FieldValue::LibraryState(LibraryStateValue::Partial)
        );
        assert_eq!(
            valid("is:ready"),
            FieldValue::LibraryState(LibraryStateValue::Ready)
        );
        assert_eq!(
            valid("is:removed"),
            FieldValue::LibraryState(LibraryStateValue::Removed)
        );
        assert_eq!(valid("has:split"), FieldValue::Content(ContentValue::Split));
        assert_eq!(valid("has:image"), FieldValue::Content(ContentValue::Image));
        assert!(is_invalid("is:new"));
        assert!(is_invalid("has:art"));
    }

    #[test]
    fn adr_0087_query_parser_r87_1_13_empty_value_is_incomplete() {
        assert_eq!(only_field("artist:").value, FieldValueState::Incomplete);
        assert_eq!(
            only_field(r#"artist:"""#).value,
            FieldValueState::Incomplete
        );
        let query = SearchQuery::parse("artist: jsmovie");
        assert_eq!(
            query.terms()[0].kind,
            TermKind::Field(FieldTerm {
                name: QueryFieldName::Artist,
                value: FieldValueState::Incomplete,
            })
        );
        assert_eq!(query.terms()[1].kind, TermKind::Word("jsmovie".to_owned()));
        assert_eq!(query.terms().len(), 2);
    }

    /// The operator saw an error until `has:image` was complete on 2026-11-06.
    #[test]
    fn adr_0087_start_of_a_fixed_value_is_incomplete_not_an_error() {
        let state = |text: &str| match only_term(text).kind {
            TermKind::Field(field) => field.value,
            other => panic!("not a field: {other:?}"),
        };
        for text in ["has:spl", "has:IM", "is:down", "in:lib", "kind:sin"] {
            assert_eq!(state(text), FieldValueState::Incomplete, "{text}");
        }
        for text in ["has:pickle", "is:nope", "kind:sin,nope", "kind:nope,sin"] {
            assert!(
                matches!(state(text), FieldValueState::Invalid(_)),
                "{text} stays an error"
            );
        }
        assert!(matches!(state("kind:album,sin"), FieldValueState::Valid(_)));
    }

    #[test]
    fn adr_0087_query_parser_r87_1_14_open_quote_closes_at_the_text_end() {
        assert_eq!(
            only_term(r#""night drive"#).kind,
            TermKind::Phrase("night drive".to_owned())
        );
        assert_eq!(
            valid(r#"release:"aged fr"#),
            FieldValue::Text("aged fr".to_owned())
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_15_backslash_escapes_inside_quotes_only() {
        assert_eq!(
            only_term(r#""the \"best\" song""#).kind,
            TermKind::Phrase(r#"the "best" song"#.to_owned())
        );
        assert_eq!(
            only_term(r#""a\\b""#).kind,
            TermKind::Phrase(r"a\b".to_owned())
        );
        assert_eq!(only_term(r"a\b").kind, TermKind::Word(r"a\b".to_owned()));
    }

    #[test]
    fn adr_0087_query_parser_r87_1_16_minus_applies_at_the_term_start_only() {
        assert_eq!(only_term("lo-fi").kind, TermKind::Word("lo-fi".to_owned()));
        assert!(SearchQuery::parse("-").terms().is_empty());
        let remix = only_term("-remix");
        assert_eq!(remix.kind, TermKind::Word("remix".to_owned()));
        assert!(remix.excluded);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_17_scope_reads_the_correct_fields() {
        let scope = |text: &str| SearchQuery::parse(text).scope();
        assert_eq!(scope("haleen"), QueryScope::Everything);
        assert_eq!(scope("is:ready"), QueryScope::LibraryOnly);
        assert_eq!(scope("-has:split"), QueryScope::LibraryOnly);
        assert_eq!(scope("in:library"), QueryScope::LibraryOnly);
        assert_eq!(scope("in:index"), QueryScope::IndexOnly);
        assert_eq!(scope("in:index is:ready"), QueryScope::Conflict);
        assert_eq!(scope("in:library in:index"), QueryScope::Conflict);
        assert_eq!(scope("is:bogus in:index"), QueryScope::IndexOnly);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_18_index_q_holds_words_and_phrases_only() {
        let index_q = |text: &str| SearchQuery::parse(text).index_q();
        assert_eq!(index_q(EXAMPLE), Some("\"night drive\"".to_owned()));
        // musicindex ADR 0049 R-QUERY-5: a quote and a backslash each give a space.
        assert_eq!(
            index_q(r#"haleen "a \"b\"""#),
            Some(r#"haleen "a  b ""#.to_owned())
        );
        assert_eq!(index_q("artist:x -remix"), None);
        for text in [
            EXAMPLE,
            "haleen kind:album title:x",
            "is:ready in:index night",
        ] {
            let q = index_q(text).unwrap_or_default();
            for info in &QUERY_FIELDS {
                assert!(!q.contains(info.token), "{q:?} holds {}", info.token);
            }
        }
    }

    #[test]
    fn adr_0087_query_parser_r87_1_20_runnable_content_needs_a_word_or_a_field() {
        let runnable = |text: &str| SearchQuery::parse(text).has_runnable_content();
        for text in ["haleen", "\"night drive\"", "kind:single"] {
            assert!(runnable(text), "{text}");
        }
        for text in [
            "",
            "!!!",
            "-remix",
            "in:library",
            "artist:",
            "after:yesterday",
            "-kind:single",
        ] {
            assert!(!runnable(text), "{text}");
        }
    }

    #[test]
    fn adr_0087_query_parser_r87_1_21_cursor_in_a_field_name_gives_the_prefix() {
        assert_eq!(
            cursor_term("ar", 2),
            CursorTerm {
                range: 0..2,
                excluded: false,
                partial: CursorPartial::FieldName {
                    prefix: "ar".to_owned(),
                },
            }
        );
        let tokens: Vec<&str> = fields_with_prefix("ar")
            .iter()
            .map(|info| info.token)
            .collect();
        assert_eq!(tokens, vec!["artist:"]);
        assert_eq!(fields_with_prefix("").len(), 12);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_22_cursor_after_a_colon_gives_the_value() {
        assert_eq!(
            cursor_term("kind:s", 6).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Kind,
                prefix: "s".to_owned(),
                value_range: 5..6,
            }
        );
        let words: Vec<&str> = QueryFieldName::Kind
            .values_with_prefix("s")
            .iter()
            .map(|value| value.word)
            .collect();
        assert_eq!(words, vec!["single"]);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_23_kind_item_and_excluded_term() {
        assert_eq!(
            cursor_term("kind:album,e", 12).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Kind,
                prefix: "e".to_owned(),
                value_range: 11..12,
            }
        );
        let term = cursor_term("haleen -ar", 10);
        assert!(term.excluded);
        assert_eq!(term.range, 8..10);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_24_empty_term_and_other_terms() {
        assert_eq!(
            cursor_term("haleen ", 7),
            CursorTerm {
                range: 7..7,
                excluded: false,
                partial: CursorPartial::FieldName {
                    prefix: String::new(),
                },
            }
        );
        assert_eq!(cursor_term("\"night dr", 9).partial, CursorPartial::Other);
        assert_eq!(cursor_term("Re:Un", 5).partial, CursorPartial::Other);
    }

    #[test]
    fn adr_0087_query_parser_r87_1_25_cursor_moves_to_a_character_boundary() {
        let text = "artist:é";
        assert_eq!(text.len(), 9);
        for cursor in [8, 9, 100] {
            let term = cursor_term(text, cursor);
            assert!(text.is_char_boundary(term.range.start));
            assert!(text.is_char_boundary(term.range.end));
            if let CursorPartial::FieldValue {
                prefix,
                value_range,
                ..
            } = &term.partial
            {
                assert!(text.is_char_boundary(value_range.start));
                assert!(text.is_char_boundary(value_range.end));
                assert!(prefix.is_empty() || prefix == "é");
            }
        }
        assert_eq!(
            cursor_term(text, 9).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Artist,
                prefix: "é".to_owned(),
                value_range: 7..9,
            }
        );
        assert_eq!(
            cursor_term(text, 8).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Artist,
                prefix: String::new(),
                value_range: 7..9,
            }
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_26_field_table_order_tokens_and_scope() {
        let names: Vec<QueryFieldName> = QUERY_FIELDS.iter().map(|info| info.name).collect();
        assert_eq!(
            names,
            vec![
                QueryFieldName::Artist,
                QueryFieldName::Title,
                QueryFieldName::Release,
                QueryFieldName::Kind,
                QueryFieldName::In,
                QueryFieldName::After,
                QueryFieldName::Before,
                QueryFieldName::Duration,
                QueryFieldName::Host,
                QueryFieldName::Is,
                QueryFieldName::Playlist,
                QueryFieldName::Has,
            ]
        );
        for info in &QUERY_FIELDS {
            assert_eq!(
                info.token,
                format!("{:?}:", info.name).to_lowercase(),
                "{info:?}"
            );
            let bare = info.token.trim_end_matches(':');
            assert_eq!(QueryFieldName::from_name(bare), Some(info.name));
            assert_eq!(info.name.info(), info);
        }
        let library_only: Vec<QueryFieldName> = QUERY_FIELDS
            .iter()
            .filter(|info| info.library_only)
            .map(|info| info.name)
            .collect();
        assert_eq!(
            library_only,
            vec![
                QueryFieldName::Is,
                QueryFieldName::Playlist,
                QueryFieldName::Has
            ]
        );
    }

    #[test]
    fn adr_0087_query_parser_r87_1_27_fixed_values_parse_as_correct_values() {
        for info in &QUERY_FIELDS {
            let expected = match info.name {
                QueryFieldName::Kind => 3,
                QueryFieldName::In | QueryFieldName::Has => 2,
                QueryFieldName::Is => 4,
                _ => 0,
            };
            let values = info.name.fixed_values();
            assert_eq!(values.len(), expected, "{}", info.token);
            for value in values {
                let text = format!("{}{}", info.token, value.word);
                assert!(
                    matches!(only_field(&text).value, FieldValueState::Valid(_)),
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn adr_0087_query_parser_r87_1_28_kind_words_resolve_to_release_kinds() {
        for kind in [KindValue::Album, KindValue::Ep, KindValue::Single] {
            let word = kind.word();
            let resolved = ReleaseKind::resolve(Some(word), 2);
            assert!(resolved.label().eq_ignore_ascii_case(word), "{word}");
        }
    }

    #[test]
    fn adr_0087_query_parser_r87_1_29_field_errors_in_text_order() {
        let query = SearchQuery::parse("after:yesterday kind:lp haleen");
        let errors = query.field_errors();
        assert_eq!(errors.len(), 2);
        let tokens = ["after:", "kind:"];
        for ((term, error), token) in errors.iter().zip(tokens) {
            let message = error.message();
            assert!(!message.is_empty());
            assert!(message.starts_with(token), "{message}");
            assert!(
                matches!(&term.kind, TermKind::Field(field) if field.name.info().token == token)
            );
        }
    }

    /// Packet 005: a quoted field value is one item. Its range starts at
    /// the opening quote, and its prefix is the raw text from the quote.
    #[test]
    fn adr_0087_query_parser_r87_1_31_quoted_value_range_holds_its_quotes() {
        let text = r#"artist:"Haleen M"#;
        assert_eq!(
            cursor_term(text, text.len()).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Artist,
                prefix: r#""Haleen M"#.to_owned(),
                value_range: 7..16,
            }
        );
        let text = r#"artist:"a \"b" x"#;
        assert_eq!(
            cursor_term(text, 12).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Artist,
                prefix: r#""a \""#.to_owned(),
                value_range: 7..14,
            }
        );
        assert_eq!(
            cursor_term(r#"kind:"si,ep"#, 8).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Kind,
                prefix: r#""si"#.to_owned(),
                value_range: 5..11,
            }
        );
        assert_eq!(
            cursor_term(r#"artist:"x""#, 7).partial,
            CursorPartial::FieldValue {
                field: QueryFieldName::Artist,
                prefix: String::new(),
                value_range: 7..10,
            }
        );
    }

    /// R87-5-07: `quote_field_value` quotes a text with whitespace or a
    /// quote, and the parser reads the result as the same text.
    #[test]
    fn adr_0087_field_values_r87_5_07_quote_field_value_round_trips() {
        assert_eq!(quote_field_value("jsmovie"), "jsmovie");
        assert_eq!(quote_field_value("aged friends"), r#""aged friends""#);
        assert_eq!(quote_field_value(r#"a "b" \c"#), r#""a \"b\" \\c""#);
        for text in ["jsmovie", "aged friends", r#"a "b" \c"#] {
            let query = format!("artist:{}", quote_field_value(text));
            assert_eq!(valid(&query), FieldValue::Text(text.to_owned()), "{text:?}");
        }
    }

    /// R87-5-08: `unquote_field_value_prefix` removes the opening quote and
    /// resolves the escapes after it.
    #[test]
    fn adr_0087_field_values_r87_5_08_unquote_a_value_prefix() {
        assert_eq!(unquote_field_value_prefix(r#""aged fr"#), "aged fr");
        assert_eq!(unquote_field_value_prefix(r#""a \"b"#), r#"a "b"#);
        assert_eq!(unquote_field_value_prefix("hal"), "hal");
        assert_eq!(unquote_field_value_prefix(r#""Haleen""#), "Haleen");
    }

    /// ADR 0089 Decision 4: an empty query gets `in:library` at its end, and
    /// the scope All removes that term again.
    #[test]
    fn adr_0089_task_008_library_scope_on_an_empty_query() {
        assert_eq!(with_library_scope("", ListScope::InLibrary), "in:library");
        assert_eq!(with_library_scope("in:library", ListScope::All), "");
        assert_eq!(with_library_scope("", ListScope::All), "");
    }

    /// One word gets the term after one space, and the scope All keeps it.
    #[test]
    fn adr_0089_task_008_library_scope_with_one_term() {
        assert_eq!(
            with_library_scope("haleen", ListScope::InLibrary),
            "haleen in:library"
        );
        assert_eq!(
            with_library_scope("haleen in:library", ListScope::All),
            "haleen"
        );
        assert_eq!(
            with_library_scope("haleen ", ListScope::InLibrary),
            "haleen in:library"
        );
        assert_eq!(with_library_scope("haleen", ListScope::All), "haleen");
    }

    /// A quoted phrase keeps its text. A phrase that holds `in:library` is
    /// not a field, so it stays in the text.
    #[test]
    fn adr_0089_task_008_library_scope_keeps_a_quoted_term() {
        assert_eq!(
            with_library_scope("\"night drive\"", ListScope::InLibrary),
            "\"night drive\" in:library"
        );
        assert_eq!(
            with_library_scope("\"in:library\"", ListScope::InLibrary),
            "\"in:library\" in:library"
        );
        assert_eq!(
            with_library_scope("\"in:library\" in:library", ListScope::All),
            "\"in:library\""
        );
    }

    /// A term in the middle is removed with the space before it. Its
    /// neighbours keep their spacing, and `InLibrary` changes nothing when
    /// the term is there.
    #[test]
    fn adr_0089_task_008_library_scope_with_a_term_in_the_middle() {
        assert_eq!(
            with_library_scope("artist:x in:library -title:remix", ListScope::All),
            "artist:x -title:remix"
        );
        assert_eq!(
            with_library_scope("haleen  in:library  night", ListScope::All),
            "haleen  night"
        );
        assert_eq!(
            with_library_scope("in:library haleen night", ListScope::All),
            "haleen night"
        );
        assert_eq!(
            with_library_scope("a in:library b", ListScope::InLibrary),
            "a in:library b"
        );
    }

    /// Two copies of the term are both removed. A `-in:library` is an
    /// exclusion, so the scope does not remove it.
    #[test]
    fn adr_0089_task_008_library_scope_removes_each_copy_and_keeps_exclusion() {
        assert_eq!(
            with_library_scope("a in:library in:library b", ListScope::All),
            "a b"
        );
        assert_eq!(
            with_library_scope("-in:library haleen", ListScope::InLibrary),
            "-in:library haleen in:library"
        );
        assert_eq!(
            with_library_scope("-in:library haleen", ListScope::All),
            "-in:library haleen"
        );
    }

    /// `in:index` does not change. The scope In Library adds its term, and
    /// the query then has the conflict of ADR 0087.
    #[test]
    fn adr_0089_task_008_library_scope_with_in_index_keeps_the_index_term() {
        let added = with_library_scope("in:index", ListScope::InLibrary);
        assert_eq!(added, "in:index in:library");
        assert_eq!(SearchQuery::parse(&added).scope(), QueryScope::Conflict);
        assert_eq!(with_library_scope("in:index", ListScope::All), "in:index");
        assert_eq!(
            with_library_scope("in:library in:index", ListScope::All),
            "in:index"
        );
    }

    /// The control shows In Library for an `in:library` term and for a
    /// field of the Library only. Each other text shows All.
    #[test]
    fn adr_0089_task_008_library_scope_of_reads_the_query_scope() {
        assert_eq!(library_scope_of("in:library haleen"), ListScope::InLibrary);
        assert_eq!(library_scope_of("is:ready"), ListScope::InLibrary);
        assert_eq!(
            library_scope_of("in:index in:library"),
            ListScope::InLibrary
        );
        assert_eq!(library_scope_of("haleen"), ListScope::All);
        assert_eq!(library_scope_of("in:index"), ListScope::All);
        assert_eq!(library_scope_of(""), ListScope::All);
    }
}

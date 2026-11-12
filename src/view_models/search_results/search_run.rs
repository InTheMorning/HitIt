//! The run of one search query (ADR 0087 Decisions 1, 5 and 6, packets 002
//! and 011).
//!
//! Each search run parses its text one time. The typed query decides the
//! parts that run: the Library part runs for the scopes `Everything` and
//! `LibraryOnly`. The Index part sends a request only when the Index can
//! apply the query. The `q` of that request holds only the words and the
//! phrases of the query (ADR 0087 invariant "No Index request carries field
//! syntax in `q`"). The Index plan, its request and its filters come from the
//! crate `musicindex-search` (musicindex ADR 0049 R-QUERY-1). This module
//! re-exports them.
//!
//! When the Index part sends no request, the plan names the cause. The
//! search page then shows the note of Decision 5.
//!
//! The module has no GPUI type and no I/O.

#![warn(clippy::pedantic)]

pub(crate) use musicindex_search::plan::{IndexEntity, IndexSearchPlan, IndexSearchRequest};
pub(crate) use musicindex_search::request::{IndexOrder, IndexRun};

#[cfg(test)]
use chrono::NaiveDate;
#[cfg(test)]
use musicindex_search::plan::{IndexFilters, IndexSkip, UnsupportedIndexTerms};
#[cfg(test)]
use musicindex_search::query::KindValue;

use crate::db::TrackRow;
use crate::view_models::live_run::RunGeneration;
use crate::view_models::search_query::{QueryScope, QueryTerm, SearchQuery};

/// The sentence of the app about its Library rows. It follows the text of a
/// note of the crate under the rows (musicindex ADR 0052 R-QUERY-8 item 2,
/// ADR 0091).
pub(crate) const INDEX_SKIP_ROWS_TEXT: &str = "The Library rows show.";

/// The title of the empty state of a query whose scopes cannot apply
/// together.
pub(crate) const SCOPE_CONFLICT_TITLE: &str = "No results";

/// What started a search run (ADR 0087 Decision 4, packet 009).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SearchRunTrigger {
    /// The pause after a change of the field text ended.
    Pause,
    /// Enter, Back, Forward, a page open, a saved search, a retry or a
    /// selected sort.
    #[default]
    Command,
}

impl SearchRunTrigger {
    /// The kind of the Index run of this trigger. A pause after a typed change
    /// is a live run, and it keeps relevance (Stophammer ADR 0084).
    #[must_use]
    pub(crate) fn index_run(self) -> IndexRun {
        match self {
            Self::Pause => IndexRun::Live,
            Self::Command => IndexRun::Command,
        }
    }
}

/// The run of one query text: the trimmed text, the typed query and the
/// Index plan.
#[derive(Clone, Debug)]
pub(crate) struct SearchRun {
    text: String,
    query: SearchQuery,
    index_plan: IndexSearchPlan,
    /// The generation of the run (ADR 0087 Decision 4, packet 003).
    generation: RunGeneration,
    /// What started the run (ADR 0087 Decision 4, packet 009).
    trigger: SearchRunTrigger,
}

impl SearchRun {
    /// Trims `text`, parses it one time and computes the Index plan.
    #[must_use]
    pub(crate) fn new(text: &str) -> SearchRun {
        let text = text.trim().to_owned();
        let query = SearchQuery::parse(&text);
        let index_plan = IndexSearchPlan::for_query(&query);
        SearchRun {
            text,
            query,
            index_plan,
            generation: RunGeneration::default(),
            trigger: SearchRunTrigger::Command,
        }
    }

    /// Gives this run with the trigger `trigger`. The text, the query, the
    /// plan and the generation do not change.
    #[must_use]
    pub(crate) fn with_trigger(mut self, trigger: SearchRunTrigger) -> SearchRun {
        self.trigger = trigger;
        self
    }

    /// The trigger of the run. [`SearchRun::new`] gives
    /// `SearchRunTrigger::Command`.
    #[must_use]
    pub(crate) fn trigger(&self) -> SearchRunTrigger {
        self.trigger
    }

    /// Gives this run with the generation `generation` (packet 003). The
    /// text, the query and the plan do not change.
    #[must_use]
    pub(crate) fn with_generation(mut self, generation: RunGeneration) -> SearchRun {
        self.generation = generation;
        self
    }

    /// The generation of the run. [`SearchRun::new`] gives
    /// `RunGeneration::default()`.
    #[must_use]
    pub(crate) fn generation(&self) -> RunGeneration {
        self.generation
    }

    /// The trimmed query text.
    #[must_use]
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// The typed query of the text.
    #[must_use]
    pub(crate) fn query(&self) -> &SearchQuery {
        &self.query
    }

    /// The plan of the Index part.
    #[must_use]
    pub(crate) fn index_plan(&self) -> &IndexSearchPlan {
        &self.index_plan
    }

    /// `true` when the query holds a term that can run, with or without `-`
    /// (ADR 0087 Decision 4). A run that is not runnable opens no page and
    /// sends nothing (packet 002 decision D19).
    #[must_use]
    pub(crate) fn is_runnable(&self) -> bool {
        query_can_run(&self.query)
    }

    /// `true` when the Library part runs (packet 002 decision D15).
    #[must_use]
    pub(crate) fn runs_library(&self) -> bool {
        library_part_runs(&self.query)
    }
}

/// The step of a live run after the pause (ADR 0087 Decision 4, packet
/// 003).
#[derive(Clone, Debug)]
pub(crate) enum LiveRunStep {
    /// Run this search.
    Run(SearchRun),
    /// The text has no term that can run, so nothing runs.
    NotRunnable,
    /// The trimmed text is the query of the mounted search page, so nothing
    /// runs (packet 003 decision D6).
    SameQuery,
    /// The open tab is not Music, so nothing runs (packet 003 decision
    /// D8).
    NotMusic,
}

/// The step of a live run of `text` when the pause ends. `mounted_query`
/// is the query of the search page of the current path. `music_tab` is
/// `true` when the Music tab is open. The first true row of packet 003
/// "The live run" gives the step.
#[must_use]
pub(crate) fn live_run_step(
    text: &str,
    mounted_query: Option<&str>,
    music_tab: bool,
) -> LiveRunStep {
    if !music_tab {
        return LiveRunStep::NotMusic;
    }
    let run = SearchRun::new(text);
    if !run.is_runnable() {
        return LiveRunStep::NotRunnable;
    }
    if mounted_query == Some(run.text()) {
        return LiveRunStep::SameQuery;
    }
    LiveRunStep::Run(run.with_trigger(SearchRunTrigger::Pause))
}

/// `true` when `query` can run: it holds a positive term that can run, or an
/// exclusion that can apply (ADR 0087 Decision 4). The Library lists each row
/// except the excluded rows, and the Index part gives its note.
fn query_can_run(query: &SearchQuery) -> bool {
    query.terms().iter().any(QueryTerm::can_run)
}

/// `true` when the Library part of `query` runs: the query is runnable, and
/// its scope is `Everything` or `LibraryOnly` (packet 002 decision D15).
#[must_use]
pub(crate) fn library_part_runs(query: &SearchQuery) -> bool {
    query_can_run(query)
        && matches!(
            query.scope(),
            QueryScope::Everything | QueryScope::LibraryOnly
        )
}

/// The Library rows of one run (packet 002 "The Library facts from the
/// tree").
#[derive(Clone, Debug, Default)]
pub(crate) struct LibrarySearchRows {
    /// The rows of the Releases tab and the Artists tab.
    pub(crate) releases: Vec<TrackRow>,
    /// The rows of the Tracks tab.
    pub(crate) tracks: Vec<TrackRow>,
    /// The row count of SQL before the date split (packet 002 decision
    /// D22).
    pub(crate) matched_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(text: &str) -> IndexSearchPlan {
        IndexSearchPlan::for_query(&SearchQuery::parse(text))
    }

    fn unsupported(field_tokens: Vec<&'static str>) -> IndexSearchPlan {
        IndexSearchPlan::Skip(IndexSkip::Unsupported(UnsupportedIndexTerms {
            field_tokens,
        }))
    }

    /// The request of a query that sends one.
    fn request(text: &str) -> IndexSearchRequest {
        match plan(text) {
            IndexSearchPlan::Send(request) => request,
            other @ IndexSearchPlan::Skip(_) => panic!("{text} must send a request, got {other:?}"),
        }
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("test date is correct")
    }

    /// R87-2-21: a Library-only field gives `Skip(LibraryOnly)`, also with
    /// `-`. `in:index` with a Library-only field gives `Skip(Conflict)`.
    #[test]
    fn adr_0087_library_fields_r87_2_21_scope_skips_the_index() {
        for text in [
            "haleen is:ready",
            "haleen -has:split",
            "haleen playlist:x",
            "in:library haleen",
        ] {
            assert_eq!(
                plan(text),
                IndexSearchPlan::Skip(IndexSkip::LibraryOnly),
                "R87-2-21: {text}"
            );
        }
        assert_eq!(
            plan("in:index is:ready"),
            IndexSearchPlan::Skip(IndexSkip::Conflict)
        );
    }

    /// R87-2-22: an excluded `kind:`, `after:`, `before:` or `duration:` gives
    /// `Skip(Unsupported)`. The contract has no `not_` form for them. Each
    /// distinct token gives one entry, in text order. `note_title` gives each
    /// text of the note of Decision 5.
    #[test]
    fn adr_0087_library_fields_r87_2_22_excluded_terms_the_index_cannot_express() {
        assert_eq!(plan("haleen -kind:single"), unsupported(vec!["kind:"]));
        assert_eq!(
            plan("haleen -after:2026-10 -kind:ep -after:2026-11"),
            unsupported(vec!["after:", "kind:"])
        );
        assert_eq!(plan("haleen -duration:>5m"), unsupported(vec!["duration:"]));
        assert_eq!(
            plan("haleen -before:2026-10-01"),
            unsupported(vec!["before:"])
        );

        let title = |tokens: Vec<&'static str>| {
            UnsupportedIndexTerms {
                field_tokens: tokens,
            }
            .note_title()
        };
        assert_eq!(title(vec!["kind:"]), "Music Index cannot exclude kind: yet");
        assert_eq!(
            title(vec!["kind:", "after:"]),
            "Music Index cannot exclude kind: and after: yet"
        );
        assert_eq!(
            title(vec!["after:", "before:", "duration:"]),
            "Music Index cannot exclude after:, before: and duration: yet"
        );
    }

    /// R87-2-23 (plan part): `q` holds only the words and the phrases of the
    /// query. A query with no word and no positive field sends nothing. `in:`
    /// never reaches `q`.
    #[test]
    fn adr_0087_library_fields_r87_2_23_send_holds_only_words() {
        let query = SearchQuery::parse("haleen \"night drive\"");
        assert_eq!(
            IndexSearchPlan::for_query(&query),
            IndexSearchPlan::Send(IndexSearchRequest {
                q: query.index_q(),
                filters: Box::default(),
            })
        );
        assert_eq!(
            plan("haleen in:index"),
            IndexSearchPlan::Send(IndexSearchRequest {
                q: Some("haleen".to_owned()),
                filters: Box::default(),
            })
        );
        assert_eq!(plan("is:"), IndexSearchPlan::Skip(IndexSkip::NoIndexPart));
    }

    /// R87-11-01: each correct field gives its own filter (ADR 0087 Decisions
    /// 2 and 5). The words stay in `q`. A phrase keeps its double quotes.
    #[test]
    fn adr_0087_index_fields_r87_11_01_each_field_gives_its_filter() {
        let sent = request(
            r#"haleen artist:jsmovie title:"night drive" release:"aged friends" kind:single,ep after:2026-10-01 before:2026-10-31 duration:>5m"#,
        );
        assert_eq!(sent.q.as_deref(), Some("haleen"));
        assert_eq!(
            *sent.filters,
            IndexFilters {
                artist: Some("jsmovie".to_owned()),
                title: Some("\"night drive\"".to_owned()),
                release: Some("\"aged friends\"".to_owned()),
                kinds: vec![KindValue::Single, KindValue::Ep],
                after: Some(date(2026, 10, 1)),
                before: Some(date(2026, 10, 31)),
                min_duration: Some(301),
                ..IndexFilters::default()
            }
        );
    }

    /// R87-11-02: a query with only fields sends no `q`. A month in `after:`
    /// gives its first day, and a month in `before:` gives its last day.
    #[test]
    fn adr_0087_index_fields_r87_11_02_fields_only_send_no_q() {
        assert_eq!(
            request("kind:single"),
            IndexSearchRequest {
                q: None,
                filters: Box::new(IndexFilters {
                    kinds: vec![KindValue::Single],
                    ..IndexFilters::default()
                }),
            }
        );
        let sent = request("after:2026-10 artist:jsmovie");
        assert_eq!(sent.q, None);
        assert_eq!(sent.filters.after, Some(date(2026, 10, 1)));
        assert_eq!(sent.filters.before, None);
    }

    /// R87-11-03: an excluded word or phrase gives `not_q`. An excluded
    /// `artist:`, `title:` or `release:` gives its `not_` filter. Each value
    /// is one exclusion.
    #[test]
    fn adr_0087_index_fields_r87_11_03_exclusions_give_not_parameters() {
        let sent = request(
            r#"haleen -remix -"night drive" -artist:jsmovie -title:remix -release:"aged friends""#,
        );
        assert_eq!(sent.q.as_deref(), Some("haleen"));
        assert_eq!(
            *sent.filters,
            IndexFilters {
                not_artist: vec!["jsmovie".to_owned()],
                not_title: vec!["remix".to_owned()],
                not_release: vec!["\"aged friends\"".to_owned()],
                not_q: vec!["remix".to_owned(), "\"night drive\"".to_owned()],
                ..IndexFilters::default()
            }
        );
    }

    /// R87-11-04: the month and the duration forms convert to the values of
    /// the contract. The day of `before:` is the last day of the month, and a
    /// leap year keeps its 29 February. A duration gives whole seconds, each
    /// limit included.
    #[test]
    fn adr_0087_index_fields_r87_11_04_month_and_duration_conversion() {
        let filters = |text: &str| request(text).filters;
        assert_eq!(
            filters("haleen before:2028-02").before,
            Some(date(2028, 2, 29))
        );
        assert_eq!(
            filters("haleen after:2026-12").after,
            Some(date(2026, 12, 1))
        );
        assert_eq!(
            filters("haleen before:2026-12").before,
            Some(date(2026, 12, 31))
        );
        assert_eq!(
            filters("haleen after:2026-10-01").after,
            Some(date(2026, 10, 1))
        );
        let timed = filters("haleen duration:<3:30");
        assert_eq!((timed.min_duration, timed.max_duration), (None, Some(209)));
        let range = filters("haleen duration:3m..5m");
        assert_eq!(
            (range.min_duration, range.max_duration),
            (Some(180), Some(300))
        );
        let hours = filters("haleen duration:>=1h");
        assert_eq!((hours.min_duration, hours.max_duration), (Some(3600), None));
        let seconds = filters("haleen duration:<=90s");
        assert_eq!(
            (seconds.min_duration, seconds.max_duration),
            (None, Some(90))
        );
    }

    /// R87-11-05: `in:` gives no filter, and a Library-only field still sends
    /// nothing. A word with `in:index` and a field sends one request.
    #[test]
    fn adr_0087_index_fields_r87_11_05_scope_and_library_fields_give_no_filter() {
        let sent = request("haleen in:index artist:x");
        assert_eq!(sent.q.as_deref(), Some("haleen"));
        assert_eq!(sent.filters.artist.as_deref(), Some("x"));
        assert_eq!(
            plan("haleen is:ready"),
            IndexSearchPlan::Skip(IndexSkip::LibraryOnly)
        );
        assert_eq!(
            plan("in:library kind:ep"),
            IndexSearchPlan::Skip(IndexSkip::LibraryOnly)
        );
    }

    /// R87-11-06: several terms of one field. A second `artist:` joins the
    /// first one. The `after:` terms give the later day, and the `before:`
    /// terms give the earlier day. Two `duration:` terms give their common
    /// range. `kind:` terms share their kinds, and no shared kind gives no
    /// request.
    #[test]
    fn adr_0087_index_fields_r87_11_06_repeated_fields_combine() {
        let sent = request(
            r#"haleen artist:jsmovie artist:"night drive" after:2026-10-01 after:2026-10-05 before:2026-10-31 before:2026-10-03 duration:>5m duration:<10m"#,
        );
        assert_eq!(
            sent.filters.artist.as_deref(),
            Some("jsmovie \"night drive\"")
        );
        assert_eq!(sent.filters.after, Some(date(2026, 10, 5)));
        assert_eq!(sent.filters.before, Some(date(2026, 10, 3)));
        assert_eq!(
            (sent.filters.min_duration, sent.filters.max_duration),
            (Some(301), Some(599))
        );
        // musicindex ADR 0049 R-QUERY-6: `ep` has no Index request, so the
        // shared kind `ep` sends none. Before ADR 0091 step 2 it sent `kind=ep`.
        assert_eq!(
            plan("kind:album,ep kind:ep"),
            IndexSearchPlan::Skip(IndexSkip::NoIndexPart)
        );
        assert_eq!(
            plan("kind:single kind:ep"),
            IndexSearchPlan::Skip(IndexSkip::NoIndexPart)
        );
    }

    /// R87-11-07: a query with exclusions only sends nothing, because
    /// Stophammer needs a positive term. A word with exclusions sends one
    /// request.
    #[test]
    fn adr_0087_index_fields_r87_11_07_exclusions_alone_send_nothing() {
        // The crate gives the cause `ExclusionsOnly` for these (musicindex ADR 0049).
        assert_eq!(
            plan("-remix"),
            IndexSearchPlan::Skip(IndexSkip::ExclusionsOnly)
        );
        assert_eq!(
            plan("-title:remix"),
            IndexSearchPlan::Skip(IndexSkip::ExclusionsOnly)
        );
        assert!(matches!(
            plan("-title:remix haleen"),
            IndexSearchPlan::Send(_)
        ));
    }

    /// R87-11-08: `for_entity` gives the request of each entity type. A
    /// publisher row takes no release, kind, date or duration. A feed row
    /// takes no duration. A track takes each filter. A type with no positive
    /// term gets no request.
    #[test]
    fn adr_0087_index_fields_r87_11_08_each_entity_takes_its_filters() {
        let publisher =
            request("haleen artist:x title:y -release:z").for_entity(IndexEntity::Publisher);
        let publisher = publisher.expect("the publisher takes artist and title");
        assert_eq!(publisher.filters.artist.as_deref(), Some("x"));
        assert_eq!(publisher.filters.title.as_deref(), Some("y"));
        assert!(publisher.filters.not_release.is_empty());

        assert_eq!(
            request("haleen kind:single").for_entity(IndexEntity::Publisher),
            None
        );
        assert_eq!(
            request("haleen after:2026-10").for_entity(IndexEntity::Publisher),
            None
        );

        let timed = request("haleen duration:>5m");
        assert_eq!(timed.for_entity(IndexEntity::Feed), None);
        assert_eq!(timed.for_entity(IndexEntity::Publisher), None);
        let track = timed
            .for_entity(IndexEntity::Track)
            .expect("a track takes a duration");
        assert_eq!(track.filters.min_duration, Some(301));

        let feed = request("kind:single -title:x").for_entity(IndexEntity::Feed);
        assert_eq!(
            feed.expect("a feed takes a kind").filters.kinds,
            vec![KindValue::Single]
        );

        let exclusions_only = IndexSearchRequest {
            q: None,
            filters: Box::new(IndexFilters {
                not_title: vec!["x".to_owned()],
                ..IndexFilters::default()
            }),
        };
        assert_eq!(exclusions_only.for_entity(IndexEntity::Track), None);
    }

    /// R87-11-09: an excluded `kind:`, `after:`, `before:` or `duration:`
    /// gives `Skip(Unsupported)`, even when a word is in the query. A value
    /// that is not correct applies nothing, so it gives no skip.
    #[test]
    fn adr_0087_index_fields_r87_11_09_unsupported_before_the_filters() {
        assert_eq!(plan("haleen -kind:single"), unsupported(vec!["kind:"]));
        assert!(matches!(
            plan("haleen -after:yesterday"),
            IndexSearchPlan::Send(_)
        ));
    }

    /// R87-3-05: a text with no runnable term gives `NotRunnable`. A text
    /// with a word and a field with no value gives `Run`.
    #[test]
    fn adr_0087_live_results_r87_3_05_text_with_no_runnable_term_runs_nothing() {
        for text in [
            "",
            "   ",
            "!!!",
            "-!!!",
            "artist:",
            "-artist:",
            "-after:yesterday",
            "in:library",
            "-in:library",
        ] {
            assert!(
                matches!(live_run_step(text, None, true), LiveRunStep::NotRunnable),
                "R87-3-05: {text:?}"
            );
        }
        assert!(matches!(
            live_run_step("haleen artist:", None, true),
            LiveRunStep::Run(run) if run.text() == "haleen artist:"
        ));
    }

    /// ADR 0087 Decision 4: a query with only exclusions runs. The Library
    /// part runs in the scopes that hold the Library, and the Index part sends
    /// no request. The agent screenshot run of 2026-11-07 found that this
    /// query started no run, so the note did not show.
    #[test]
    fn adr_0087_live_results_exclusions_alone_start_a_run() {
        for text in ["-remix", "-\"night drive\"", "-title:remix", "-kind:single"] {
            let LiveRunStep::Run(run) = live_run_step(text, None, true) else {
                panic!("{text:?} must run");
            };
            assert!(run.runs_library(), "{text:?}");
            assert!(
                matches!(run.index_plan(), IndexSearchPlan::Skip(_)),
                "{text:?}"
            );
        }
        let library = SearchRun::new("-remix in:library");
        assert!(library.is_runnable());
        assert!(library.runs_library());
        assert_eq!(
            *library.index_plan(),
            IndexSearchPlan::Skip(IndexSkip::LibraryOnly)
        );
        assert_eq!(
            *SearchRun::new("-remix").index_plan(),
            IndexSearchPlan::Skip(IndexSkip::ExclusionsOnly)
        );
    }

    /// R87-3-06: the query of the mounted page gives `SameQuery`. A
    /// different text gives `Run`. A tab other than Music gives `NotMusic`.
    #[test]
    fn adr_0087_live_results_r87_3_06_same_query_and_other_tab_run_nothing() {
        assert!(matches!(
            live_run_step("haleen ", Some("haleen"), true),
            LiveRunStep::SameQuery
        ));
        assert!(matches!(
            live_run_step("haleen", Some("hale"), true),
            LiveRunStep::Run(_)
        ));
        for text in ["", "haleen", "artist:", "haleen artist:"] {
            for mounted in [None, Some("hale"), Some("haleen")] {
                assert!(
                    matches!(live_run_step(text, mounted, false), LiveRunStep::NotMusic),
                    "R87-3-06: {text:?} {mounted:?}"
                );
            }
        }
    }

    /// R87-3-07: a new run has the default generation. `with_generation`
    /// changes only the generation.
    #[test]
    fn adr_0087_live_results_r87_3_07_with_generation_keeps_the_run() {
        let run = SearchRun::new(" haleen after:2026-10 ");
        assert_eq!(run.generation(), RunGeneration::default());
        let mut gate = crate::view_models::live_run::LiveRunGate::default();
        let generation = gate.next_run();
        let next = run.clone().with_generation(generation);
        assert_eq!(next.generation(), generation);
        assert_ne!(next.generation(), RunGeneration::default());
        assert_eq!(next.text(), run.text());
        assert_eq!(next.query(), run.query());
        assert_eq!(next.index_plan(), run.index_plan());
    }

    /// R87-2-28: a saved search keeps its text, and each run parses it
    /// again (ADR 0087 Decision 6).
    /// R87-9-01: a new run has the trigger `Command`. A live run of the
    /// pause has the trigger `Pause`, and `with_generation` keeps it.
    #[test]
    fn adr_0087_one_index_error_r87_9_01_live_run_has_the_pause_trigger() {
        assert_eq!(
            SearchRun::new("haleen d").trigger(),
            SearchRunTrigger::Command
        );
        let LiveRunStep::Run(run) = live_run_step("haleen d", Some("haleen"), true) else {
            panic!("R87-9-01: the text of a new run runs");
        };
        assert_eq!(run.trigger(), SearchRunTrigger::Pause);
        let mut gate = crate::view_models::live_run::LiveRunGate::default();
        let kept = run.with_generation(gate.next_run());
        assert_eq!(kept.trigger(), SearchRunTrigger::Pause);
    }

    #[test]
    fn adr_0087_library_fields_r87_2_28_saved_text_parses_at_each_run() -> anyhow::Result<()> {
        use crate::db::saved_searches::{insert_saved_search, saved_searches, StoredStartTab};

        let conn = rusqlite::Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        crate::db::init_schema(&conn)?;
        crate::db::migrate_schema(&conn)?;
        let text = "artist:jsmovie kind:single -title:remix";

        insert_saved_search(&conn, text, StoredStartTab::Releases, 1_792_000_000)?;
        let loaded = saved_searches(&conn)?;

        let run = SearchRun::new(&loaded[0].query);
        assert_eq!(run.text(), text);
        assert_eq!(run.query(), &SearchQuery::parse(text));
        assert!(run.is_runnable());
        assert!(run.runs_library());
        Ok(())
    }
}

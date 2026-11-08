//! The sort of the search rows (ADR 0089 Decisions 4 and 5).
//!
//! The sort orders the rows of one run. It does not change the query and
//! sends no request. Newest puts the newest date first, and a row with no
//! date goes last. A to Z orders the rows by name, without case. Both sorts
//! are stable, so rows that compare equal keep the order of the run.

#![warn(clippy::pedantic)]

use std::cmp::Ordering;

use crate::view_models::list_page::ListSort;

/// Orders `rows` by `sort`. `date` gives the Unix seconds of a row, or
/// `None` for a row with no date. `label` gives the name of a row.
pub(super) fn sort_rows<Row>(
    rows: &mut [Row],
    sort: ListSort,
    date: impl Fn(&Row) -> Option<i64>,
    label: impl Fn(&Row) -> &str,
) {
    match sort {
        // Relevance keeps the order of the run (musicindex ADR 0053 R-LIST-9).
        ListSort::Relevance => {}
        ListSort::Newest => rows.sort_by(|a, b| newest_first(date(a), date(b))),
        ListSort::AToZ => rows.sort_by_key(|row| label(row).to_lowercase()),
    }
}

/// The order of two dates, newest first. A row with no date goes last.
fn newest_first(a: Option<i64>, b: Option<i64>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => b.cmp(&a),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0089 Decision 4: Newest puts the newest date first, a row with
    /// no date goes last, and rows with equal dates keep their order.
    #[test]
    fn adr_0089_task_008_newest_puts_dateless_rows_last_and_is_stable() {
        let mut rows = vec![
            ("none-first", None),
            ("old", Some(10)),
            ("new", Some(30)),
            ("none-second", None),
            ("new-too", Some(30)),
        ];
        sort_rows(&mut rows, ListSort::Newest, |row| row.1, |row| row.0);
        let order: Vec<&str> = rows.iter().map(|row| row.0).collect();
        assert_eq!(
            order,
            ["new", "new-too", "old", "none-first", "none-second"]
        );
    }

    /// ADR 0089 Decision 5: A to Z orders by name without case.
    #[test]
    fn adr_0089_task_008_a_to_z_orders_by_name_without_case() {
        let mut rows = vec![("beta", Some(1)), ("Alpha", None), ("gamma", Some(9))];
        sort_rows(&mut rows, ListSort::AToZ, |row| row.1, |row| row.0);
        let order: Vec<&str> = rows.iter().map(|row| row.0).collect();
        assert_eq!(order, ["Alpha", "beta", "gamma"]);
    }

    /// Musicindex ADR 0053 R-LIST-9: Relevance keeps the order of the run.
    #[test]
    fn adr_0053_r9_relevance_keeps_the_run_order() {
        let mut rows = vec![("beta", Some(1)), ("Alpha", Some(9)), ("gamma", None)];
        sort_rows(&mut rows, ListSort::Relevance, |row| row.1, |row| row.0);
        let order: Vec<&str> = rows.iter().map(|row| row.0).collect();
        assert_eq!(order, ["beta", "Alpha", "gamma"]);
    }
}

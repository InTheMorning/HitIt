//! The count of a task with a known length (ADR 0101 Decision 12, musicindex
//! ADR 0026 R-LOADING-1 item 3).
//!
//! [`ProgressCount`] gives the count text, for example "3 of 9 done", and the
//! fraction of the bar. The text carries the meaning. The bar only repeats it.

#![warn(clippy::pedantic)]

/// How many parts of a task are done, and how many parts the task has.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProgressCount {
    pub(crate) done: usize,
    pub(crate) total: usize,
}

impl ProgressCount {
    /// The count text, for example "3 of 9 done". The word "done" keeps it
    /// apart from the position in the title of a download item.
    #[must_use]
    pub(crate) fn count_text(self) -> String {
        format!("{} of {} done", self.done, self.total)
    }

    /// The part of the bar that is full, from 0.0 to 1.0. A task of no
    /// parts gives 0.0.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a count of parts is far below the range where f32 loses a unit"
    )]
    pub(crate) fn fraction(self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.done.min(self.total) as f32) / (self.total as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressCount;

    /// R-LOADING-1 item 3: a download of nine tracks with three done shows
    /// "3 of 9 done" and a bar that is one third full.
    #[test]
    fn progress_count_gives_the_count_text_and_the_fraction() {
        let count = ProgressCount { done: 3, total: 9 };
        assert_eq!(count.count_text(), "3 of 9 done");
        assert!((count.fraction() - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn progress_count_keeps_the_fraction_between_zero_and_one() {
        assert!(ProgressCount { done: 0, total: 0 }.fraction().abs() < f32::EPSILON);
        assert!((ProgressCount { done: 12, total: 9 }.fraction() - 1.0).abs() < f32::EPSILON);
        assert!(ProgressCount { done: 0, total: 9 }.fraction().abs() < f32::EPSILON);
    }
}

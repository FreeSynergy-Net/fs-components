//! The mark of the work-in-progress limit: `n`, or `n/max` with a limit.
//!
//! Pattern: none - pure function. The rule (b7): the mark warns above the limit and is **never a lock**; it counts every
//! card of the column, also the ones a filter hides.

/// The mark of a column head.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WipMark {
    /// `3` or `3/5`.
    pub text: String,
    /// More cards than the limit: drawn in the warning colour with a symbol (a colour alone says nothing to everyone).
    pub over: bool,
}

/// The mark for `count` cards (all of the column) and an optional `limit`.
pub fn wip_mark(count: usize, limit: Option<usize>) -> WipMark {
    match limit {
        Some(max) => WipMark {
            text: format!("{count}/{max}"),
            over: count > max,
        },
        None => WipMark {
            text: count.to_string(),
            over: false,
        },
    }
}

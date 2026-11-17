//! The source of the candidates.
//!
//! Pattern: Strategy. The picker calls it; the program implements it over its people directory (`suggest` for `search`,
//! `asker` for `me`, `resolve_many` for `display_of`). It answers from what it already has: a source that must wait on a
//! socket answers `Ok(None)` ("still loading") and the program calls `PersonPicker::refresh` when the answer is there, so the
//! interface thread never blocks.

use super::model::{CandidatePage, PersonRef, PickerError};

/// Where the people come from.
pub trait CandidateSource: Send + Sync {
    /// Candidates for `query` (empty = the first ones in the source's order), at most `limit`, in the source's rank.
    /// `Ok(None)` = not there yet (the picker shows the loading surface); `Err` = could not (the picker shows the error).
    ///
    /// # Errors
    /// The source could not answer; the picker shows the code and the parameter.
    fn search(&self, query: &str, limit: usize) -> Result<Option<CandidatePage>, PickerError>;

    /// The asking person's own reference ("assign to me"). `Ok(None)` = signed out or no own card set (not an error: the
    /// picker says "set your own card first").
    ///
    /// # Errors
    /// The source could not answer.
    fn me(&self) -> Result<Option<PersonRef>, PickerError>;

    /// The name the source releases for `person` now, for a chosen row that is not among the candidates. Asked at each draw,
    /// never stored by the picker. `None` = nothing released.
    fn display_of(&self, person: &PersonRef) -> Option<String>;
}

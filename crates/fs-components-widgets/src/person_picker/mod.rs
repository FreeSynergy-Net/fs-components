//! `person-picker`: choose people from a source the caller names. Only the picker.
//!
//! Pattern: State + Command + Adapter (like `share-dialog`) over a Strategy, [`CandidateSource`]. The picker knows no people
//! directory: the program puts an adapter over `person_directory_v1` (`suggest`, `asker`, `resolve_many`) behind the trait, and
//! gets back one value, [`Outcome::Confirmed`] with a list of opaque [`PersonRef`]s, never a name (ADR 0085 section 3: a name
//! is shown by looking it up, not copied). Data is shown only as far as the source releases it (`display: None` is drawn as
//! "name not shared", never filled in from anywhere else); the picker keeps the answer of the current query and nothing
//! more, and the chosen list holds references only.

mod bindings;
mod fake;
mod handle;
mod labels;
mod model;
mod source;
mod state;
mod styles;
mod view;

pub use bindings::{Bindings, Command};
pub use fake::{FakeSource, Script};
pub use handle::PersonPicker;
pub use labels::{Labels, TEXTS};
pub use model::{
    Candidate, CandidatePage, ErrorCode, Outcome, PersonRef, PickerError, Refusal, Selection,
};
pub use source::CandidateSource;
pub use state::{Notice, Phase, Picking};
pub use styles::STYLES;

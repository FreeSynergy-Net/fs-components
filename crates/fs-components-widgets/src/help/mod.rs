//! `help`: the help of a window, read from the texts of the program through the CONTRACT, never written here (ADR 0065, ADR 0072,
//! ADR 0097: every program has "a help entry with content or a visible fallback").
//!
//! Pattern: Strategy (`HelpSource`: where a block comes from) + Chain of Responsibility (`Chain`: the wiki of the person, the copy in
//! the package, the catalog; the highest `rev` wins, ADR 0072 point 4) + State (`Outcome`: found, missing, unavailable) + Adapter
//! (`view.rs`). A block is asked like ADR 0072 says: program, purpose `help.<topic>`, a wish list of languages with fallback;
//! the answer says whether it was shown in another language (`fallback_from`) or is older than its original (`outdated`), and
//! the view SAYS so. Nothing here knows fs-texts: the host gives a `HelpSource` that talks to the interface `texts`.
//!
//! The rule of ADR 0097 is built in: there is no way to build a help element that is empty or greyed out. A missing entry, an
//! empty block and a source that does not answer each show a sentence and, when the host gave one, the way to the documentation.

mod labels;
mod model;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use model::{
    Chain, HelpBlock, HelpError, HelpSource, Outcome, paragraphs, purpose_of, resolve,
};
pub use styles::STYLES;
pub use view::Help;

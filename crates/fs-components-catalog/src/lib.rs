//! What a building block ACCEPTS (ADR 0087 Nachtrag, stage E3, trait-delta F1): the key `accepts` of the block contract 0.4.0.
//!
//! * [`BlockCatalog`] reads `contract.toml` and the block files (`contract/blocks/*.toml`) and holds the slots of every block
//!   ([`Slot`]: kind, basic types, role, cardinality, `required`, `within`, `repeat`).
//! * [`check`] tests the bindings of one block (a forms description) against the block, the type catalog of the provider and
//!   its record shapes ([`Provider`]); it returns EVERY problem as a code with parameters ([`AcceptsCode`]), never only the
//!   first, so the host shows the part locked with the reason (user rule: a missing data source is never greyed out silently).
//! * [`ContractRange`] (`tasks@>=0.2.0, <0.3.0`, `tasks@^1.2.0`) is the contract with a version need in Cargo grammar (`semver`), never a
//!   program name (ADR 0045, 0068); a version is `x.y.z`, one rule for the whole building kit (V-3).
//!
//! No sentence in code (ADR 0057): failures are codes, the words live in MF2 files. No window, no Blinc: the host reads this
//! crate at run time and starts no Python.
//!
//! Written with Claude (Anthropic), as the FreeSynergy READMEs say; an AI is a tool here, not an author of record.

#![forbid(unsafe_code)]

mod accepts;

pub use accepts::*;
pub use fs_types::coded;

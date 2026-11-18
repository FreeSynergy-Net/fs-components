//! What a building block accepts (block contract 0.4, key `accepts`), and whether a binding fits it.
//!
//! Target: crate `fs-components-catalog` in repo fs-components (draft §1). The contract files stay the source
//! (`contract/blocks/*.toml`); the Python check `contract/check` keeps checking names, parts and states, and is to get one more
//! check for `accepts` (same rules as [`read_blocks`], draft §9; NOT there yet: only the Rust reader checks `accepts` today). The Rust reader exists because the host `fs-forms-run` reads the
//! catalog at run time and must not run Python.
//!
//! A slot is one place of a block that takes data: a value of a field (in one of the four roles of ADR 0066 §4), the
//! records of a type (a list or one record), a grouping of those records (the columns of a board, or its lanes), an order, a
//! command (a button), or the lock state of a record. A frame without data has `accepts = []`.
//!
//! Pattern: Builder (files in, one immutable [`BlockCatalog`] out) + Specification ([`check`]: all problems, never the first).

mod bind;
mod bind_check;
mod catalog;
mod choose;
mod codes;
mod parse;
mod parse_keys;
mod range;
mod renders;
mod slot;

pub use bind::{Binding, Provider, Source};
pub use bind_check::check;
pub use catalog::{BlockCatalog, BlockSpec, CONTRACT_VERSION};
pub use choose::{Chosen, FALLBACK_BLOCK};
pub use codes::{AcceptsCode, keys};
pub use parse::read_blocks;
pub use range::{ContractRange, Version2};
pub use renders::Renders;
pub use slot::{BASICS, BasicSet, Cardinality, Role, Slot, SlotKind, basic_from_key, basic_key};

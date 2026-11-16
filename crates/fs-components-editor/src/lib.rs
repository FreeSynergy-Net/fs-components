//! The contract of the editor building blocks.
//!
//! Pattern: Strategy — several editors (`SourceEditor`, `BlockEditor`, `RichEditor`) stand behind one trait,
//! [`TextEditor`]. The input is a document in the text model ([`Doc`], blocks of Markdown source), the output is only
//! change operations of that same model ([`Op`]). An editor never reformats data on its own: a document that is loaded
//! and saved without a change comes out byte for byte the same, and every change is visible as an operation.
//!
//! The text model of the wiki (`fs-texts`, in the fs-texts repository) depends on this repository, not the other way
//! round, so the contract has operation types of its own, small and block-addressed; the wiki maps them onto its model.
//! Extension points (modules in the sense of ADR 0045) are only declared here ([`modules`]); nothing is built on them yet.
//!
//! A building block is not an editor program: opening, saving and the choice of block and frontend belong to the program.

pub mod conformance;
mod conformance_v2;
mod doc;
mod edit_text;
mod editor;
pub mod modules;
mod ops;
pub mod rich;

pub use doc::{Doc, Fence, fence_run, fence_step};
pub use edit_text::{backspace_text, toggle_task_line};
pub use editor::{CONTRACT_VERSION, Edit, TextEditor};
pub use ops::{Op, OpError, apply, diff};

//! `board`: the board workbench: columns of cards that move by mouse, by keyboard and by command, with work-in-progress marks,
//! a quick entry at the foot of every column, a frame at the side for one card and a line under the board for the answer of the
//! program (moved - undo, refused - why, "move anyway?").
//!
//! Pattern: none (declares modules and re-exports only). [`BoardBench`] is what a host places. The workbench knows no program and
//! no store, only [`BoardSource`], [`BoardEvents`] and [`DetailContent`]; it writes nothing and ranks nothing. The state machines
//! work without a window: [`BoardState`] (cursor, held card, moves, notice, undo), [`Drag`] (the pointer gesture, the 4 px
//! rule), [`Geometry`] (the pointer to a column and a place), [`quick`] (the foot), [`wip_mark`], [`scroll`] (auto-scroll).
//! A workbench does not run alone (ADR 0056); it gives the program its requests through [`BoardEvents`].

mod bench;
mod bench_foot;
mod bench_menu;
pub mod commands;
mod detail;
pub mod drag;
pub mod hit;
mod labels;
mod meta;
mod model;
pub mod quick;
pub mod scroll;
mod source;
mod state;
mod state_keys;
mod state_moves;
mod state_pointer;
mod styles;
mod view;
mod view_drag;
mod view_line;
mod wip;

pub use bench::{BoardBench, ContextHook};
pub use detail::{DetailContent, DetailFrame, MAX_ELEMENTS};
pub use drag::{Drag, THRESHOLD};
pub use fs_components_kanban::{Card, Column, Due, DueState, FieldSlot, Label, MAX_FACTS, Tone};
pub use hit::{Geometry, Spot};
pub use labels::{Labels, TEXTS};
pub use meta::card_meta;
pub use model::{
    Answer, ColumnAction, ContextTarget, FilterState, MoveRequest, NewCard, Question, Reason,
    Supports,
};
pub use source::{BoardEvents, BoardSource, MemoryBoard, NoEvents};
pub use state::{Action, BoardState, Head, Notice};
pub use styles::styles;
pub use view::DETAIL_W;
pub use wip::{WipMark, wip_mark};

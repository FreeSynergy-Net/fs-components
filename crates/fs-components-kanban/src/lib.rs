//! `fs-components-kanban`: ONE board element of columns with cards. The element ([`BoardBench`]) is the core of Ports and Adapters:
//! it owns its ports ([`BoardSource`], [`BoardEvents`], [`DetailContent`]), takes columns with cards and gives requests back
//! ([`MoveRequest`], [`NewCard`]); the forms adapter (feature `form`, directory `form/`: `FormSource`, `FormEvents`, `View`) is the only part that knows channel names.
//!
//! One state machine ([`BoardState`], over the shared `Sorter`) serves mouse drag (the card component: 4 px rule, measured places, auto-scroll), keys with modifiers,
//! click-to-move, the commands of the menu and undo: every way ends in `place` and yields the same [`MoveRequest`]. Operations the
//! host does not support are drawn disabled with the reason in a tooltip ([`BoardSource::supports`], ADR 0096 H-K5). Words come from the
//! element itself ([`Labels`], `MessageFormat` 2); looks from `fs-components-styles` through the field components.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
// Pixel arithmetic over a few dozen cards (counts far below 2^23) and short names for the style and
// palette in the view functions: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::similar_names
)]

mod bands;
mod bench;
mod bench_click;
mod bench_foot;
mod bench_menu;
mod board_grid;
mod card_fit;
mod card_line;
pub mod commands;
mod detail;
#[cfg(feature = "form")]
mod form;
mod labels;
pub mod limits;
mod memory;
mod meta;
pub mod model;
mod places;
pub mod quick;
mod requests;
pub mod scroll;
pub mod sizes;
mod source;
mod state;
mod state_click;
mod state_keys;
mod state_moves;
mod styles;
pub mod swimlanes;
mod view;
mod view_card;
mod view_cell;
mod view_drag;
mod view_line;
mod view_move;
mod wip;

pub use bands::NONE_KEY;
pub use bench::{BoardBench, ContextHook};
pub use detail::{DetailContent, DetailFrame};
#[cfg(feature = "form")]
pub use form::{
    COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, FormDetail, FormEvents, FormSource, LANE, LIMIT,
    NAME, REFUSED_CODES, View, WORD_KEYS, WORD_TEXTS, carried_channels, columns_of, word_id,
};
pub use fs_components_fields::{FieldSlot, Limit, SlotReason, SlotState};
pub use labels::{Labels, TEXTS};
pub use limits::{DEFAULT_ELEMENTS, DEFAULT_FACTS, ELEMENTS_SETTING, FACTS_SETTING};
pub use memory::MemoryBoard;
pub use meta::{card_meta, card_more};
pub use model::{Card, Column, Due, DueState, Operation};
pub use requests::{
    Answer, ColumnAction, ContextTarget, FilterState, MoveRequest, NewCard, Question, Reason,
};
pub use source::{BoardEvents, BoardSource, NoEvents};
pub use state::{Action, BoardState, Head, Notice};
pub use styles::styles;
pub use swimlanes::{Lane, LaneCard, LaneHead, LanePlan, lanes_of};
pub use view::DETAIL_W;
pub use wip::{WipMark, wip_mark};

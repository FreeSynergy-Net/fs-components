//! The people workbench: a list of people, one person read, edited, created or deleted.
//!
//! Pattern: none (declares modules and re-exports only) — [`PeopleWidget`] is what a program places. Like the reader the workbench is
//! open (ADR 0037): it knows no store and no program core, only [`PeopleSource`], and every word
//! comes from [`PeopleLabels`]. The state machine [`PeopleState`] works without a window.

mod capabilities;
mod command_texts;
mod commands;
mod delete;
mod edit;
pub mod form_view;
mod keypress;
mod kinds;
mod labels;
mod list_head;
mod marking;
mod model;
mod shown;
mod source;
mod state;
mod view;
mod widget;
mod widget_actions;
mod widget_menu;

pub use commands::{
    COPY, DELETE, EDIT, EXPORT, GROUP_PREFIX, KIND as CONTEXT_KIND, MAIL, MARK_PREFIX, MERGE, NEW,
    PeopleRequest, Ran, SELECT, group_id, mark_id,
};
pub use edit::PersonEdit;
pub use form_view::View;
pub use keypress::{asks_for_menu, press_from};
pub use labels::PeopleLabels;
pub use model::{
    DraftField, DraftGroup, FieldCheck, FieldKind, GroupFilter, PeopleCapabilities, PersonDraft,
    PersonField, PersonRow, PersonSection, PersonSheet,
};
pub use shown::{Outcome, Shown, Variant};
pub use source::PeopleSource;
pub use state::{PAGE, PeopleState};
pub use view::{MenuFor, styles};
pub use widget::{ContextHook, PeopleWidget, RequestHook};

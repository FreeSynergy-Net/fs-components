//! The dialogs: `alertdialog`, `confirm-dialog`, `choice-dialog`, `input-dialog`, `license-dialog` and the frame they share (the contract block `dialog`).
//!
//! Pattern: Template method (`frame.rs`: title, body, footer, Enter and Escape, filled in by each dialog) + Decorator
//! (`modal.rs`: the page behind is taken out of the tab order). Every dialog knows no data: it hands the answer to the caller.
//! The words of the buttons come from the caller as [`Labels`], made for a language with `Localized::for_locale` from the
//! `MessageFormat` 2 files in `texts/` (ADR 0057); a dialog may rename its buttons ("Install", "Not now").
//!
//! Keys: Escape leaves (cancel, decline), Enter takes the default answer unless a button has the keyboard (then Enter is that
//! button), except where a wrong Enter would do harm (a `danger` confirm, the licence), Tab walks the buttons and the lists of the
//! dialog only and starts again at the first ([`modal`]).

mod alert_dialog;
mod choice;
mod choice_option;
mod confirm;
mod frame;
mod input;
mod labels;
mod license;
mod modal;
mod styles;

pub use alert_dialog::AlertDialog;
pub use choice::ChoiceDialog;
pub use choice_option::ChoiceOption;
pub use confirm::ConfirmDialog;
pub use input::{InputDialog, OnInput};
pub use labels::{Labels, TEXTS};
pub use license::{LicenseDialog, TEXT_HEIGHT as LICENSE_TEXT_HEIGHT};
pub use modal::modal;
pub use styles::STYLES;

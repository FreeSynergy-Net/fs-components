//! The port [`Outlet`]: where a notification is shown. The core knows no window and no operating system; a frontend (toast, modal)
//! and a system adapter each implement this one trait (ADR 0106 point 5; A3 and A2 of the cut).
//!
//! Pattern: Port and Adapter (Strategy). An outlet says which [`Form`]s it can show; the rule ([`crate::Policy`]) chooses the forms,
//! the facade hands the notification to every outlet that has a chosen form.

use fs_types::coded::Coded;

use crate::codes::NotificationCode;
use crate::message::NoteId;
use crate::notification::Notification;

/// How a notification is shown (ADR 0106 point 4). The banner is no form of a notification: it shows a state of a view that stays.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Form {
    /// A short note in the window (`fs-components-overlays::toast`).
    Toast,
    /// A dialog that needs an answer (`fs-components-overlays::dialogs`).
    Modal,
    /// The notification of the operating system. Shown WITHOUT the parameters of the texts (the lock screen is not the person's alone).
    System,
}

impl Form {
    /// The name (`toast`, `modal`, `system`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Toast => "toast",
            Self::Modal => "modal",
            Self::System => "system",
        }
    }
}

/// Where the program's window is, as far as the host knows.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Window {
    /// There is no window (a server, an updater).
    Absent,
    /// There is one, not in front or minimised.
    Away,
    /// In front.
    #[default]
    Front,
}

/// What the host tells the rule: the window and whether it is calm (no input in progress; ADR 0069 E5: a modal only in calm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Situation {
    /// The window.
    pub window: Window,
    /// Nothing is being typed or dragged: a modal may take the screen.
    pub calm: bool,
}

impl Default for Situation {
    fn default() -> Self {
        Self {
            window: Window::Front,
            calm: true,
        }
    }
}

/// A place that shows notifications.
pub trait Outlet: Send + Sync {
    /// The forms this outlet can show.
    fn forms(&self) -> &[Form];

    /// Shows (or, for an id that is shown already, replaces) the notification in this form. The texts of a
    /// [`Form::System`] call arrive without parameters.
    ///
    /// # Errors
    /// Any `show-failed` or other code; the facade tries the other outlets and the other forms.
    fn show(
        &self,
        form: Form,
        id: &NoteId,
        notification: &Notification,
    ) -> Result<(), Coded<NotificationCode>>;

    /// Takes it away again (closed, expired, answered elsewhere). Idempotent: an id that is not shown is no mistake.
    fn close(&self, form: Form, id: &NoteId);
}

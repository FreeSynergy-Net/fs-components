//! The codes of the notification crate, with their parameter keys. Message key: `notification-error-` and the name.
//!
//! Pattern: none, an enum with total `match`es (as `fs-settings` `codes.rs`). A code is never renamed or reused once released; a
//! new one is appended to the enum AND to `ALL` (test: each variant exactly once, and each has its text in both languages).

use fs_types::coded::ErrorCode;

/// What the notification crate found wrong, without a sentence.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotificationCode {
    /// A name (message key, parameter key, id, category, scope) does not have the form (parameters `what`, `value`).
    NameInvalid,
    /// A question without an action, or an action id given twice (parameter `cause` = `no-action`, `duplicate-action`).
    ActionsInvalid,
    /// A notification of the wrong kind was handed to `tell` or `ask` (parameter `cause` = `not-a-question`, `is-a-question`).
    WrongKind,
    /// No notification with this id is open; it was closed, never shown, or the id was not issued here (parameter `id`).
    UnknownId,
    /// An answer names an action the notification does not have: dropped and logged, never run (parameters `id`, `action`).
    UnknownAction,
    /// No outlet can show the notification in any chosen form (parameter `forms`).
    NoOutlet,
    /// The rule chose no form at all: no window and the system form is off (no parameters).
    NoForm,
    /// An outlet could not show it (parameters `form`, `kind` as in the outlet's own code).
    ShowFailed,
    /// An unknown command or a missing or wrong argument of a `notification.*` command (parameters `command`, `argument`).
    BadCommand,
    /// No action of a question that asks to remember is a word of the setting's list, or the answer asked to be remembered
    /// where no box was offered (parameters `key`, `cause` = `no-word`, `not-offered`).
    RememberInvalid,
    /// Remembering is not possible here: no settings are connected (parameter `key`). Also the yellow line `remember-unavailable`.
    RememberUnavailable,
    /// The answer counted, but writing it failed (parameters `key`, `cause` = the code of `fs-settings`, e.g. `locked`).
    RememberFailed,
}

impl ErrorCode for NotificationCode {
    const PREFIX: &'static str = "notification-error-";
    const ALL: &'static [Self] = &[
        Self::NameInvalid,
        Self::ActionsInvalid,
        Self::WrongKind,
        Self::UnknownId,
        Self::UnknownAction,
        Self::NoOutlet,
        Self::NoForm,
        Self::ShowFailed,
        Self::BadCommand,
        Self::RememberInvalid,
        Self::RememberUnavailable,
        Self::RememberFailed,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::NameInvalid => "name-invalid",
            Self::ActionsInvalid => "actions-invalid",
            Self::WrongKind => "wrong-kind",
            Self::UnknownId => "unknown-id",
            Self::UnknownAction => "unknown-action",
            Self::NoOutlet => "no-outlet",
            Self::NoForm => "no-form",
            Self::ShowFailed => "show-failed",
            Self::BadCommand => "bad-command",
            Self::RememberInvalid => "remember-invalid",
            Self::RememberUnavailable => "remember-unavailable",
            Self::RememberFailed => "remember-failed",
        }
    }
}

/// The parameter keys of the codes.
pub mod keys {
    /// What kind of name was bad (`message`, `parameter`, `id`, `action`, `category`, `scope`).
    pub const WHAT: &str = "what";
    /// The bad value itself.
    pub const VALUE: &str = "value";
    /// Why (a short word, see the code).
    pub const CAUSE: &str = "cause";
    /// A notification id.
    pub const ID: &str = "id";
    /// An action id.
    pub const ACTION: &str = "action";
    /// The forms that were chosen (`toast+system`).
    pub const FORMS: &str = "forms";
    /// One form.
    pub const FORM: &str = "form";
    /// The class of a failure of the outlet.
    pub const KIND: &str = "kind";
    /// The command id.
    pub const COMMAND: &str = "command";
    /// The argument of a command.
    pub const ARGUMENT: &str = "argument";
    /// The key of a setting.
    pub const KEY: &str = "key";
}

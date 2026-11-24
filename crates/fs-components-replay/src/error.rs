//! The error of a replay: a stable code and a sentence that names the cause.
//!
//! Pattern: Value Object (code + detail, compared by value).

use std::fmt;

/// Why a replay stopped. The text form is stable (it goes to exit messages and logs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// The body is not a valid Perform Actions body, or a value is out of range.
    Invalid,
    /// An action, source, pointer type, origin or button this module does not play.
    Unsupported,
    /// A key value outside the table.
    Key,
    /// `keyUp` of a key the driver cannot release.
    KeyUp,
    /// Control, Alt or Meta, which the driver cannot hold.
    Modifier,
    /// An element reference that matches nothing in the UI.
    ElementMissing,
    /// The driver itself failed (no graphics adapter, a picture that cannot be written).
    Driver,
}

impl Code {
    /// The text of the code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "steps-invalid",
            Self::Unsupported => "steps-unsupported",
            Self::Key => "steps-key-unsupported",
            Self::KeyUp => "steps-key-up-unsupported",
            Self::Modifier => "steps-modifier-unsupported",
            Self::ElementMissing => "steps-element-missing",
            Self::Driver => "steps-driver",
        }
    }
}

/// A code and what it is about (the action, the key, the element).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayError {
    /// What kind of failure.
    pub code: Code,
    /// The cause in one sentence, with the name of the action, key or element.
    pub detail: String,
}

impl ReplayError {
    /// A new error.
    #[must_use]
    pub fn new(code: Code, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for ReplayError {}

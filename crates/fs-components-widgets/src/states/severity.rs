//! How serious a message is: the one scale of `alert`, `toast` and the partial notice.
//!
//! Pattern: none (a value type). Each step has a name of the contract (`info`, `ok`, `warning`, `error`), a glyph and a role, so
//! the step is told by shape and word, never by colour alone (WCAG 1.4.1).

use crate::icon::Glyph;

/// The seriousness of a message.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Something the person may like to know.
    Info,
    /// Something went well.
    Ok,
    /// Something is not as it should be, nothing is lost.
    Warn,
    /// Something failed.
    Error,
}

impl Severity {
    /// All steps, mildest first.
    pub const ALL: [Severity; 4] = [Self::Info, Self::Ok, Self::Warn, Self::Error];

    /// The name in the theme contract (the variant of `alert` and `toast`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Ok => "ok",
            Self::Warn => "warning",
            Self::Error => "error",
        }
    }

    /// The glyph that tells the step apart without colour.
    pub const fn glyph(self) -> Glyph {
        match self {
            Self::Info => Glyph::Info,
            Self::Ok => Glyph::Check,
            Self::Warn => Glyph::Warning,
            Self::Error => Glyph::Close,
        }
    }

    /// The WAI-ARIA live role: a warning and an error interrupt (`alert`), the rest wait their turn (`status`).
    pub const fn role(self) -> &'static str {
        match self {
            Self::Info | Self::Ok => "status",
            Self::Warn | Self::Error => "alert",
        }
    }
}

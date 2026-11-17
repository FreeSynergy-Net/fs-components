//! Layers, profile scopes, locks and the two save targets.
//!
//! Pattern: Value objects. The ORDER of the layers is the order of [`Layer`] (derived `Ord`, lowest first); [`crate::resolve`]
//! is the only place that walks it.

use crate::names::ProgramName;

/// Where an effective value came from, lowest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Layer {
    /// The declaring block's default.
    Default,
    /// The instance default set by the admin (a lock makes it win over all, [`crate::Lock`]).
    Instance,
    /// The system: desktop language, light/dark (ADR 0057 chain, ADR 0074 line 35). Read only.
    System,
    /// The person's general profile ("for all programs").
    Person,
    /// The profile of this program ("for this program").
    Program,
}

impl Layer {
    /// The token in codes and in the configurator.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Instance => "instance",
            Self::System => "system",
            Self::Person => "person",
            Self::Program => "program",
        }
    }
}

/// Which profile a stored value belongs to.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Scope {
    /// The person's general profile.
    General,
    /// The profile of one program.
    Program(ProgramName),
}

/// The two save buttons of a program (user's word: "for this program" / "for all"; their words are MF2 texts
/// `settings-save-this-program`, `settings-save-all`, so they can be changed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// This program's profile only.
    ThisProgram,
    /// The person's general profile; this program then follows it (its own override of that key is removed, draft O5).
    /// Other programs' overrides are never touched.
    AllPrograms,
}

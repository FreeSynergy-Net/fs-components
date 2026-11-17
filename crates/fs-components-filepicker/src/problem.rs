//! The problems a chooser reports, and the answer "the person cancelled", which is no problem.
//!
//! Pattern: Value objects (code + parameters, ADR rule: errors as a code with parameters, the text comes from the MF2 files of
//! [`crate::labels`]). No text is made here.

use std::fmt;

/// The code of a problem; [`ProblemCode::as_str`] is the stable name (also the MF2 id suffix `filepicker-problem-{name}`).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProblemCode {
    /// There is no system dialog (no portal, no session bus, not Linux).
    PortalUnavailable,
    /// The portal did not start in time (parameter `limit-ms`).
    PortalTimeout,
    /// The portal refused (parameter `detail`).
    PortalDenied,
    /// Reading or looking at a file failed (parameters `path`, `detail`).
    Io,
    /// The path of the path field does not exist (parameter `path`).
    PathMissing,
    /// The path exists but cannot be read (parameter `path`).
    PathUnreadable,
    /// The path is of the wrong kind: a folder where a file is wanted or the other way round (parameter `path`).
    WrongKind,
}

impl ProblemCode {
    /// The stable name: `portal-unavailable`, `portal-timeout`, `portal-denied`, `io`, `path-missing`, `path-unreadable`,
    /// `wrong-kind`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PortalUnavailable => "portal-unavailable",
            Self::PortalTimeout => "portal-timeout",
            Self::PortalDenied => "portal-denied",
            Self::Io => "io",
            Self::PathMissing => "path-missing",
            Self::PathUnreadable => "path-unreadable",
            Self::WrongKind => "wrong-kind",
        }
    }

    /// Whether the system dialog could not be shown at all, so that the path field may take its place. A refusal by the
    /// portal is not one: it is shown, never worked around.
    pub fn means_no_dialog(self) -> bool {
        matches!(self, Self::PortalUnavailable | Self::PortalTimeout)
    }

    /// Every code (the completeness test of the texts walks it).
    pub const ALL: [Self; 7] = [
        Self::PortalUnavailable,
        Self::PortalTimeout,
        Self::PortalDenied,
        Self::Io,
        Self::PathMissing,
        Self::PathUnreadable,
        Self::WrongKind,
    ];
}

/// A problem: a code and named parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// What kind of problem.
    pub code: ProblemCode,
    /// The parameters, in the order given.
    pub params: Vec<(&'static str, String)>,
}

impl Problem {
    /// A problem without parameters.
    pub fn new(code: ProblemCode) -> Self {
        Self {
            code,
            params: Vec::new(),
        }
    }

    /// Add a parameter.
    #[must_use]
    pub fn with(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.params.push((name, value.into()));
        self
    }

    /// The value of the parameter `name`.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.as_str())
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code.as_str())?;
        for (name, value) in &self.params {
            write!(f, " {name}={value}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Problem {}

/// Why a chooser gave no path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChooserError {
    /// The person closed the dialog or pressed Cancel. Not an error of the program: nothing to show.
    Cancelled,
    /// Something went wrong; the person is told.
    Problem(Problem),
}

impl ChooserError {
    /// A problem with `code` and no parameters.
    pub fn problem(code: ProblemCode) -> Self {
        Self::Problem(Problem::new(code))
    }

    /// Whether this is only a cancel.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

impl From<Problem> for ChooserError {
    fn from(problem: Problem) -> Self {
        Self::Problem(problem)
    }
}

impl fmt::Display for ChooserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("cancelled"),
            Self::Problem(p) => p.fmt(f),
        }
    }
}

impl std::error::Error for ChooserError {}

//! The shapes the picker works with.
//!
//! Pattern: none, neutral. A person is an opaque [`PersonRef`] (the program's IRI or id); the picker never reads inside it.

/// A reference to a person, opaque to the picker. The program maps it to its own `PersonRef` (IRI or id).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonRef(String);

impl PersonRef {
    /// A reference from the program's identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier, as given.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One person the source offers. `display` and `detail` are what the source releases for this ask; `None` means it releases
/// nothing, and the picker shows "name not shared", never a guess.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// Who.
    pub reference: PersonRef,
    /// The name, if the source releases it.
    pub display: Option<String>,
    /// A second line (what matched: an address, an organisation), if the source releases it.
    pub detail: Option<String>,
}

impl Candidate {
    /// A candidate by reference alone (nothing released).
    pub fn new(reference: PersonRef) -> Self {
        Self {
            reference,
            display: None,
            detail: None,
        }
    }

    /// With the released name.
    #[must_use]
    pub fn named(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    /// With the released second line.
    #[must_use]
    pub fn detailed(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// The answer of a search: the candidates in the source's rank, and whether there are more than were asked for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandidatePage {
    /// In rank order (the source's, not ours).
    pub items: Vec<Candidate>,
    /// More exist than `limit`.
    pub more: bool,
}

/// What kind of thing a source reports (the words are the picker's, by code and parameter; ADR 0057).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// No source is set up (no people directory); the parameter names what is missing.
    SourceMissing,
    /// The source does not release this to the asker; the parameter names what.
    Denied,
    /// The source cannot be reached now; the parameter is the reason.
    Unavailable,
    /// Anything else; the parameter is the reason.
    Failed,
}

impl ErrorCode {
    /// The stable code shown with the message (`picker.source-missing`), so a report can name it.
    pub fn code(self) -> &'static str {
        match self {
            Self::SourceMissing => "picker.source-missing",
            Self::Denied => "picker.denied",
            Self::Unavailable => "picker.unavailable",
            Self::Failed => "picker.failed",
        }
    }
}

/// A source could not answer: a code and one parameter. Never silent: the picker draws it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerError {
    /// The code.
    pub code: ErrorCode,
    /// The parameter of the sentence (may be empty).
    pub param: String,
}

impl PickerError {
    /// An error of `code` with `param`.
    pub fn new(code: ErrorCode, param: impl Into<String>) -> Self {
        Self {
            code,
            param: param.into(),
        }
    }
}

impl std::fmt::Display for PickerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.code(), self.param)
    }
}

impl std::error::Error for PickerError {}

/// One person or several.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Selection {
    /// Choosing a row replaces the choice (one assignee).
    Single,
    /// Choosing a row adds it or takes it away again.
    #[default]
    Multiple,
}

/// Why a step did nothing.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No such row among the candidates.
    NoSuchCandidate,
    /// Nothing under the cursor (no candidates).
    Empty,
    /// Confirm with nobody chosen, while a choice is required.
    NothingChosen,
    /// The source is not ready (loading) or failed; there is nothing to act on.
    NotReady,
    /// "Assign to me": the source knows no own card (not signed in, or none set); the picker says so.
    NoMe,
    /// The source failed on "assign to me"; the picker shows the error.
    SourceFailed,
}

/// What a step did.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Something changed (query, choice, cursor).
    Changed,
    /// The person confirmed: the references chosen, in the order chosen.
    Confirmed(Vec<PersonRef>),
    /// The person cancelled; the caller removes the picker.
    Cancelled,
    /// The step was not possible.
    Refused(Refusal),
    /// The key is not bound to anything.
    Ignored,
}

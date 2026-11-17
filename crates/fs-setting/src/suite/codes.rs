//! The codes of the suite's failures. Message key: `settings-suite-` and the name.
//!
//! In the probe these are `fs_conformance::Coded` (its stand-in `fault`); after the fs-types lift both are `crate::coded_stub`.
//!
//! Pattern: none, an enum with total `match`es.

use fs_conformance::ErrorCode;

/// What a rule of the suite found wrong.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SuiteCode {
    /// The catalog of the program's declarations was refused (parameter `code` = the crate's code name, `key`).
    CatalogRefused,
    /// A label or description text is missing (parameters `key` = the MF2 key, `language`).
    TextMissing,
    /// A standard key is not declared (parameter `key`).
    StandardMissing,
    /// The effective value is not the expected one (parameters `key`, `step`).
    WrongValue,
    /// The value came from another layer (parameters `key`, `layer` = the expected one, `step`).
    WrongLayer,
    /// A change that must be refused went through (parameters `key`, `step`).
    NotRefused,
    /// A change that must go through was refused (parameters `key`, `code`, `step`).
    Refused,
    /// The lock is missing, or one shows where none is (parameters `key`, `step`).
    LockWrong,
    /// The lock button shows without the right, or not with it (parameters `key`, `step`).
    ButtonWrong,
    /// Live: no change told, or not exactly once (parameters `key`, `told`).
    NotTold,
    /// A bad stored value left no problem (parameter `key`).
    ProblemMissing,
    /// The frontend choice handed to the starter is wrong (parameter `step`).
    ChoiceWrong,
}

impl ErrorCode for SuiteCode {
    const PREFIX: &'static str = "settings-suite-";
    const ALL: &'static [Self] = &[
        Self::CatalogRefused,
        Self::TextMissing,
        Self::StandardMissing,
        Self::WrongValue,
        Self::WrongLayer,
        Self::NotRefused,
        Self::Refused,
        Self::LockWrong,
        Self::ButtonWrong,
        Self::NotTold,
        Self::ProblemMissing,
        Self::ChoiceWrong,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::CatalogRefused => "catalog-refused",
            Self::TextMissing => "text-missing",
            Self::StandardMissing => "standard-missing",
            Self::WrongValue => "wrong-value",
            Self::WrongLayer => "wrong-layer",
            Self::NotRefused => "not-refused",
            Self::Refused => "refused",
            Self::LockWrong => "lock-wrong",
            Self::ButtonWrong => "button-wrong",
            Self::NotTold => "not-told",
            Self::ProblemMissing => "problem-missing",
            Self::ChoiceWrong => "choice-wrong",
        }
    }
}

/// The parameter keys of the suite's codes.
pub mod keys {
    /// A setting key or an MF2 key.
    pub const KEY: &str = "key";
    /// A language tag.
    pub const LANGUAGE: &str = "language";
    /// A layer token.
    pub const LAYER: &str = "layer";
    /// A code name of the crate.
    pub const CODE: &str = "code";
    /// Which step of the rule (`1`, `2` ...).
    pub const STEP: &str = "step";
    /// How often a change was told.
    pub const TOLD: &str = "told";
}

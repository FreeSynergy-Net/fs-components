//! Names of profiles and their codes (part of the stand-in for `fs_types::profile`, see `profile.rs`).
//!
//! Pattern: Value objects + an enum of codes with total `match`es.

use crate::coded_stub::{Coded, ErrorCode};

/// What a profile operation found wrong. Message key: `profile-error-` and the name.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProfileCode {
    /// The name is empty, longer than 64 characters or holds a control character (parameter `name`).
    NameInvalid,
    /// A profile of that name exists, case ignored (parameter `name`).
    Exists,
    /// No profile of that name (parameter `name`).
    Unknown,
}

impl ErrorCode for ProfileCode {
    const PREFIX: &'static str = "profile-error-";
    const ALL: &'static [Self] = &[Self::NameInvalid, Self::Exists, Self::Unknown];

    fn as_str(self) -> &'static str {
        match self {
            Self::NameInvalid => "name-invalid",
            Self::Exists => "exists",
            Self::Unknown => "unknown",
        }
    }
}

/// The parameter key of every profile code.
pub const NAME: &str = "name";

pub(crate) fn fault(code: ProfileCode, name: &str) -> Coded<ProfileCode> {
    Coded::new(code).with_text(NAME, name)
}

/// The name a person gives a profile, in their own words ("Arbeit", "Vortrag"): trimmed, 1 to 64 characters, one line.
/// Two names are the same when they are equal ignoring case.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileName(String);

impl ProfileName {
    /// The name `text`.
    ///
    /// # Errors
    /// `name-invalid`.
    pub fn new(text: &str) -> Result<Self, Coded<ProfileCode>> {
        let t = text.trim();
        let n = t.chars().count();
        if n == 0 || n > 64 || t.chars().any(char::is_control) {
            return Err(fault(ProfileCode::NameInvalid, text));
        }
        Ok(Self(t.to_owned()))
    }

    /// The name as given.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `other` is the same name, case ignored.
    #[must_use]
    pub fn same(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

/// Which profile: the base (shown as "Allgemein" by MF2) or a named one.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProfileRef {
    /// The base; always there, never deleted, never renamed.
    Base,
    /// A named profile.
    Named(ProfileName),
}

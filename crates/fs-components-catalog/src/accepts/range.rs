//! A contract with a version need, never a program name (ADR 0045, ADR 0068: the description binds to a capability; any
//! provider of it that meets the need serves).
//!
//! One rule for every version (VERSION-EINE-QUELLE, V-3): a version is a `semver::Version` (`0.2.0`, three parts), a need is
//! an `fs_types::interface::InterfaceReq` in Cargo grammar with a comma between comparisons, written `name@<req>`:
//! `tasks@>=0.2.0, <0.3.0` (the generator's form for `0.0.x` and `0.x`) or `tasks@^1.2.0`. The reader takes only FULL needs
//! (every comparator names minor and patch; no `*`, no `0.2`, no pre-release); the compatibility rule is the one of Cargo
//! (caret), so `^0.2.0` holds 0.2.x and not 0.3.0.
//!
//! Pattern: Value object, made only by [`ContractRange::parse`] or [`ContractRange::new`] (checked).

use std::fmt;

use super::codes::{AcceptsCode, keys};
use fs_types::coded::Coded;
use fs_types::interface::InterfaceReq;
use semver::Version;

/// A capability with the need of its contract version.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContractRange {
    req: InterfaceReq,
}

impl ContractRange {
    /// Checked: the capability is a key of `capabilities.txt` form (`[a-z][a-z0-9-]*`, at most 64) and the need is full.
    ///
    /// # Errors
    /// `range-invalid` with the text form.
    pub fn new(req: InterfaceReq) -> Result<Self, Coded<AcceptsCode>> {
        let name = req.name().as_bytes();
        let key_ok = (1..=64).contains(&name.len())
            && name
                .iter()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-');
        if key_ok && req.is_full() {
            Ok(Self { req })
        } else {
            Err(Coded::new(AcceptsCode::RangeInvalid).with_text(keys::TEXT, req.to_string()))
        }
    }

    /// `tasks@>=0.2.0, <0.3.0` or `tasks@^1.2.0`: the name, `@`, a Cargo need.
    ///
    /// # Errors
    /// `range-invalid` with the text.
    pub fn parse(text: &str) -> Result<Self, Coded<AcceptsCode>> {
        let bad = || Coded::new(AcceptsCode::RangeInvalid).with_text(keys::TEXT, text);
        let req: InterfaceReq = text.parse().map_err(|_| bad())?;
        Self::new(req).map_err(|_| bad())
    }

    /// The capability key.
    #[must_use]
    pub fn capability(&self) -> &str {
        self.req.name()
    }

    /// Whether a provider with contract version `v` serves this need.
    #[must_use]
    pub fn contains(&self, v: &Version) -> bool {
        self.req.req().matches(v)
    }
}

impl fmt::Display for ContractRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.req.fmt(f)
    }
}

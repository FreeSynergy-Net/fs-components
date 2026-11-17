//! A contract with a version range, never a program name (ADR 0045, ADR 0068: the description binds to a capability; any
//! provider of it in the range serves).
//!
//! Text form `<capability> >=<from> <<below>`, e.g. `tasks >=0.2 <0.3`: half-open, so a draft contract (0.x, where a minor step
//! may break) is bound to exactly one minor line, and a 1.x contract to `>=1.2 <2.0`. No other operators, no wildcards.
//!
//! Pattern: Value object, made only by [`ContractRange::parse`] or [`ContractRange::new`] (checked).

use std::fmt;

use super::codes::{AcceptsCode, keys};
use fs_types::coded::Coded;

/// A contract version `major.minor` (`contractVersion` of `SupportReport`, `CapabilityList`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version2 {
    /// The major step.
    pub major: u32,
    /// The minor step.
    pub minor: u32,
}

impl Version2 {
    /// `"0.2"` → 0.2. No sign, no leading `v`, no third part (a contract version has two), no leading zero but `0` itself.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (a, b) = text.split_once('.')?;
        Some(Self {
            major: number(a)?,
            minor: number(b)?,
        })
    }
}

fn number(text: &str) -> Option<u32> {
    let ok = !text.is_empty()
        && text.bytes().all(|b| b.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'));
    if ok { text.parse().ok() } else { None }
}

impl fmt::Display for Version2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// A capability with the half-open range `[from, below)`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContractRange {
    capability: String,
    from: Version2,
    below: Version2,
}

impl ContractRange {
    /// Checked: the capability is a key of `capabilities.txt` form (`[a-z][a-z0-9-]*`, at most 64) and `from < below`.
    ///
    /// # Errors
    /// `range-invalid` with the text form.
    pub fn new(
        capability: &str,
        from: Version2,
        below: Version2,
    ) -> Result<Self, Coded<AcceptsCode>> {
        let b = capability.as_bytes();
        let key_ok = (1..=64).contains(&b.len())
            && b[0].is_ascii_lowercase()
            && b.iter()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-');
        if key_ok && from < below {
            Ok(Self {
                capability: capability.to_owned(),
                from,
                below,
            })
        } else {
            Err(Coded::new(AcceptsCode::RangeInvalid)
                .with_text(keys::TEXT, format!("{capability} >={from} <{below}")))
        }
    }

    /// `tasks >=0.2 <0.3`: exactly three words separated by one space.
    ///
    /// # Errors
    /// `range-invalid` with the text.
    pub fn parse(text: &str) -> Result<Self, Coded<AcceptsCode>> {
        let bad = || Coded::new(AcceptsCode::RangeInvalid).with_text(keys::TEXT, text);
        let words: Vec<&str> = text.split(' ').collect();
        let [cap, from, below] = words.as_slice() else {
            return Err(bad());
        };
        let from = from
            .strip_prefix(">=")
            .and_then(Version2::parse)
            .ok_or_else(bad)?;
        let below = below
            .strip_prefix('<')
            .and_then(Version2::parse)
            .ok_or_else(bad)?;
        Self::new(cap, from, below).map_err(|_| bad())
    }

    /// The capability key.
    #[must_use]
    pub fn capability(&self) -> &str {
        &self.capability
    }

    /// Whether a provider with contract version `v` serves this range.
    #[must_use]
    pub fn contains(&self, v: Version2) -> bool {
        self.from <= v && v < self.below
    }
}

impl fmt::Display for ContractRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} >={} <{}", self.capability, self.from, self.below)
    }
}

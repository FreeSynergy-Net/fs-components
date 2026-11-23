//! The forms a building block draws (block contract 0.4, key `forms`): `forms = ["tree@1.0"]`, one entry `name@H.N` per form of
//! the form catalog (`fs-interfaces` `schema/forms.txt`). `accepts` stays for single values; `forms` says which WHOLE shapes of
//! data (a tree, a board, tiles) the block can draw. The host compares this with the form a module offers (`fs-forms-bind`).
//!
//! Pattern: Value object (`FormRef`) + reader of one key; closed schema, a wrong entry is a problem, never ignored.

use toml::Table;

use super::codes::{AcceptsCode, keys};
use fs_types::coded::Coded;

/// One form a block draws: `name@major.minor`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormRef {
    /// The form name (kebab-case, e.g. `cards-in-columns`).
    pub name: String,
    /// Main version: changes when a channel goes or changes its meaning.
    pub major: u32,
    /// Minor version: an optional channel came.
    pub minor: u32,
}

impl FormRef {
    /// Reads `name@H.N` (both numbers needed; `@1` alone is not a spelling of the catalog).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (name, version) = text.split_once('@')?;
        let (major, minor) = version.split_once('.')?;
        let kebab = !name.is_empty()
            && !name.starts_with('-')
            && !name.ends_with('-')
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        let number = |s: &str| {
            (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                .then(|| s.parse().ok())
                .flatten()
        };
        kebab.then_some(())?;
        Some(Self {
            name: name.to_owned(),
            major: number(major)?,
            minor: number(minor)?,
        })
    }
}

impl std::fmt::Display for FormRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}.{}", self.name, self.major, self.minor)
    }
}

/// The `forms` of one block body (empty = the block draws no form, allowed); problems are pushed (`forms-invalid`).
pub(super) fn read_forms(
    block: &str,
    body: &Table,
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Vec<FormRef> {
    let Some(value) = body.get("forms") else {
        return Vec::new();
    };
    let bad = |text: &str| {
        Coded::new(AcceptsCode::FormsInvalid)
            .with_text(keys::BLOCK, block)
            .with_text(keys::TEXT, text)
    };
    let Some(list) = value.as_array() else {
        problems.push(bad("forms"));
        return Vec::new();
    };
    let mut out: Vec<FormRef> = Vec::new();
    for entry in list {
        let Some(text) = entry.as_str() else {
            problems.push(bad("forms"));
            continue;
        };
        match FormRef::parse(text) {
            // One block names a form once: two versions of one form would make the match ambiguous.
            Some(f) if out.iter().any(|o| o.name == f.name) => problems.push(bad(text)),
            Some(f) => out.push(f),
            None => problems.push(bad(text)),
        }
    }
    out
}

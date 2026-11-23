//! The forms a building block draws (block contract 0.4, key `forms`): `forms = ["tree@^1.0.0"]`, one entry `name@<req>` per form of
//! the form catalog (`fs-interfaces` `schema/forms.txt`): what the block NEEDS, so a need in Cargo grammar, never a bare
//! version. `accepts` stays for single values; `forms` says which WHOLE shapes of data (a tree, a board, tiles) the block can
//! draw. The host compares this with the form a module offers (`fs-forms-bind`).
//!
//! Pattern: Value object (`FormRef`) + reader of one key; closed schema, a wrong entry is a problem, never ignored.

use toml::Table;

use super::codes::{AcceptsCode, keys};
use fs_types::coded::Coded;
use fs_types::interface::{InterfaceReq, InterfaceVersion};

/// One form a block draws: `name@<req>`, a full need (`tree@^1.0.0`, `tiles@>=1.0.0, <2.0.0`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormRef {
    req: InterfaceReq,
}

impl FormRef {
    /// Reads `name@<req>`. The name is kebab case (no dots: forms are not interfaces of a program), the need is full
    /// (`@1` or `@1.0` alone is not a spelling of the catalog).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let req: InterfaceReq = text.parse().ok()?;
        let name = req.name();
        let kebab = !name.contains('.') && !name.ends_with('-');
        (kebab && req.is_full()).then_some(Self { req })
    }

    /// The form name (kebab case, e.g. `cards-in-columns`).
    #[must_use]
    pub fn name(&self) -> &str {
        self.req.name()
    }

    /// Whether the form `offered` (name and version of the catalog) is one this block draws.
    #[must_use]
    pub fn draws(&self, offered: &InterfaceVersion) -> bool {
        self.req.matches(offered)
    }
}

impl std::fmt::Display for FormRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.req.fmt(f)
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
            Some(f) if out.iter().any(|o| o.name() == f.name()) => problems.push(bad(text)),
            Some(f) => out.push(f),
            None => problems.push(bad(text)),
        }
    }
    out
}

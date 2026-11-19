//! Choice: one entry of a fixed list.
//!
//! Pattern: Strategy (one [`BasicField`] implementation). The rule is `Choice::pick` of the contract crate; the allowed keys come
//! from the field description (`FieldDescription.choices`), not from the type, so the trait's `problem` (which knows no list) says
//! nothing, and [`check`] asks WITH the list. The stored text is the key of the entry, trimmed; what the reader sees is the label
//! `<label>.<key>` (a translation key, ADR 0057), which the view of the input says through `Say`.

use fs_components_styles::Glyph;
use fs_types::{Choice as Pick, TypeProblem};

use crate::field::{BasicField, Segment};
use crate::problem::Problem;

/// The choice type.
#[derive(Debug, Clone, Copy)]
pub struct Choice;

const SEGMENTS: [Segment; 1] = [Segment {
    key: "key",
    weight: 1,
    required: true,
}];

/// What is wrong with `typed` against the allowed `keys`: `None` for nothing typed or a key of the list.
pub fn check(typed: &str, keys: &[String]) -> Option<Problem> {
    match Pick::pick(typed, keys) {
        Err(TypeProblem::NotInList) => Some(Problem::NotInList),
        _ => None,
    }
}

impl BasicField for Choice {
    fn key(&self) -> &'static str {
        "choice"
    }

    fn glyph(&self) -> Glyph {
        Glyph::ViewList
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, _parts: &[String]) -> Option<Problem> {
        // the list is not known here: see [`check`]
        None
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![text.trim().to_owned()]
    }

    fn join(&self, parts: &[String]) -> String {
        parts.first().map_or("", |p| p.trim()).to_owned()
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

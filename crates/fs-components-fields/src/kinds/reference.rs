//! Reference: the identity of another object, an absolute IRI kept exactly as written (AS2 `id`).
//!
//! Pattern: Strategy (one [`BasicField`] implementation). The rule is `Reference` of the contract crate (absolute IRI, something after
//! the scheme, 2048 bytes at most). The type has ONE part, the IRI. A reference to a PERSON, profile, group or organisation is
//! drawn by the `person-picker` (the catalog chooses it by the target, `contract/blocks`), not by this component; this one is the
//! reference to anything else, found through a source of candidates ([`crate::view::ReferenceInput`]).

use fs_components_styles::Glyph;
use fs_types::{BasicValue, Reference as Iri, TypeProblem};

use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The reference type.
#[derive(Debug, Clone, Copy)]
pub struct Reference;

const SEGMENTS: [Segment; 1] = [Segment {
    key: "iri",
    weight: 1,
    required: true,
}];

impl BasicField for Reference {
    fn key(&self) -> &'static str {
        "reference"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Share
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        match Iri::parse(parts.first().map_or("", String::as_str)) {
            Ok(_) => None,
            Err(TypeProblem::NotAbsolute) => Some(Problem::NotAbsolute),
            Err(_) => Some(Problem::BadReference),
        }
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

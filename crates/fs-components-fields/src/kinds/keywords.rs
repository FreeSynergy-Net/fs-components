//! Keywords: a set of short words.
//!
//! Pattern: Strategy (one [`BasicField`] implementation). The rule is `Keywords` of the contract crate (comma list, each word
//! trimmed, NFC, lower case, no duplicates, sorted; 64 characters each, 256 words). The type has ONE part, the typed list; the input
//! (`view/keywords.rs`) shows the words as chips, the stored text is the canonical one ("a, b, c").

use fs_components_styles::Glyph;
use fs_types::{BasicValue, Keywords as Set, TypeProblem};

use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The keywords type.
#[derive(Debug, Clone, Copy)]
pub struct Keywords;

const SEGMENTS: [Segment; 1] = [Segment {
    key: "words",
    weight: 1,
    required: true,
}];

/// The canonical text of a typed list, or `None` when it is not a set of keywords.
pub fn canonical(typed: &str) -> Option<String> {
    Set::parse(typed).ok().map(|k| k.canonical())
}

impl BasicField for Keywords {
    fn key(&self) -> &'static str {
        "keywords"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Filter
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        match Set::parse(parts.first().map_or("", String::as_str)) {
            Ok(_) => None,
            Err(TypeProblem::TooLong) => Some(Problem::TooMany),
            Err(_) => Some(Problem::BadKeyword),
        }
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![text.to_owned()]
    }

    fn join(&self, parts: &[String]) -> String {
        let typed = parts.first().map_or("", String::as_str);
        canonical(typed).unwrap_or_else(|| typed.trim().to_owned())
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

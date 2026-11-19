//! Flag: yes or no (basic type `flag`, canonical `true` / `false`, `yes` / `no` are read).
//!
//! Pattern: Strategy (one [`BasicField`] implementation). One part, `true`, `false` or empty (not answered yet). The rule is the
//! one of `fs-types` (`Flag`), not a copy. The input is a switch (`Entry::Switch`), not a text field.

use fs_components_styles::Glyph;
use fs_types::{BasicValue, Flag as Value};

use crate::field::{BasicField, Entry, Segment, is_blank};
use crate::problem::Problem;

/// The yes/no type.
#[derive(Debug, Clone, Copy)]
pub struct Flag;

const SEGMENTS: [Segment; 1] = [Segment {
    key: "flag",
    weight: 1,
    required: true,
}];

impl BasicField for Flag {
    fn key(&self) -> &'static str {
        "flag"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Check
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn entry(&self) -> Entry {
        Entry::Switch
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        Value::parse(&parts[0]).err().map(|_| Problem::NotYesNo)
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![self.join(&[text.to_owned()])]
    }

    fn join(&self, parts: &[String]) -> String {
        match parts.first().map(|p| Value::parse(p)) {
            Some(Ok(v)) => v.canonical(),
            _ => parts.first().map_or_else(String::new, |p| p.trim().to_owned()),
        }
    }

    fn word(&self, parts: &[String]) -> Option<&'static str> {
        match self.join(parts).as_str() {
            "true" => Some("value.flag.true"),
            "false" => Some("value.flag.false"),
            _ => None,
        }
    }

    fn mark(&self, parts: &[String]) -> Option<Glyph> {
        match self.join(parts).as_str() {
            "true" => Some(Glyph::Check),
            "false" => Some(Glyph::Close),
            _ => None,
        }
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(t: &str) -> Vec<String> {
        vec![t.to_owned()]
    }

    #[test]
    fn yes_and_no_are_read_and_written_canonically() {
        assert_eq!(Flag.join(&p("Yes")), "true");
        assert_eq!(Flag.join(&p(" no ")), "false");
        assert_eq!(Flag.split("TRUE"), p("true"));
        assert_eq!(Flag.join(&p("")), "");
    }

    #[test]
    fn anything_else_is_a_problem_and_empty_is_not() {
        assert_eq!(Flag.problem(&p("maybe")), Some(Problem::NotYesNo));
        assert_eq!(Flag.problem(&p("true")), None);
        assert_eq!(Flag.problem(&p("")), None);
        assert_eq!(Flag.problem(&[]), None);
    }
}

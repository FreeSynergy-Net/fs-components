//! Whole numbers: `integer` (signed, 64 bit) and `whole` (from 0 up), typed into a number field.
//!
//! Pattern: Strategy (two [`BasicField`] implementations over one rule). The rule is the one of `fs-types` (`Integer`, `Whole`:
//! optional sign, ASCII digits, the range of `i64` / `u64`); the joined form is its canonical decimal text (no `+`, no leading
//! zeros, no `-0`). The input is a text field that takes digits and steps with the arrow keys (`Entry::Number`).

use fs_components_styles::Glyph;
use fs_types::{BasicValue, Integer as Signed, TypeProblem, Whole as Unsigned};

use crate::field::{BasicField, Entry, Segment, is_blank};
use crate::problem::Problem;

/// The signed whole number type.
#[derive(Debug, Clone, Copy)]
pub struct Integer;

/// The whole number type from 0 up.
#[derive(Debug, Clone, Copy)]
pub struct Whole;

const SEGMENTS: [Segment; 1] = [Segment {
    key: "number",
    weight: 1,
    required: true,
}];

fn problem_of(e: TypeProblem) -> Problem {
    match e {
        TypeProblem::OutOfRange => Problem::OutOfRange,
        _ => Problem::NotANumber,
    }
}

fn join_with<T: BasicValue>(parts: &[String]) -> String {
    let Some(text) = parts.first() else {
        return String::new();
    };
    T::parse(text).map_or_else(|_| text.trim().to_owned(), |v| v.canonical())
}

impl BasicField for Integer {
    fn key(&self) -> &'static str {
        "integer"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Sort
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn entry(&self) -> Entry {
        Entry::Number { signed: true }
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        Signed::parse(&parts[0]).err().map(problem_of)
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![join_with::<Signed>(&[text.to_owned()])]
    }

    fn join(&self, parts: &[String]) -> String {
        join_with::<Signed>(parts)
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

impl BasicField for Whole {
    fn key(&self) -> &'static str {
        "whole"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Sort
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn entry(&self) -> Entry {
        Entry::Number { signed: false }
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        Unsigned::parse(&parts[0]).err().map(problem_of)
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![join_with::<Unsigned>(&[text.to_owned()])]
    }

    fn join(&self, parts: &[String]) -> String {
        join_with::<Unsigned>(parts)
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

/// `text` moved by `delta` steps: from nothing the step starts at 0; `whole` never goes below 0; the end of the range stays
/// (never wraps). `None`: `text` is not a number of this type.
pub fn stepped(signed: bool, text: &str, delta: i32) -> Option<String> {
    let now: i128 = if text.trim().is_empty() {
        0
    } else if signed {
        i128::from(Signed::parse(text).ok()?.get())
    } else {
        i128::from(Unsigned::parse(text).ok()?.get())
    };
    let (low, high) = if signed {
        (i128::from(i64::MIN), i128::from(i64::MAX))
    } else {
        (0, i128::from(u64::MAX))
    };
    Some((now + i128::from(delta)).clamp(low, high).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(t: &str) -> Vec<String> {
        vec![t.to_owned()]
    }

    #[test]
    fn the_canonical_form_has_no_plus_zeros_or_negative_zero() {
        assert_eq!(Integer.join(&p("+007")), "7");
        assert_eq!(Integer.join(&p("-0")), "0");
        assert_eq!(Integer.join(&p(" -12 ")), "-12");
        assert_eq!(Whole.join(&p("0042")), "42");
        assert_eq!(Integer.join(&p("")), "");
    }

    #[test]
    fn problems_name_the_reason() {
        assert_eq!(Integer.problem(&p("12a")), Some(Problem::NotANumber));
        assert_eq!(Integer.problem(&p("-")), Some(Problem::NotANumber));
        assert_eq!(
            Integer.problem(&p("9223372036854775808")),
            Some(Problem::OutOfRange)
        );
        assert_eq!(Integer.problem(&p("-9223372036854775808")), None);
        assert_eq!(Whole.problem(&p("-1")), Some(Problem::OutOfRange));
        assert_eq!(Whole.problem(&p("-0")), None);
        assert_eq!(Whole.problem(&p("")), None);
    }

    #[test]
    fn stepping_starts_at_zero_and_stays_in_the_range() {
        assert_eq!(stepped(true, "", -1).as_deref(), Some("-1"));
        assert_eq!(stepped(false, "", -1).as_deref(), Some("0"));
        assert_eq!(stepped(false, "0", -1).as_deref(), Some("0"));
        assert_eq!(stepped(true, "41", 1).as_deref(), Some("42"));
        assert_eq!(
            stepped(true, "9223372036854775807", 1).as_deref(),
            Some("9223372036854775807")
        );
        assert_eq!(stepped(true, "x", 1), None);
    }
}

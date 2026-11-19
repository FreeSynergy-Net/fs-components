//! Many-line text: text with line breaks (JSContact/RFC 5545 `TEXT`), typed into a text area.
//!
//! Pattern: Strategy (one [`BasicField`] implementation). One part. The joined form has `\n` line ends (a pasted `\r\n` or a
//! lone `\r` becomes `\n`) and no blank lines or spaces at its end. The compact line and the list cell show the first line that
//! has text, and a mark when more follows, so a row stays one line.

use fs_components_styles::Glyph;

use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The many-line text type.
#[derive(Debug, Clone, Copy)]
pub struct Multiline;

/// What a one-line view shows after the first line when more follows.
pub const MORE: &str = " \u{2026}";

const SEGMENTS: [Segment; 1] = [Segment {
    key: "text",
    weight: 1,
    required: true,
}];

/// `text` with `\n` line ends.
fn unix(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

impl BasicField for Multiline {
    fn key(&self) -> &'static str {
        "multiline"
    }

    fn glyph(&self) -> Glyph {
        Glyph::FileText
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn multiline(&self) -> bool {
        true
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        let text = parts.first()?;
        text.chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            .then_some(Problem::ControlCharacter)
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![unix(text)]
    }

    fn join(&self, parts: &[String]) -> String {
        if is_blank(parts) {
            return String::new();
        }
        unix(&parts[0]).trim_end().to_string()
    }

    fn compact(&self, parts: &[String]) -> String {
        let text = self.join(parts);
        let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
        let Some(first) = lines.next() else {
            return String::new();
        };
        if lines.next().is_some() {
            format!("{first}{MORE}")
        } else {
            first.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(text: &str) -> Vec<String> {
        vec![text.to_string()]
    }

    #[test]
    fn line_ends_are_unix_and_the_end_is_trimmed() {
        assert_eq!(Multiline.join(&m("a\r\nb\rc\n\n  ")), "a\nb\nc");
        assert_eq!(Multiline.join(&m("  \n ")), "");
        assert_eq!(Multiline.split("a\r\nb"), m("a\nb"));
        assert_eq!(
            Multiline.join(&m("  indented\nsecond")),
            "  indented\nsecond"
        );
    }

    #[test]
    fn one_line_views_show_the_first_line_and_a_mark() {
        assert_eq!(Multiline.compact(&m("only")), "only");
        assert_eq!(
            Multiline.compact(&m("\n  first \nsecond")),
            "first \u{2026}"
        );
        assert_eq!(Multiline.cell(&m("one\n\n")), "one");
        assert_eq!(Multiline.compact(&m("")), "");
    }

    #[test]
    fn line_breaks_and_tabs_are_text_other_controls_are_not() {
        assert_eq!(Multiline.problem(&m("a\nb\tc")), None);
        assert_eq!(
            Multiline.problem(&m("a\u{7}b")),
            Some(Problem::ControlCharacter)
        );
        assert_eq!(
            Multiline.problem(&m("a\u{0}")),
            Some(Problem::ControlCharacter)
        );
        assert_eq!(Multiline.problem(&[]), None);
    }
}

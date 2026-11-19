//! What a value is drawn as: text, or a mark (the check and the cross of a yes/no value).
//!
//! Pattern: none — one pure function the views of display, compact and cell share, so the rule is written once and tested alone.
//! A line that can say words (the display has `Say`) says the value in words (`BasicField::word`); a line that has no `Say`
//! (compact, cell) draws the mark (`BasicField::mark`) when the type has one; every other value is its text.

use fs_components_styles::Glyph;

use super::Say;
use crate::field::BasicField;

/// How a value is drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Shown {
    /// An icon in place of text.
    Mark(Glyph),
    /// A line of text.
    Text(String),
}

/// The value `parts` of `field` as it is drawn; `text` is the plain text of the line; `say`: the words of the program, if the line
/// has them (`None`: a compact line or a cell).
pub(super) fn shown(
    field: &dyn BasicField,
    parts: &[String],
    text: &str,
    say: Option<&dyn Say>,
) -> Shown {
    match say {
        Some(say) => Shown::Text(
            field
                .word(parts)
                .map_or_else(|| text.to_owned(), |key| say.say(key)),
        ),
        None => field
            .mark(parts)
            .map_or_else(|| Shown::Text(text.to_owned()), Shown::Mark),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::Kind;

    fn p(t: &str) -> Vec<String> {
        vec![t.to_owned()]
    }

    fn words(k: &str) -> String {
        format!("<{k}>")
    }

    #[test]
    fn a_line_with_words_says_a_flag_in_words_and_a_line_without_draws_the_mark() {
        let f = Kind::Flag.field();
        assert_eq!(
            shown(f, &p("true"), "true", Some(&words)),
            Shown::Text("<value.flag.true>".to_owned())
        );
        assert_eq!(
            shown(f, &p("false"), "false", Some(&words)),
            Shown::Text("<value.flag.false>".to_owned())
        );
        assert_eq!(
            shown(f, &p("true"), "true", None),
            Shown::Mark(Glyph::Check)
        );
        assert_eq!(
            shown(f, &p("false"), "false", None),
            Shown::Mark(Glyph::Close)
        );
    }

    #[test]
    fn every_other_value_is_its_text() {
        let n = Kind::Integer.field();
        assert_eq!(
            shown(n, &p("42"), "42", Some(&words)),
            Shown::Text("42".to_owned())
        );
        assert_eq!(shown(n, &p("42"), "42", None), Shown::Text("42".to_owned()));
        let f = Kind::Flag.field();
        assert_eq!(shown(f, &p(""), "", None), Shown::Text(String::new()));
    }
}

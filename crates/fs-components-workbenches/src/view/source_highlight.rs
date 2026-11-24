//! The colours of a source text: the [`Language`] of the editor contract behind Blinc's `SyntaxHighlighter`.
//!
//! Pattern: Adapter (a `Language` to Blinc's `SyntaxHighlighter`) — the language says which range is of which kind
//! ([`TokenKind`]), the [`SyntaxPalette`] says which colour a kind has. The colours come from the theme (Blinc's colour
//! tokens), none is written here (ADR 0074).
//!
//! Limit: Blinc asks for one line at a time and keeps no state between the lines, so the state of a line (an open code
//! fence) reaches only the lines of one `highlight` call (the minimap asks for the whole text). See the report of E3.

use std::sync::Arc;

use blinc_core::Color;
use blinc_layout::styled_text::{StyledLine, StyledText, TextSpan};
use blinc_layout::syntax::{SyntaxHighlighter, TokenRule};
use blinc_theme::{ColorToken, ThemeState};
use fs_components_editor::{Language, LineState, TokenKind};

/// The colour of every [`TokenKind`], one per kind.
///
/// Pattern: Value object — a table from a kind to a colour; made from the theme or given as it is (tests).
#[derive(Clone, Debug)]
pub(crate) struct SyntaxPalette {
    colours: [Color; KINDS.len()],
}

/// Every kind once, in the order of the palette.
const KINDS: [TokenKind; 16] = [
    TokenKind::Plain,
    TokenKind::Keyword,
    TokenKind::Type,
    TokenKind::Function,
    TokenKind::String,
    TokenKind::Number,
    TokenKind::Comment,
    TokenKind::Punct,
    TokenKind::Key,
    TokenKind::Heading,
    TokenKind::Emphasis,
    TokenKind::Strong,
    TokenKind::Link,
    TokenKind::Code,
    TokenKind::Fence,
    TokenKind::Invalid,
];

/// The theme token each kind takes (the theme of Blinc has no token of its own for syntax, see the report of E3).
fn token_of(kind: TokenKind) -> ColorToken {
    match kind {
        TokenKind::Plain | TokenKind::Emphasis | TokenKind::Strong => ColorToken::TextPrimary,
        TokenKind::Keyword | TokenKind::Heading => ColorToken::Primary,
        TokenKind::Type | TokenKind::Key => ColorToken::Info,
        TokenKind::Function => ColorToken::Accent,
        TokenKind::String | TokenKind::Code => ColorToken::Success,
        TokenKind::Number => ColorToken::Warning,
        TokenKind::Comment | TokenKind::Fence => ColorToken::TextTertiary,
        TokenKind::Punct => ColorToken::TextSecondary,
        TokenKind::Link => ColorToken::TextLink,
        TokenKind::Invalid => ColorToken::Error,
    }
}

impl SyntaxPalette {
    /// The palette of the theme that is set now (the theme must be initialised).
    pub(crate) fn from_theme() -> Self {
        let theme = ThemeState::get();
        Self::with(|kind| theme.color(token_of(kind)))
    }

    /// A palette from a function (kind to colour).
    pub(crate) fn with(colour: impl Fn(TokenKind) -> Color) -> Self {
        Self {
            colours: KINDS.map(colour),
        }
    }

    /// The colour of `kind`.
    pub(crate) fn colour(&self, kind: TokenKind) -> Color {
        let at = KINDS.iter().position(|k| *k == kind).unwrap_or(0);
        self.colours[at]
    }
}

/// A [`Language`] with a [`SyntaxPalette`] as Blinc's highlighter.
pub(crate) struct BlincHighlight {
    language: Arc<dyn Language>,
    palette: SyntaxPalette,
}

impl BlincHighlight {
    /// The highlighter of `language` in the colours of `palette`.
    pub(crate) fn new(language: Arc<dyn Language>, palette: SyntaxPalette) -> Self {
        Self { language, palette }
    }

    fn styled(&self, state: Option<LineState>, line: &str) -> (StyledLine, Option<LineState>) {
        let (tokens, after) = self.language.line(state, line);
        let spans = tokens
            .iter()
            .filter(|t| t.start < t.end && t.end <= line.len())
            .map(|t| {
                let span = TextSpan::new(
                    t.start,
                    t.end,
                    self.palette.colour(t.kind),
                    matches!(t.kind, TokenKind::Strong | TokenKind::Heading),
                );
                span.with_italic(t.kind == TokenKind::Emphasis)
            })
            .collect();
        (StyledLine::new(line, spans), after)
    }
}

impl SyntaxHighlighter for BlincHighlight {
    /// No regex rules: the language colours the lines itself ([`highlight`](SyntaxHighlighter::highlight)).
    fn token_rules(&self) -> &[TokenRule] {
        &[]
    }

    fn default_color(&self) -> Color {
        self.palette.colour(TokenKind::Plain)
    }

    fn highlight(&self, text: &str) -> StyledText {
        let mut state = None;
        let mut lines = Vec::new();
        for line in text.split('\n') {
            let (styled, after) = self.styled(state, line);
            state = after;
            lines.push(styled);
        }
        StyledText::from_lines(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fs_components_editor::language::FakeLanguage;

    fn red_for_keyword() -> BlincHighlight {
        let pal = SyntaxPalette::with(|k| {
            if k == TokenKind::Keyword {
                Color::rgba(1.0, 0.0, 0.0, 1.0)
            } else if k == TokenKind::Code {
                Color::rgba(0.0, 1.0, 0.0, 1.0)
            } else {
                Color::rgba(0.5, 0.5, 0.5, 1.0)
            }
        });
        BlincHighlight::new(Arc::new(FakeLanguage), pal)
    }

    #[test]
    fn a_keyword_gets_the_colour_of_its_kind() {
        let styled = red_for_keyword().highlight("fn x");
        let first = &styled.lines[0].spans[0];
        assert_eq!((first.start, first.end), (0, 2));
        assert_eq!(first.color, Color::rgba(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn the_state_of_a_fence_runs_over_the_lines_of_one_call() {
        let styled = red_for_keyword().highlight("```\nfn\n```\nfn");
        assert_eq!(styled.lines.len(), 4);
        // inside the fence `fn` is code (green), after it a keyword (red) again
        assert_eq!(
            styled.lines[1].spans[0].color,
            Color::rgba(0.0, 1.0, 0.0, 1.0)
        );
        assert_eq!(
            styled.lines[3].spans[0].color,
            Color::rgba(1.0, 0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn an_empty_text_is_one_empty_line() {
        let styled = red_for_keyword().highlight("");
        assert_eq!(styled.lines.len(), 1);
        assert_eq!(styled.lines[0].spans.len(), 0);
    }

    #[test]
    fn the_theme_gives_every_kind_a_colour_and_keywords_differ_from_comments() {
        blinc_theme::ThemeState::init_default();
        let pal = SyntaxPalette::from_theme();
        assert_ne!(
            pal.colour(TokenKind::Keyword),
            pal.colour(TokenKind::Comment)
        );
        assert_ne!(pal.colour(TokenKind::String), pal.colour(TokenKind::Number));
    }
}

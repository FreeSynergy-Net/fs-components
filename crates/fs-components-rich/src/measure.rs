//! Measuring text the way it is drawn: the faces of the surface, line steps, wrap points, widths of prefixes.
//!
//! Pattern: Adapter over Blinc's text measurer (`measure_text_with_options`, `text_line_spans`) — the one place that asks it; the
//! geometry (caret, click) and the drawing both use these faces, so they agree.

use blinc_layout::text_measure::{
    LineSpan, TextLayoutOptions, line_height_px, measure_text_with_options, text_line_spans,
};

/// How a run of text is set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    /// Font size in pixels.
    pub size: f32,
    /// Bold.
    pub bold: bool,
    /// The monospace font (code).
    pub mono: bool,
    /// Line height multiplier.
    pub line: f32,
}

impl Face {
    fn options(self) -> TextLayoutOptions {
        let mut o = TextLayoutOptions::new().with_line_height(self.line);
        if self.bold {
            o = o.bold();
        }
        if self.mono {
            o = o.monospace();
        }
        o
    }

    /// The height of one line.
    #[must_use]
    pub fn line_height(self) -> f32 {
        line_height_px(self.size, &self.options())
    }

    /// The width of `text` on one line.
    #[must_use]
    pub fn width(self, text: &str) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        measure_text_with_options(text, self.size, &self.options()).width
    }

    /// Where `text` wraps in a column `max` px wide: the byte range of each line (empty text is one empty line). A trailing
    /// space stays with the line before it, so every byte of the text is on exactly one line.
    #[must_use]
    pub fn lines(self, text: &str, max: f32) -> Vec<(usize, usize)> {
        if text.is_empty() {
            return vec![(0, 0)];
        }
        let spans: Vec<LineSpan> = text_line_spans(
            text,
            self.size,
            &self.options().with_max_width(max.max(1.0)),
        );
        if spans.is_empty() {
            return vec![(0, text.len())];
        }
        let mut out = Vec::with_capacity(spans.len());
        for (k, s) in spans.iter().enumerate() {
            let end = spans.get(k + 1).map_or(text.len(), |n| n.start);
            out.push((if k == 0 { 0 } else { s.start }, end.max(s.start)));
        }
        out
    }
}

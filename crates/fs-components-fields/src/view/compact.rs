//! Compact: the value in one line of running text, with the icon of its type.
//!
//! Pattern: Adapter over Blinc's row, text and svg elements; text and spacing from the [`Look`].

use blinc_layout::div::div;

use super::Look;
use super::link::{Follow, linked};
use super::shown::{Shown, shown};
use crate::kinds::Kind;

/// The compact component of `kind` for `parts`: icon, then the value in one line. Empty parts give
/// an empty line of the same height, so a row does not jump.
pub fn compact(
    kind: Kind,
    parts: &[String],
    look: &Look,
) -> impl fs_components_widgets::Element + use<> {
    compact_with(kind, parts, look, &None)
}

/// [`compact`] with what follows a link given by the caller (`None`: the system's program opens it).
pub fn compact_with(
    kind: Kind,
    parts: &[String],
    look: &Look,
    to: &Follow,
) -> impl fs_components_widgets::Element + use<> {
    let field = kind.field();
    let pal = look.palette();
    let s = look.style;
    let line = field.compact(parts);
    div()
        .class("fs-field-compact")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .h(s.text.body_line)
        .overflow_clip()
        .child(look.icon(field.glyph(), Some(s.text.body), pal.text_weak))
        .child(match shown(field, parts, &line, None) {
            Shown::Mark(glyph) => div().child(look.icon(glyph, Some(s.text.body), pal.text)),
            Shown::Text(text) => match field.href(parts) {
                Some(href) => linked(
                    look,
                    "field-link-open-compact",
                    &text,
                    s.text.body,
                    &href,
                    to,
                ),
                None => div().child(look.text(text, s.text.body, pal.text)),
            },
        })
}

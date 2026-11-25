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
    row(look, kind).child(match shown(field, parts, &line, None) {
        Shown::Mark(glyph) => div().child(look.icon(glyph, None, pal.text)),
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

/// The row of a compact component: the icon of `kind`, then what the caller adds.
fn row(look: &Look, kind: Kind) -> blinc_layout::div::Div {
    let s = look.style;
    div()
        .class("fs-field-compact")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .h(s.text.body_line)
        .overflow_clip()
        .child(look.icon(
            kind.field().glyph(),
            Some(s.text.body),
            look.palette().text_weak,
        ))
}

/// The compact component of `kind` with a text the caller chose (a value cut to fit its room, ending in an ellipsis): the same icon,
/// colour and size as [`compact`], so a cut fact looks like the others.
pub fn compact_text(
    kind: Kind,
    text: &str,
    look: &Look,
) -> impl fs_components_widgets::Element + use<> {
    let s = look.style;
    row(look, kind).child(div().child(look.text(text, s.text.body, look.palette().text)))
}

/// The width [`compact`] takes for `parts` (icon, gap, text in the body size): the one measure of it, for whoever shares a room
/// between components.
#[must_use]
pub fn compact_width(kind: Kind, parts: &[String], look: &Look) -> f32 {
    let field = kind.field();
    let s = look.style;
    let line = field.compact(parts);
    let value = match shown(field, parts, &line, None) {
        Shown::Mark(_) => s.icons.size,
        Shown::Text(text) => text_width(&text, look),
    };
    s.text.body + s.spacing.xs + value
}

/// The width of `text` in the body size of `look`.
#[must_use]
pub fn text_width(text: &str, look: &Look) -> f32 {
    blinc_layout::text_measure::measure_text(text, look.style.text.body).width
}

//! Display: the value to read, every part with its label.
//!
//! Pattern: Adapter over Blinc's column, row and text elements; words from [`Say`], measures from
//! the [`Look`]. Parts that are empty are left out; a value with no part shows nothing.

use blinc_layout::div::div;

use super::link::{Follow, linked};
use super::shown::{Shown, shown};
use super::{Look, Say};
use crate::kinds::Kind;

/// The display component of `kind` for `parts`: the icon and the name of the type, then one
/// row per given part, label (weak, small) and content.
pub fn display(
    kind: Kind,
    parts: &[String],
    look: &Look,
    say: &dyn Say,
) -> impl fs_components_widgets::Element + use<> {
    display_with(kind, parts, look, say, &None)
}

/// [`display`] with what follows a link given by the caller (`None`: the system's program opens it).
pub fn display_with(
    kind: Kind,
    parts: &[String],
    look: &Look,
    say: &dyn Say,
    to: &Follow,
) -> impl fs_components_widgets::Element + use<> {
    let field = kind.field();
    let pal = look.palette();
    let s = look.style;
    let mut column = div()
        .class("fs-field-display")
        .flex_col()
        .gap_px(s.spacing.xs);
    column = column.child(
        div()
            .flex_row()
            .items_center()
            .gap_px(s.spacing.xs)
            .child(look.icon(field.glyph(), Some(s.text.body), pal.text_weak))
            .child(look.text(
                say.say(&format!("field.{}", field.key())),
                s.text.caption,
                pal.text_weak,
            )),
    );
    for (i, segment) in field.segments().iter().enumerate() {
        let content = parts.get(i).map_or("", |p| p.trim());
        if content.is_empty() {
            continue;
        }
        let shown = match shown(field, parts, content, Some(say)) {
            Shown::Text(text) => text,
            Shown::Mark(_) => content.to_owned(),
        };
        column = column.child(
            div()
                .flex_row()
                .items_center()
                .gap_px(s.spacing.s)
                .min_h(s.text.body_line)
                .child(div().w(s.text.body * 10.0).child(look.text(
                    say.say(&format!("label.{}.{}", field.key(), segment.key)),
                    s.text.caption,
                    pal.text_weak,
                )))
                .child(match field.href(parts) {
                    Some(href) => linked(look, "field-link-open", content, s.text.body, &href, to),
                    None => div().child(look.text(shown, s.text.body, pal.text)),
                }),
        );
    }
    column
}

//! Small drawing parts shared by the month and the time views.
//!
//! Pattern: View function — pure functions from data and the [`Look`] to elements; the colour knowledge is the value object
//! [`ContentLook`] of the styles (no colour rule of its own here).

use blinc_layout::div::{Div, div};
use fs_components_fields::Look;
use fs_components_styles::{ContentLook, Rgba};

use crate::layout::Rect;
use crate::model::{GridItem, Tone};

/// The style colour of a role (`Tone`), `None` for the quiet look (no colour).
fn role_colour(look: &Look, tone: Tone) -> Option<Rgba> {
    let p = look.palette();
    match tone {
        Tone::Accent => Some(p.accent_text),
        Tone::Success => Some(p.success),
        Tone::Error => Some(p.error),
        Tone::Neutral => None,
    }
}

/// How an item looks: its own content colour as given, else its role colour, else the quiet look of the style (KG7).
pub fn item_look(look: &Look, item: &GridItem) -> ContentLook {
    let p = look.palette();
    match item.colour.or_else(|| role_colour(look, item.tone)) {
        Some(c) => ContentLook::of(c, p, p.content_tint),
        None => ContentLook::quiet(p, p.content_tint),
    }
}

/// An empty box at `r` inside a relatively placed parent.
pub fn at(r: Rect) -> Div {
    div().absolute().left(r.x).top(r.y).w(r.w).h(r.h)
}

/// The stripe of an item: its mark, unchanged (the securing outline is the border of what holds it).
pub fn stripe(c: &ContentLook, w: f32, h: f32) -> Div {
    div().w(w).h(h).flex_shrink_0().bg(Look::color(c.mark))
}

/// A bar or block for an item: the tinted ground, a stripe in the colour, the ink; a border when selected.
pub fn chip(look: &Look, r: Rect, c: &ContentLook, title: &str, selected: bool) -> Div {
    let (s, pal) = (look.style, look.palette());
    let edge = if selected {
        Some((2.0, pal.accent_text))
    } else {
        c.outline.map(|o| (1.0, o))
    };
    let mut d = at(r)
        .overflow_clip()
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .rounded(s.radii.control.min(4.0))
        .bg(Look::color(c.ground))
        .child(stripe(c, 4.0, r.h))
        .child(look.text(title, s.text.caption, c.ink));
    if let Some((w, col)) = edge {
        d = d.border(w, Look::color(col));
    }
    d.cursor(blinc_layout::element::CursorStyle::Pointer)
}

/// The number of a day; today gets a filled accent circle (and the number stays readable on it).
pub fn day_number(look: &Look, n: u8, today: bool, weak: bool) -> Div {
    let (s, pal) = (look.style, look.palette());
    let size = s.text.caption * 1.8;
    let color = if today {
        pal.on_accent
    } else if weak {
        pal.text_weak
    } else {
        pal.text
    };
    let mut d = div()
        .w(size)
        .h(size)
        .flex_row()
        .items_center()
        .justify_center()
        .rounded(size / 2.0);
    if today {
        d = d.bg(Look::color(pal.accent));
    }
    d.child(look.text(n.to_string(), s.text.caption, color))
}

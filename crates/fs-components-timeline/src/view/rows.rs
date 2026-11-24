//! Bars with progress, milestones, and the line for today.
//!
//! Pattern: View function — rectangles from `layout`. A click on an item chooses it; the chosen
//! one has an accent border (not colour alone: the border is thicker).

use blinc_core::{Brush, Path};
use blinc_layout::canvas::canvas;
use blinc_layout::div::{Div, div};
use fs_components_fields::Look;

use super::parts::{at, tone_color};
use crate::bars::{Fill, state_look};
use crate::handle::Timeline;
use crate::layout::{Rect, Rows, Scale, item_rect, outside_label_x, progress_rect};
use crate::model::{Item, Kind};

/// An element for item number `row`, placed `top` pixels below the top of the body.
pub fn item(
    t: &Timeline,
    it: &Item,
    row: usize,
    scale: &Scale,
    g: Rows,
    top: f32,
    selected: bool,
) -> Div {
    let (look, s, pal) = (t.look, t.look.style, t.look.palette());
    let mut r = item_rect(it, row, scale, g);
    r.y += top;
    let c = tone_color(&look, it.tone);
    // A state with the fill `Outline` (cancelled) draws the frame only: the state is told without colour as well.
    let outline = it
        .state
        .as_deref()
        .is_some_and(|s| state_look(s).fill == Fill::Outline);
    let (me, key) = (t.clone(), it.key.clone());
    let edge = if selected {
        (2.0, pal.accent_text)
    } else {
        (1.0, c)
    };
    let base = at(r)
        .id(format!("tl-item-{}", it.key))
        .cursor(blinc_layout::element::CursorStyle::Pointer)
        .on_click(move |_| {
            me.select(&key);
        });
    match it.kind {
        Kind::Bar { progress, .. } => {
            // A title that does not fit stands right of the bar (`outside_title`), not cut off inside it.
            let inside = outside_label_x(r, t.title_width(&it.title), s.spacing.xs).is_none();
            let done = progress_rect(
                Rect {
                    x: 0.0,
                    y: 0.0,
                    ..r
                },
                progress,
            );
            base.overflow_clip()
                .rounded(s.radii.control.min(4.0))
                .bg(Look::color(c.alpha(if outline { 0x00 } else { 0x30 })))
                .border(edge.0, Look::color(edge.1))
                .child(
                    at(done)
                        .id(format!("tl-done-{}", it.key))
                        .bg(Look::color(c.alpha(0x90)))
                        .pointer_events_none(),
                )
                .children(inside.then(|| {
                    at(Rect {
                        x: s.spacing.xs,
                        y: 0.0,
                        w: r.w,
                        h: r.h,
                    })
                    .flex_row()
                    .items_center()
                    .pointer_events_none()
                    .child(look.text(&it.title, s.text.caption, pal.text))
                }))
        }
        Kind::Milestone { .. } => {
            // A diamond is a square turned by 45 degrees; the title stands to its right.
            let side = r.h * 0.72;
            let inset = (r.h - side) / 2.0;
            base.child(
                at(Rect {
                    x: inset,
                    y: inset,
                    w: side,
                    h: side,
                })
                .rotate_deg(45.0)
                .bg(Look::color(c))
                .border(edge.0, Look::color(edge.1))
                .pointer_events_none(),
            )
            .child(
                // No width given: the title takes what its glyphs need, whatever they are (W, M, CJK).
                div()
                    .id(format!("tl-title-{}", it.key))
                    .absolute()
                    .left(r.h + s.spacing.xs)
                    .top(0.0)
                    .h(r.h)
                    .flex_row()
                    .items_center()
                    .pointer_events_none()
                    .child(look.text(&it.title, s.text.caption, pal.text)),
            )
        }
    }
}

/// The line for today: a thin full-height bar, in the error colour, marked by `tl-today`.
pub fn today_line(t: &Timeline, x: f32, total_h: f32) -> Div {
    let pal = t.look.palette();
    at(Rect {
        x,
        y: 0.0,
        w: 2.0,
        h: total_h,
    })
    .id("tl-today")
    .bg(Look::color(pal.error))
    .pointer_events_none()
}

/// The mark for a today off the axis, on a row `width` wide and `high` tall above the scroll box:
/// a drawn triangle towards it and the word `timeline.today`, at the left or right rim.
pub fn today_edge(t: &Timeline, width: f32, high: f32, after: bool) -> Div {
    let (look, s, pal) = (t.look, t.look.style, t.look.palette());
    let color = Look::color(pal.error);
    let side = s.text.caption;
    // The triangle is drawn, not typed: the fonts of the styles have no arrow glyphs.
    let arrow = canvas(move |ctx, _| {
        let (x0, x1) = if after { (0.0, side) } else { (side, 0.0) };
        let p = Path::new()
            .move_to(x0, 0.0)
            .line_to(x1, side / 2.0)
            .line_to(x0, side)
            .close();
        ctx.fill_path(&p, Brush::Solid(color));
    })
    .redraw_while(|| false)
    .w(side)
    .h(side);
    let word = look.text(t.say.say("timeline.today"), s.text.caption, pal.error);
    let mark = div()
        .id("tl-today-edge")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .padding_x_px(s.spacing.xs);
    let mark = if after {
        mark.child(word)
            .child(div().id("tl-today-arrow").w(side).h(side).child(arrow))
    } else {
        mark.child(div().id("tl-today-arrow").w(side).h(side).child(arrow))
            .child(word)
    };
    let row = div()
        .id("tl-today-row")
        .flex_row()
        .items_center()
        .w(width)
        .h(high);
    if after {
        row.justify_end().child(mark)
    } else {
        row.child(mark)
    }
}

/// The heading of a lane at body row `row`: a faint band over the whole width and the name, `tl-lane-<name>`.
pub fn lane(t: &Timeline, name: &str, row: usize, width: f32, g: Rows, top: f32) -> Div {
    let (look, s, pal) = (t.look, t.look.style, t.look.palette());
    at(Rect {
        x: 0.0,
        y: top + row as f32 * g.row,
        w: width,
        h: g.row,
    })
    .id(format!("tl-lane-{name}"))
    .bg(Look::color(pal.text_weak.alpha(0x18)))
    .flex_row()
    .items_center()
    .padding_x_px(s.spacing.xs)
    .pointer_events_none()
    .child(look.text(name, s.text.caption, pal.text_weak))
}

/// The title of a bar that does not fit in it, right of the bar (`tl-title-<key>`), behind the first stretch of its
/// dependency arrow. `None` for a bar whose title fits and for a milestone (its title always stands right).
pub fn outside_title(
    t: &Timeline,
    it: &Item,
    row: usize,
    scale: &Scale,
    g: Rows,
    top: f32,
) -> Option<Div> {
    let (look, s, pal) = (t.look, t.look.style, t.look.palette());
    if !matches!(it.kind, Kind::Bar { .. }) {
        return None;
    }
    let mut r = item_rect(it, row, scale, g);
    r.y += top;
    let x = outside_label_x(r, t.title_width(&it.title), s.spacing.xs)?;
    Some(
        div()
            .id(format!("tl-title-{}", it.key))
            .absolute()
            .left(x)
            .top(r.y)
            .h(r.h)
            .flex_row()
            .items_center()
            .pointer_events_none()
            .child(look.text(&it.title, s.text.caption, pal.text)),
    )
}

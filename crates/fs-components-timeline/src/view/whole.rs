//! The whole timeline: toolbar, then the axis, rows, arrows and today line in one scroll box.
//!
//! Pattern: View function — the source is read once per build; sizes come from `layout`.
//! One scroll box (both directions) holds everything: the axis scrolls away with the rows
//! (sticky header and labels are listed as Unsupported in DESIGN.md).

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::scroll::scroll;

use super::{arrows, axis, rows, toolbar};
use crate::handle::Timeline;
use crate::layout::{
    BodyRow, Rect, Rows, Scale, TodayMark, body_rows, item_rect, range, today_mark,
};
use crate::model::{Day, Item};
use crate::route::route;

/// The timeline filling `width` x `height`.
pub fn timeline(t: &Timeline, width: f32, height: f32) -> Div {
    let (s, st) = (t.look.style, t.snapshot());
    let items = t.source.items();
    let (first, last) = range(&items, 7, st.today.unwrap_or(Day(0)));
    let scale = Scale {
        origin: first,
        px: st.zoom.px_per_day(),
    };
    let g = Rows {
        row: s.heights.row_dense + s.spacing.xs,
        bar: s.heights.row_dense - s.spacing.xs,
    };
    let lines = body_rows(&items);
    let row_of: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(r, b)| matches!(b, BodyRow::Item(_)).then_some(r))
        .collect();
    let mark = today_mark(st.today, first, last);
    let rh = axis::row_height(t);
    let head = 2.0 * rh;
    let (total_w, rows_h) = (
        scale.x_of(last.add_days(1)).max(width),
        lines.len() as f32 * g.row,
    );
    // Not shrunk to the view: the scroll box must see the whole width.
    let mut body = div()
        .relative()
        .w(total_w)
        .min_w(total_w)
        .flex_shrink_0()
        .h(head + rows_h)
        .id("tl-body");
    for e in axis::axis(t, st.zoom, (first, last), &scale, head + rows_h) {
        body = body.child(e);
    }
    let rect_of = |key: &str| {
        items
            .iter()
            .position(|i| i.key == key)
            .map(|n| item_rect(&items[n], row_of[n], &scale, g))
    };
    let routes: Vec<_> = t
        .source
        .dependencies()
        .iter()
        .filter_map(|d| Some(route(rect_of(&d.from)?, rect_of(&d.to)?, g.row)))
        .collect();
    body = body.child(arrows::arrows(
        &t.look,
        Rect {
            x: 0.0,
            y: head,
            w: total_w,
            h: rows_h,
        },
        routes,
    ));
    for e in body_lines(
        t,
        &items,
        &lines,
        &scale,
        g,
        (head, total_w, st.selected.as_deref()),
    ) {
        body = body.child(e);
    }
    match mark {
        TodayMark::Line(today) => {
            body = body.child(rows::today_line(
                t,
                scale.x_of(today) + scale.px / 2.0 - 1.0,
                head + rows_h,
            ));
        }
        TodayMark::Before | TodayMark::After | TodayMark::None => {}
    }
    // A today off the axis is shown on a row of its own above the scroll box, so it is always at the
    // visible rim whatever the scroll position.
    let rim = matches!(mark, TodayMark::Before | TodayMark::After);
    let bar = s.heights.control + s.spacing.s + if rim { rh } else { 0.0 };
    let mut whole = div()
        .flex_col()
        .w(width)
        .h(height)
        .child(toolbar::toolbar(t));
    if rim {
        whole = whole.child(rows::today_edge(t, width, rh, mark == TodayMark::After));
    }
    whole.child(
        scroll()
            .id("tl-scroll")
            .bind(&t.scroll)
            .both_directions()
            .w(width)
            .h((height - bar).max(0.0))
            .content(body),
    )
}

/// The heading and item elements of the body rows, `top` pixels below the top of the body.
fn body_lines(
    t: &Timeline,
    items: &[Item],
    lines: &[BodyRow],
    scale: &Scale,
    g: Rows,
    (top, width, selected): (f32, f32, Option<&str>),
) -> Vec<Div> {
    lines
        .iter()
        .enumerate()
        .map(|(r, line)| match line {
            BodyRow::Lane(name) => rows::lane(t, name, r, width, g, top),
            BodyRow::Item(n) => {
                let it = &items[*n];
                rows::item(t, it, r, scale, g, top, selected == Some(it.key.as_str()))
            }
        })
        .collect()
}

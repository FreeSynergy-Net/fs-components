//! The outer layout: the list on the left (bar, search, rows), the person on the right.
//!
//! Pattern: Adapter (state to elements) — fixed skeleton; the areas are passed in. Only the widths are worked out here: the
//! heights are the flow of the columns (the bar and the search have theirs from the theme, the rest takes what is left), so a
//! larger font or a theme with other spaces cannot break the layout.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

/// The part of the width the list takes.
const LIST_SHARE: f32 = 0.34;
/// The least and the most width of the list column in pixels: a name must fit, and the person needs the room.
const LIST_MIN: f32 = 260.0;
const LIST_MAX: f32 = 380.0;

/// The areas of the frame, ready made.
pub(in crate::people) struct Areas<H, S, L, D> {
    /// The bar of the list: plus, title, choose, menu.
    pub head: H,
    /// The search field.
    pub search: S,
    /// The list.
    pub list: L,
    /// The person.
    pub detail: D,
}

/// Below this width the list stands above the person, each with the whole width.
const STACK_BELOW: f32 = 560.0;

/// Do the two areas stand one above the other?
fn stacked(width: f32) -> bool {
    width < STACK_BELOW
}

/// Width and height of the column of the list.
pub(in crate::people) fn side_size(width: f32, height: f32) -> (f32, f32) {
    if stacked(width) {
        (width.max(0.0), (height / 2.0).max(0.0))
    } else {
        (
            (width * LIST_SHARE)
                .clamp(LIST_MIN.min(width), LIST_MAX)
                .max(0.0),
            height.max(0.0),
        )
    }
}

/// Width and height of the person's area.
pub(in crate::people) fn detail_size(width: f32, height: f32) -> (f32, f32) {
    let (side_w, side_h) = side_size(width, height);
    if stacked(width) {
        (width.max(0.0), (height - side_h).max(0.0))
    } else {
        ((width - side_w).max(0.0), height.max(0.0))
    }
}

/// Everything, filling `width` x `height`.
pub(in crate::people) fn frame<H, S, L, D>(areas: Areas<H, S, L, D>, width: f32, height: f32) -> Div
where
    H: ElementBuilder + 'static,
    S: ElementBuilder + 'static,
    L: ElementBuilder + 'static,
    D: ElementBuilder + 'static,
{
    let (side_width, side_height) = side_size(width, height);
    let (main_width, main_height) = detail_size(width, height);
    let side = div()
        .class("fs-people-side")
        .flex_col()
        .w(side_width)
        .h(side_height)
        .min_h(0.0)
        .child(areas.head)
        .child(
            div()
                .class("fs-people-search")
                .flex_row()
                .items_center()
                .w(side_width)
                .child(areas.search),
        )
        .child(div().flex_1().w(side_width).min_h(0.0).child(areas.list));
    let root = div().class("fs-people-root");
    let root = if stacked(width) {
        root.flex_col()
    } else {
        root.flex_row()
    };
    root.w(width).h(height).child(side).child(
        div()
            .class("fs-people-main")
            .flex_col()
            .w(main_width)
            .h(main_height)
            .min_h(0.0)
            .child(areas.detail),
    )
}

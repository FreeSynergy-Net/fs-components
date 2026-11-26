//! The bar above the grid: previous, today, next, the shown month, and the mode buttons.
//!
//! Pattern: View function — state and `Say` in, elements out; a click calls the handle.

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};

use crate::handle::CalendarGrid;
use crate::model::{Mode, Nav};
use crate::names::month_title;
use crate::texts::view_key;

fn button(
    label: &str,
    variant: ButtonVariant,
    id: &str,
    on: impl Fn() + Send + Sync + 'static,
) -> Box<dyn blinc_layout::prelude::ElementBuilder> {
    Box::new(
        cn::button(label.to_owned())
            .variant(variant)
            .on_click(move |_| on())
            .build_component()
            .id(id),
    )
}

/// The toolbar for the state of `grid`.
pub fn toolbar(grid: &CalendarGrid) -> Div {
    let (s, pal) = (grid.look.style, grid.look.palette());
    let state = grid.snapshot();
    let say = |k: &str| grid.say.say(k);
    let nav = |n: Nav, key: &str, id: &str| {
        let g = grid.clone();
        button(&say(key), ButtonVariant::Secondary, id, move || {
            g.navigate(n);
        })
    };
    let title = month_title(&*grid.dates, state.anchor);
    let mut modes = div().flex_row().gap_px(s.spacing.xs);
    for m in Mode::ALL {
        let g = grid.clone();
        let variant = if m == state.mode {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        };
        modes = modes.child(button(
            &say(&view_key(m.name())),
            variant,
            &format!("cal-mode-{}", m.name()),
            move || g.set_mode(m),
        ));
    }
    div()
        .id("cal-toolbar")
        .flex_row()
        .items_center()
        .gap_px(s.spacing.s)
        .w_full()
        .h(s.heights.control + s.spacing.s)
        .child(nav(Nav::Previous, "calendar-previous", "cal-prev"))
        .child(nav(Nav::Today, "calendar-today", "cal-today"))
        .child(nav(Nav::Next, "calendar-next", "cal-next"))
        .child(
            div()
                .flex_grow_value(1.0)
                .flex_row()
                .gap_px(s.spacing.xs)
                .child(grid.look.text(title, s.text.title, pal.text)),
        )
        .child(modes)
}

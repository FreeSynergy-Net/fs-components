//! The outer layout: sidebar and content side by side.
//!
//! Pattern: Adapter (state to elements) — fixed skeleton; the areas are passed in.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

/// The widest the sidebar is, in pixels.
const SIDE: f32 = 320.0;
/// The least room the content gets while the window allows it, and the least the sidebar keeps.
const MAIN_MIN: f32 = 320.0;
const SIDE_LEAST: f32 = 160.0;

/// The sidebar width in a window `width` wide: the sidebar gives way so the content keeps `MAIN_MIN` (a 320 px sidebar at 480 px
/// left 160 px and the title broke letter by letter).
pub(crate) fn side_width(width: f32) -> f32 {
    SIDE.min((width - MAIN_MIN).max(SIDE_LEAST))
}

/// Sidebar (search box above `side`) left, `main` right, filling `width` x `height`.
pub(crate) fn frame(
    search: impl ElementBuilder + 'static,
    side: impl ElementBuilder + 'static,
    main: impl ElementBuilder + 'static,
    width: f32,
    height: f32,
) -> Div {
    div()
        .class("fs-rd-root")
        .w(width)
        .h(height)
        .child(
            div()
                .class("fs-rd-side")
                .w(side_width(width))
                .h(height)
                .min_h(0.0)
                .child(div().class("fs-rd-search").child(search))
                .child(side),
        )
        .child(div().class("fs-rd-main").h(height).min_h(0.0).child(main))
}

#[cfg(test)]
mod tests {
    use super::side_width;

    #[test]
    fn the_sidebar_gives_way_to_the_content() {
        assert_eq!(side_width(1200.0), 320.0);
        assert_eq!(side_width(680.0), 320.0);
        assert_eq!(side_width(560.0), 240.0);
        assert_eq!(side_width(480.0), 160.0);
        assert_eq!(side_width(100.0), 160.0);
    }
}

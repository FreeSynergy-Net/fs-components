//! How wide the side columns of the wiki layouts are, so the page in the middle never gets too narrow to read.
//!
//! Pattern: none — pure functions of the window width. The middle column keeps at least `MIDDLE_MIN` pixels: first the
//! right panel (outline and history) goes, then the left column shrinks. A title never breaks letter by letter (finding E5 of
//! 2026-11-14: a 130 px column at a window width of 700 px).

/// The least room the page in the middle gets, in pixels.
pub(super) const MIDDLE_MIN: f32 = 360.0;
/// The widest the left column is (the page tree).
pub(super) const LEFT: f32 = 300.0;
/// The Notion sidebar.
pub(super) const NOTION_SIDE: f32 = 264.0;
/// The right panel of the Confluence layout.
pub(super) const RIGHT: f32 = 270.0;
/// The left column never gets narrower than this while the window allows it.
const LEFT_LEAST: f32 = 120.0;

/// Below this window width the side column does not stand beside the page but above it (a column of 120 px cut every title to
/// its first letter, and the page beside it was too narrow for its own title and buttons).
pub(super) const STACK_BELOW: f32 = 460.0;

/// Is the window so narrow that the side column goes above the page?
pub(super) fn stacked(width: f32) -> bool {
    width < STACK_BELOW
}

/// The height of the side column when it stands above the page: a third of the window, but room for a few rows.
pub(super) fn stacked_height(height: f32) -> f32 {
    (height / 3.0).max(120.0)
}

/// Left column and right panel of the Confluence layout in a window `width` wide: the right panel is `None` when the middle
/// would be narrower than `MIDDLE_MIN` with it.
pub(super) fn confluence(width: f32) -> (f32, Option<f32>) {
    if stacked(width) {
        (width, None)
    } else if width >= LEFT + RIGHT + MIDDLE_MIN {
        (LEFT, Some(RIGHT))
    } else {
        (shrunk(width, LEFT), None)
    }
}

/// The Notion sidebar in a window `width` wide.
pub(super) fn notion(width: f32) -> f32 {
    if stacked(width) {
        return width;
    }
    shrunk(width, NOTION_SIDE)
}

fn shrunk(width: f32, wanted: f32) -> f32 {
    wanted.min((width - MIDDLE_MIN).max(LEFT_LEAST))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_narrow_window_stacks_the_column_above_the_page() {
        assert!(!stacked(560.0));
        assert!(stacked(380.0));
        assert_eq!(notion(380.0), 380.0, "the column takes the whole width");
        assert_eq!(confluence(380.0), (380.0, None));
        assert_eq!(notion(560.0), 200.0, "beside the page it still shrinks");
        assert!((stacked_height(400.0) - 133.3).abs() < 0.1);
        assert_eq!(stacked_height(200.0), 120.0);
    }
}

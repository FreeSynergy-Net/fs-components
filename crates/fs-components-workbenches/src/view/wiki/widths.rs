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

/// What the side column spends above its tree when it stands above the page (the search, the heading): the rows get the
/// rest of a third of the window.
const STACKED_CHROME: f32 = 110.0;
/// The fewest rows of the tree the stacked column shows (the tree of a small wiki is then whole).
const STACKED_ROWS_LEAST: usize = 3;
/// The most rows of the tree the stacked column shows.
const STACKED_ROWS_MOST: usize = 10;

/// How many whole rows (each `row` px high) of the tree the side column shows when it stands above the page: what a third of
/// the window leaves after the head, but at least three (and never more than the tree has, `pages`). The column is then as high as its head plus these rows, so no row is
/// cut by the line below it (finding B4 of the review of 0.0.20: 133 px showed one and a half rows).
pub(super) fn stacked_rows(height: f32, row: f32, pages: usize) -> usize {
    let room = (height / 3.0 - STACKED_CHROME) / row;
    // A small positive number of rows: the cast cannot truncate anything that matters.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = room.max(0.0).floor() as usize;
    // never more rows than the tree has (a small wiki does not get an empty column), never fewer than one
    whole
        .clamp(STACKED_ROWS_LEAST, STACKED_ROWS_MOST)
        .min(pages.max(1))
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
        // a third of 400 px is 133, the head takes 110: still three whole rows, never a part of one
        assert_eq!(stacked_rows(400.0, 30.0, 40), 3);
        assert_eq!(stacked_rows(900.0, 30.0, 40), 6);
        assert_eq!(
            stacked_rows(900.0, 30.0, 2),
            2,
            "a small tree: no empty rows"
        );
        assert_eq!(stacked_rows(900.0, 30.0, 0), 1);
        assert_eq!(stacked_rows(9000.0, 30.0, 40), 10);
        assert_eq!(stacked_rows(0.0, 30.0, 40), 3);
    }
}

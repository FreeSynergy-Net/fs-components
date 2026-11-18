//! The grey line under the title of a card as elements: the pieces joined (it takes the room that is left and is clipped at its
//! end) and, apart from it, the piece "+ N more" at the right end, which keeps its width and is never clipped.
//!
//! Pattern: Adapter (pieces in, Blinc layout out). Never cut silently: a long line may lose its end, the note that facts were
//! left out may not.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

/// The line of the card `key` of the board `id`: `pieces` joined, `more` fixed at the right.
pub(super) fn card_line(id: &str, key: &str, pieces: &[String], more: Option<String>) -> Div {
    let mut row = div()
        .class("fs-board-card-line")
        .flex_row()
        .items_center()
        .w_full();
    row = row.child(
        div()
            .id(format!("{id}-card-{key}-line"))
            .flex_grow()
            .flex_shrink()
            .min_w(0.0)
            .overflow_clip()
            .child(
                text(pieces.join("  \u{b7}  "))
                    .class("fs-board-card-meta")
                    .no_wrap(),
            ),
    );
    if let Some(more) = more {
        row = row.child(
            div()
                .id(format!("{id}-card-{key}-more"))
                .flex_shrink_0()
                .child(
                    text(more)
                        .class("fs-board-card-meta")
                        .class("fs-board-card-more")
                        .no_wrap(),
                ),
        );
    }
    row
}

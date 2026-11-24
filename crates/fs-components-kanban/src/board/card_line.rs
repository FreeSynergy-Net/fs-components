//! The grey line under the title of a card as elements: the subtitle, the facts, the due text and the assignee in a row (it takes the
//! room that is left and is clipped at its end) and, apart from it, the piece "+ N more" at the right end, which keeps its width and is
//! never clipped. Under the line the bar of the channel `progress`.
//!
//! Pattern: Adapter (the pieces of a card in, Blinc layout out) + Composite (the facts are the compact components of the field
//! blocks the catalog chose, `FieldSlot::compact_view`: one module for all, the component draws the value, never a text in its
//! place). Never cut silently: a long line may lose its end, the note that facts were left out may not.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_fields::Look;

use super::labels::Labels;
use super::meta::{card_lead, card_more, card_tail};
use crate::layout::PROGRESS_BAR;
use crate::{Card, FieldSlot};

const PIECE_GAP: f32 = 8.0;

fn piece(content: &str) -> blinc_layout::text::Text {
    text(content).class("fs-board-card-meta").no_wrap()
}

/// Does `card` have anything for the line (a subtitle, a shown fact, a due text, an assignee, or the note "+ N more")?
pub(super) fn has_line(card: &Card) -> bool {
    !card_lead(card).is_empty()
        || card.shown_facts().next().is_some()
        || !card_tail(card).is_empty()
        || card.more_facts() > 0
}

/// The line of `card` of the board `id`: the pieces in a row, the facts drawn by their components in `look`, `more` fixed at the right.
pub(super) fn card_line(id: &str, card: &Card, look: &Look, labels: &Labels) -> Div {
    let key = &card.key;
    let mut pieces = div()
        .id(format!("{id}-card-{key}-line"))
        .flex_row()
        .items_center()
        .gap_px(PIECE_GAP)
        .flex_grow()
        .flex_shrink()
        .min_w(0.0)
        .overflow_clip();
    for lead in card_lead(card) {
        pieces = pieces.child(piece(&lead));
    }
    // no value (`Unset`) takes no room: `shown_facts`; hidden or missing shows its reason (drawn by the component)
    for fact in card.shown_facts() {
        pieces = pieces.child(fact_view(fact, look));
    }
    for tail in card_tail(card) {
        pieces = pieces.child(piece(&tail));
    }
    let mut row = div()
        .class("fs-board-card-line")
        .flex_row()
        .items_center()
        .w_full()
        .child(pieces);
    if let Some(more) = card_more(card, labels) {
        row = row.child(
            div()
                .id(format!("{id}-card-{key}-more"))
                .flex_shrink_0()
                .child(piece(&more).class("fs-board-card-more")),
        );
    }
    row
}

fn fact_view(fact: &FieldSlot, look: &Look) -> Div {
    div().flex_shrink_0().child(fact.compact_view(look))
}

/// The bar of the optional channel `progress`: `percent` (cut to 100) of the width is filled. Two boxes share the width in the
/// proportion fill : rest, so the bar is as wide as the card lets it be.
pub(super) fn progress_bar(id: &str, key: &str, percent: u8) -> Div {
    let fill = f32::from(percent.min(100));
    div()
        .id(format!("{id}-card-{key}-progress"))
        .class("fs-board-progress")
        .flex_row()
        .w_full()
        .h(PROGRESS_BAR)
        .overflow_clip()
        .child(
            div()
                .id(format!("{id}-card-{key}-progress-fill"))
                .class("fs-board-progress-fill")
                .h(PROGRESS_BAR)
                .flex_grow_value(fill),
        )
        .child(div().h(PROGRESS_BAR).flex_grow_value(100.0 - fill))
}

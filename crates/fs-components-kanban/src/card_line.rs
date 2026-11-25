//! The grey line under the title of a card as elements: the subtitle, the facts, the due text and the assignees in a row and, apart
//! from it, the piece "+ N more" at the right end, which keeps its width and is never clipped. Under the line the bar of the channel
//! `progress`.
//!
//! Pattern: Adapter (the pieces of a card in, Blinc layout out) + Composite (the facts are the compact components of the field
//! blocks the catalog chose, `FieldSlot::compact_view`: one module for all, the component draws the value, never a text in its
//! place) + Decorator (a tooltip with the whole text on every piece that was shortened and on every assignee). Deviation: a fact
//! that does not get its room is drawn as its text with an ellipsis (`card_fit`), because a component cannot cut itself. Never cut
//! silently: the share of the room is computed (`card_fit::share`), what does not fit ends in "…" and keeps its whole text in a tooltip.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_fields::Look;
use fs_components_widgets::tooltip::Tooltip;

use crate::card_fit::{LINE_ROOM, fit, share, width_of};
use crate::labels::Labels;
use crate::meta::{assignee_marks, card_lead, card_more, card_tail};

use crate::{Card, FieldSlot};

const PIECE_GAP: f32 = 8.0;

/// What the note "+ N more" takes besides its text: its padding at the left.
const MORE_PAD: f32 = 8.0;

/// A rounding margin: a piece that misses its room by less than this is drawn whole.
const SLACK: f32 = 0.5;

/// The height of the progress bar of a card, in pixels.
const PROGRESS_BAR: f32 = 4.0;

fn piece(content: &str) -> blinc_layout::text::Text {
    text(content).class("fs-board-card-meta").no_wrap()
}

/// `trigger` with the whole `words` as a tooltip (hover).
fn tipped(id: &str, trigger: Div, words: &str) -> Div {
    Tooltip::new(words).id(id).attach(trigger)
}

/// Does `card` have anything for the line (a subtitle, a shown fact, a due text, an assignee, or the note "+ N more")?
pub(crate) fn has_line(card: &Card) -> bool {
    !card_lead(card).is_empty()
        || card.shown_facts().next().is_some()
        || !card_tail(card).is_empty()
        || card.more_facts() > 0
}

/// One piece that gives way: the text, and the fact behind it (a fact fits as its component, a subtitle as text).
struct Flex<'a> {
    text: String,
    fact: Option<&'a FieldSlot>,
}

/// The line of `card` of the board `id`: the pieces in a row, the facts drawn by their components in `look`, `more` fixed at the right.
pub(crate) fn card_line(id: &str, card: &Card, look: &Look, labels: &Labels) -> Div {
    let key = &card.key;
    let more = card_more(card, labels);
    let marks = assignee_marks(card);
    let due = card.due.as_ref().map(|d| d.text.clone());
    let mut flex: Vec<Flex> = card_lead(card)
        .into_iter()
        .map(|text| Flex { text, fact: None })
        .collect();
    flex.extend(card.shown_facts().map(|f| Flex {
        text: f.line(),
        fact: Some(f),
    }));
    // what keeps its width: the due text, the assignees, the note; the rest shares what is left (gaps between the pieces counted)
    let fixed: f32 = due.iter().map(|t| width_of(t)).sum::<f32>()
        + marks.iter().map(|(s, _)| width_of(s)).sum::<f32>()
        + more.as_ref().map_or(0.0, |m| width_of(m) + MORE_PAD);
    let pieces_n = flex.len() + usize::from(due.is_some()) + marks.len();
    let gaps = PIECE_GAP * pieces_n.saturating_sub(1) as f32;
    let wanted: Vec<f32> = flex.iter().map(|f| width_of(&f.text)).collect();
    let given = share(&wanted, LINE_ROOM - fixed - gaps);

    let mut pieces = div()
        .id(format!("{id}-card-{key}-line"))
        .flex_row()
        .items_center()
        .gap_px(PIECE_GAP)
        .flex_grow()
        .flex_shrink()
        .min_w(0.0)
        .overflow_clip();
    for (n, item) in flex.iter().enumerate() {
        let tip_id = format!("{id}-card-{key}-flex-{n}");
        if given[n] + SLACK >= wanted[n] {
            pieces = pieces.child(match item.fact {
                Some(fact) => fact_view(fact, look),
                None => div().flex_shrink_0().child(piece(&item.text)),
            });
        } else {
            let cut = div()
                .flex_shrink_0()
                .child(piece(&fit(&item.text, given[n])));
            pieces = pieces.child(tipped(&tip_id, cut, &item.text));
        }
    }
    if let Some(due) = &due {
        pieces = pieces.child(div().flex_shrink_0().child(piece(due)));
    }
    for (n, (short, full)) in marks.iter().enumerate() {
        let mark = div().flex_shrink_0().child(piece(short));
        pieces = pieces.child(tipped(&format!("{id}-card-{key}-who-{n}"), mark, full));
    }
    let mut row = div()
        .class("fs-board-card-line")
        .flex_row()
        .items_center()
        .w_full()
        .child(pieces);
    if let Some(more) = more {
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
pub(crate) fn progress_bar(id: &str, key: &str, percent: u8) -> Div {
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

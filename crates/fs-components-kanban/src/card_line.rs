//! The grey line under the title of a card as elements: the subtitle, the facts, the due text and the assignees in a row and, apart
//! from it, the piece "+ N more" at the right end, which keeps its width and is never clipped. Under the line the bar of the channel
//! `progress`.
//!
//! Pattern: Adapter (the pieces of a card in, Blinc layout out) + Composite (the facts are the compact components of the field
//! blocks the catalog chose, `FieldSlot::compact_view`: one module for all, the component draws the value, never a text in its
//! place) + Decorator (a tooltip with the whole text on every piece that was shortened and on every assignee). The width of a fact is measured as it is drawn
//! (`FieldSlot::compact_width`: icon, gap, text in the look); one that does not get its room is drawn by the same component with its
//! text cut (`compact_view_text`). Never cut silently: the share of the room is computed (`card_fit::share`), what does not fit ends
//! in "…" with its whole text in a tooltip; a fact with no room at all is left out and counted in "+ N more".

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_fields::Look;
use fs_components_widgets::tooltip::Tooltip;

use crate::card_fit::{LINE_ROOM, fit, share, width_of};
use crate::labels::Labels;
use crate::meta::{assignee_marks, card_lead, card_tail};
use fs_components_fields::text_width;
use fs_components_widgets::list_view::fit_text;

const ELLIPSIS: &str = "\u{2026}";
/// The least a due text keeps when it is cut, and the least the pieces that give way are left.
const MIN_DUE: f32 = 40.0;
const MIN_FLEX: f32 = 40.0;

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

/// One piece that gives way: the subtitle (text) or a fact (a component).
enum Flex<'a> {
    Lead(String),
    Fact(&'a FieldSlot, String),
}

impl Flex<'_> {
    fn wanted(&self, look: &Look) -> f32 {
        match self {
            Flex::Lead(t) => width_of(t),
            Flex::Fact(f, _) => f.compact_width(look),
        }
    }

    /// The room a fact needs at least: icon and gap and one "…" (less than that, it is left out and counted in "+ N more").
    fn least(&self, look: &Look) -> f32 {
        match self {
            Flex::Lead(_) => width_of(ELLIPSIS),
            Flex::Fact(f, line) => {
                f.compact_width(look) - text_width(line, look) + text_width(ELLIPSIS, look)
            }
        }
    }
}

/// What the line draws: the pieces that give way with their room, the due text (cut if it must be), the number of facts left out.
struct Plan<'a> {
    flex: Vec<Flex<'a>>,
    given: Vec<f32>,
    due: Option<String>,
    dropped: usize,
}

/// The room of the line shared out. The marks and the note keep their width; the due text may be cut to what they leave (not under
/// `MIN_DUE`); a fact that would get less than its least is left out (the note counts it).
fn plan<'a>(card: &'a Card, look: &Look, labels: &Labels) -> Plan<'a> {
    let marks_w: f32 = assignee_marks(card)
        .iter()
        .map(|(s, _)| width_of(s) + PIECE_GAP)
        .sum();
    let facts: Vec<&FieldSlot> = card.shown_facts().collect();
    let mut dropped = 0;
    loop {
        let more = card.more_facts() + dropped;
        let more_w = if more > 0 {
            width_of(&labels.more(more)) + MORE_PAD
        } else {
            0.0
        };
        let due_room = (LINE_ROOM - marks_w - more_w - MIN_FLEX).max(MIN_DUE);
        let due = card.due.as_ref().map(|d| fit(&d.text, due_room));
        let due_w = due.as_ref().map_or(0.0, |d| width_of(d) + PIECE_GAP);
        let mut flex: Vec<Flex> = card_lead(card).into_iter().map(Flex::Lead).collect();
        flex.extend(
            facts[..facts.len() - dropped]
                .iter()
                .map(|f| Flex::Fact(f, f.line())),
        );
        let gaps = PIECE_GAP * flex.len().saturating_sub(1) as f32;
        let wanted: Vec<f32> = flex.iter().map(|f| f.wanted(look)).collect();
        let given = share(&wanted, LINE_ROOM - marks_w - more_w - due_w - gaps);
        let starved = flex.iter().zip(&given).any(|(f, g)| {
            matches!(f, Flex::Fact(..)) && *g + SLACK < f.least(look) && *g + SLACK < f.wanted(look)
        });
        if starved && dropped < facts.len() {
            dropped += 1;
            continue;
        }
        return Plan {
            flex,
            given,
            due,
            dropped,
        };
    }
}

/// The line of `card` of the board `id`: the pieces in a row, the facts drawn by their components in `look`, `more` fixed at the right.
pub(crate) fn card_line(id: &str, card: &Card, look: &Look, labels: &Labels) -> Div {
    let key = &card.key;
    let Plan {
        flex,
        given,
        due,
        dropped,
    } = plan(card, look, labels);
    let more = card.more_facts() + dropped;
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
        let whole = given[n] + SLACK >= item.wanted(look);
        pieces = match (item, whole) {
            (Flex::Fact(fact, _), true) => pieces.child(fact_view(fact, look)),
            (Flex::Lead(t), true) => pieces.child(div().flex_shrink_0().child(piece(t))),
            (Flex::Lead(t), false) => {
                let cut = div().flex_shrink_0().child(piece(&fit(t, given[n])));
                pieces.child(tipped(&tip_id, cut, t))
            }
            (Flex::Fact(fact, line), false) => {
                let room = given[n] - item.least(look) + text_width(ELLIPSIS, look);
                let cut = fit_text(line, room, look.style.text.body);
                let shown = div()
                    .flex_shrink_0()
                    .child(fact.compact_view_text(look, &cut));
                pieces.child(tipped(&tip_id, shown, line))
            }
        };
    }
    if let (Some(shown), Some(full)) = (&due, &card.due) {
        let piece = div().flex_shrink_0().child(piece(shown));
        pieces = pieces.child(if *shown == full.text {
            piece
        } else {
            tipped(&format!("{id}-card-{key}-due"), piece, &full.text)
        });
    }
    for (n, (short, full)) in assignee_marks(card).iter().enumerate() {
        let mark = div().flex_shrink_0().child(piece(short));
        pieces = pieces.child(tipped(&format!("{id}-card-{key}-who-{n}"), mark, full));
    }
    let mut row = div()
        .class("fs-board-card-line")
        .flex_row()
        .items_center()
        .w_full()
        .child(pieces);
    if more > 0 {
        row = row.child(
            div()
                .id(format!("{id}-card-{key}-more"))
                .flex_shrink_0()
                .child(piece(&labels.more(more)).class("fs-board-card-more")),
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

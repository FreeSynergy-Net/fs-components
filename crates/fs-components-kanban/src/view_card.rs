//! The pieces of the board that more than one view draws: the head of a column, the work-in-progress mark and the card.
//!
//! Pattern: Template method (split from `view.rs`; the columns and the lanes both call the same pieces, so a card looks the same in either).

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::Card;
use crate::bench::BoardBench;
use crate::card_line::{card_line, has_line, progress_bar};
use crate::hit::{CARD_H, COLUMN_W, HEAD_H};
use crate::requests::ContextTarget;
use crate::state::BoardState;
use crate::wip::WipMark;

impl BoardBench {
    pub(crate) fn head_row(&self, state: &BoardState, n: usize) -> Div {
        let head = &state.heads()[n];
        let mark = state.wip(n);
        let id = &self.id;
        let this = self.clone();
        let key = head.key.clone();
        let mut name_row = div()
            .id(format!("{id}-head-{}", head.key))
            .class("fs-board-head")
            .flex_row()
            .items_center()
            .w(COLUMN_W)
            .h(HEAD_H)
            .child(text(head.title.clone()).class("fs-board-col-name"));
        if let Some(mark) = mark {
            name_row = name_row.child(self.wip(&head.key, &mark));
        }
        if head.hidden > 0 {
            name_row = name_row.child(
                text(self.labels.hidden(head.hidden))
                    .class("fs-board-hidden")
                    .id(format!("{id}-hidden-{}", head.key)),
            );
        }
        let menu = self.clone();
        let menu_key = key.clone();
        let menu_button = self.menu_button(&head.key);
        name_row = name_row
            .child(div().flex_1())
            .child(menu_button.on_click(move |ctx| {
                menu.context_by_click(
                    &ContextTarget::Column(menu_key.clone()),
                    (ctx.mouse_x, ctx.mouse_y),
                );
            }))
            .on_right_click(move |ctx| {
                this.context(
                    &ContextTarget::Column(key.clone()),
                    (ctx.mouse_x, ctx.mouse_y),
                );
            });
        name_row
    }

    pub(crate) fn wip(&self, column: &str, mark: &WipMark) -> Div {
        let mut row = div()
            .id(format!("{}-wip-{column}", self.id))
            .class("fs-board-wip")
            .flex_row()
            .items_center();
        if mark.over {
            // a symbol as well as a colour: the colour alone says nothing to everyone
            row = row
                .class("fs-board-wip-over")
                .attr("aria-label", self.labels.wip_over.as_str())
                .child(text("\u{25b2}").class("fs-board-wip-symbol"));
        }
        row.child(text(mark.text.clone()).class("fs-board-wip-text"))
    }

    pub(crate) fn card(&self, state: &BoardState, card: &Card) -> Div {
        self.card_in(state, card, None)
    }

    /// A card; in a lane (`lane` = the lane key) its id carries the lane (a card may stand in several lanes) and it is not picked up
    /// by the pointer: the geometry of hits knows one row of columns (moving between lanes is only shown, never done).
    pub(crate) fn card_in(&self, state: &BoardState, card: &Card, lane: Option<&str>) -> Div {
        let id = &self.id;
        let card_id = lane.map_or_else(
            || format!("{id}-card-{}", card.key),
            |l| format!("{id}-lane-{l}-card-{}", card.key),
        );
        let key = card.key.clone();
        let carried = state.carried().is_some_and(|(k, ..)| k == card.key);
        let mut d = div()
            .id(card_id)
            .class("fs-board-card")
            .attr("role", "listitem")
            .flex_col()
            .w(COLUMN_W)
            .h(CARD_H)
            .overflow_clip()
            .child(text(card.title.clone()).class("fs-board-card-title"));
        if has_line(card) {
            d = d.child(card_line(id, card, &self.look, &self.labels));
        }
        if let Some(percent) = card.progress {
            d = d.child(progress_bar(id, &card.key, percent));
        }
        if state.cursor() == Some(card.key.as_str()) {
            d = d.class("fs-board-card-cursor");
        }
        if state.held() == Some(card.key.as_str()) {
            d = d.class("fs-board-card-held").attr("aria-grabbed", "true");
        }
        if carried {
            d = d.class("fs-board-card-carried");
        }
        if state.locked(&card.key).is_some() {
            d = d.class("fs-board-card-locked");
        }
        let (down, drag, end, click, menu) = (
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        );
        let (k1, k2, k3, k4) = (key.clone(), key.clone(), key.clone(), key);
        let mut d = d.cursor_pointer();
        if lane.is_none() {
            d = d
                .on_mouse_down(move |ctx| {
                    if ctx.mouse_button == 0 {
                        down.press(&k1, (ctx.mouse_x, ctx.mouse_y));
                    }
                })
                .on_drag(move |ctx| {
                    drag.drag(&k2, (ctx.mouse_x, ctx.mouse_y), (ctx.local_x, ctx.local_y));
                })
                .on_drag_end(move |_| end.release());
        }
        d
            // A click without a carry opens the card; with a card held it puts the held card down in this column (the click-to-move).
            .on_click(move |_| click.card_click(&k3))
            .on_right_click(move |ctx| {
                menu.context(&ContextTarget::Card(k4.clone()), (ctx.mouse_x, ctx.mouse_y));
            })
    }
}

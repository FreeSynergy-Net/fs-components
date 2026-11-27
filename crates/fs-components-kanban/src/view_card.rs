//! The pieces of the board that more than one view draws: the head of a column, the work-in-progress mark and the card.
//!
//! Pattern: Template method (split from `view.rs`; the columns and the lanes both call the same pieces, so a card looks the same in either).

use std::sync::{Arc, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_widgets::card::{Card as CardWidget, Target};

use crate::Card;
use crate::bench::BoardBench;
use crate::card_line::{card_line, has_line, progress_bar};
use crate::places::{Events, ref_of};
use crate::requests::ContextTarget;
use crate::sizes::{COLUMN_W, HEAD_H};
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

    /// A card, made by the card component; in a lane (`lane` = the lane key) its id carries the lane (a card may stand in several
    /// lanes). It is as high as its content. It can be carried when the program takes moves; without that it is only clickable.
    pub(crate) fn card_in(&self, state: &BoardState, card: &Card, lane: Option<&str>) -> Div {
        let id = &self.id;
        let card_id = lane.map_or_else(
            || format!("{id}-card-{}", card.key),
            |l| format!("{id}-lane-{l}-card-{}", card.key),
        );
        let column = state
            .lanes()
            .iter()
            .find(|l| l.items.iter().any(|i| i.key == card.key))
            .map_or_else(String::new, |l| l.key.clone());
        let mut widget = CardWidget::new(card_id.clone(), ref_of(id, &card.key))
            .at(Target::cell(lane, &column))
            .events(Arc::new(Events(self.clone())));
        if let Some(places) = self.current_places() {
            widget = widget.targets(places);
        }
        if let Some(reason) = state.locked(&card.key) {
            widget = widget.locked(reason.clone());
        }
        self.cards
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(card_id, widget.clone());
        let mut parts = vec![
            div()
                .class("fs-card-title")
                .class("fs-board-card-title")
                .child(text(card.title.clone())),
        ];
        if has_line(card) {
            parts.push(card_line(id, card, &self.look, &self.labels));
        }
        if let Some(percent) = card.progress {
            parts.push(progress_bar(id, &card.key, percent));
        }
        let mut d = widget.build(parts).class("fs-board-card").w(COLUMN_W);
        if state.cursor() == Some(card.key.as_str()) {
            d = d.class("fs-board-card-cursor");
        }
        if state.held() == Some(card.key.as_str()) {
            d = d.class("fs-board-card-held").attr("aria-grabbed", "true");
        }
        d.attr("role", "listitem")
    }
}

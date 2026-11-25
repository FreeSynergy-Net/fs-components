//! The board as elements: the title, the columns with their cards, the foot of each column, the layer of the carried card, the
//! frame at the side and the line under the board.
//!
//! Pattern: Adapter (the state in, Blinc layout out) + Template method (`render` fixes the order). Sizes come from `hit.rs`: a
//! card is `CARDH` high whatever it holds and a column `COLUMN_W` wide, so the rectangles the pointer is tested against are
//! exactly the ones drawn. The cards and the layer of the carried card are two live areas: moving the pointer draws only the
//! layer, never the cards, so Blinc keeps the gesture on the element that was pressed.
//! Look (b7, lessons of the pictures): big column names, readable 15 px titles, a quiet card without coloured badges (one grey line
//! of facts), the work-in-progress mark as plain text (a warning colour and a symbol only above the limit), generous space.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::keys::bind_keys;

use crate::bench::BoardBench;
use crate::hit::{CARD_GAP, COLUMN_GAP, COLUMN_W, PAD};
use crate::requests::ContextTarget;
use crate::state::BoardState;

/// Width of the frame at the side.
pub const DETAIL_W: f32 = 360.0;

impl BoardBench {
    /// The element, `width` x `height`; place it in a window with [`styles`](super::styles) in its theme.
    pub fn build(&self, width: f32, height: f32) -> Div {
        let this = self.clone();
        let body = self
            .live
            .region("board", move || this.render(width, height));
        let root = div()
            .id(&self.id)
            .class("fs-board-root")
            .w(width)
            .h(height)
            .child(body);
        let this = self.clone();
        bind_keys(root, Arc::new(move |event| this.press_key(&event)))
    }

    fn render(&self, width: f32, height: f32) -> Div {
        let state = self.snapshot();
        let detail = state.open_card().and_then(|k| {
            self.detail
                .as_ref()
                .and_then(|d| d.frame(k))
                .map(|f| (k.to_owned(), f))
        });
        let side = if detail.is_some() { DETAIL_W } else { 0.0 };
        let main_w = (width - side).max(0.0);
        let mut head = div()
            .class("fs-board-title-row")
            .w(main_w)
            .h(64.0)
            .overflow_clip();
        if let Some(title) = &self.title {
            head = head.child(text(title.clone()).class("fs-board-title"));
        }
        let line = self.line(&state);
        let line_h = if line.is_some() { 56.0 } else { 0.0 };
        let title_h = if self.title.is_some() { 64.0 } else { 0.0 };
        let board_h = (height - title_h - line_h).max(0.0);
        let mut main = div().class("fs-board-main").flex_col().w(main_w).h(height);
        if self.title.is_some() {
            main = main.child(head);
        }
        if matches!(state.lane_plan(), crate::LanePlan::Missing) {
            main = main.child(
                div()
                    .class("fs-board-lanes-missing")
                    .id(format!("{}-lanes-missing", self.id))
                    .child(text(self.labels.lanes_missing.clone())),
            );
        }
        main = main.child(self.columns_area(&state, main_w, board_h));
        if let Some(line) = line {
            main = main.child(line);
        }
        let mut row = div()
            .class("fs-board-row")
            .flex_row()
            .w(width)
            .h(height)
            .child(main);
        if let Some((card, frame)) = detail {
            row = row.child(self.detail_frame(&card, &frame, height));
        }
        row
    }

    // ---- the columns ------------------------------------------------------------------------------------------------------

    fn columns_area(&self, state: &BoardState, width: f32, height: f32) -> Div {
        let geometry = state.geometry();
        let (mut inner_w, mut inner_h) = geometry.size();
        let lanes = self.lanes_of_state(state);
        if let Some(lanes) = &lanes {
            (inner_w, inner_h) = self.lanes_size(state, lanes);
        }
        let (inner_w, inner_h) = (inner_w.max(width), inner_h.max(height));
        let body = match &lanes {
            Some(lanes) => self.lanes_columns(state, lanes),
            None => self.plain_columns(state),
        };
        let this = self.clone();
        let layer = self.drag_live.region("drag", move || this.drag_layer());
        let on_empty = self.clone();
        let content = div()
            .class("fs-board-content")
            .id(format!("{}-content", self.id))
            .flex_col()
            .w(inner_w)
            .min_w(inner_w)
            .h(inner_h)
            .flex_shrink_0()
            .child(layer)
            .child(body)
            .on_right_click(move |ctx| {
                on_empty.context(&ContextTarget::Board, (ctx.mouse_x, ctx.mouse_y));
            });
        div().flex_1().min_h(0.0).w(width).h(height).child(
            scroll()
                .bind(&self.scroll)
                .both_directions()
                .scrollbar_always()
                .w_full()
                .h_full()
                .content(content),
        )
    }

    fn plain_columns(&self, state: &BoardState) -> Div {
        let mut columns = div()
            .class("fs-board-columns")
            .flex_row()
            .gap_px(COLUMN_GAP)
            .padding_x_px(PAD)
            .padding_y_px(PAD);
        for n in 0..state.heads().len() {
            columns = columns.child(self.column(state, n));
        }
        columns
    }

    fn column(&self, state: &BoardState, n: usize) -> Div {
        let head = &state.heads()[n];
        let lane = &state.lanes()[n];
        let id = &self.id;
        let name_row = self.head_row(state, n);
        // The click-to-move targets the list of cards, not the foot or the "..." button: a click on a locked control
        // must not put a held card down.
        let (put_down, column_key) = (self.clone(), head.key.clone());
        let mut cards = div()
            .class("fs-board-cards")
            .flex_col()
            .gap_px(CARD_GAP)
            .w(COLUMN_W)
            .attr("role", "list")
            .id(format!("{id}-cards-{}", head.key))
            .on_click(move |_| put_down.click_cell(&column_key));
        for item in &lane.items {
            if let Some(card) = state.card(&item.key) {
                cards = cards.child(self.card(state, card));
            }
        }
        if lane.items.is_empty() {
            cards = cards.child(
                text(self.labels.empty.clone())
                    .class("fs-board-empty")
                    .id(format!("{id}-empty-{}", head.key)),
            );
        }
        div()
            .id(format!("{id}-col-{}", head.key))
            .class("fs-board-column")
            .flex_col()
            .w(COLUMN_W)
            .flex_shrink_0()
            .attr("role", "group")
            .child(name_row)
            .child(cards)
            .child(self.foot(state, &head.key))
    }

    // ---- the foot: quick entry ----------------------------------------------------------------------------------------------

    // ---- the carried card ---------------------------------------------------------------------------------------------------

    // ---- under the board ------------------------------------------------------------------------------------------------------

    // ---- the frame at the side --------------------------------------------------------------------------------------------------
}

//! The board as elements: the title, the columns with their cards, the foot of each column, the layer of the carried card, the
//! frame at the side and the line under the board.
//!
//! Pattern: Adapter (the state in, Blinc layout out) + Template method (`render` fixes the order). The columns and the lanes are a `grid` (`board_grid.rs`), the cards are `card` components
//! as high as their content; the places a card can land are measured from the laid-out tree (`places.rs`). The cards and the layer of the carried card are two live areas: moving the pointer draws only the
//! layer, never the cards, so Blinc keeps the gesture on the element that was pressed.
//! Look (b7, lessons of the pictures): big column names, readable 15 px titles, a quiet card without coloured badges (one grey line
//! of facts), the work-in-progress mark as plain text (a warning colour and a symbol only above the limit), generous space.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::scroll;

use crate::bench::BoardBench;
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
        // No tab stop of its own: the keyboard is the grid's (`board_grid.rs`), which gives the keys to the board inside a cell.
        div()
            .id(&self.id)
            .class("fs-board-root")
            .w(width)
            .h(height)
            .child(body)
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

    /// The grid of the board in a scroll area, and over it the layer of the carried card.
    fn columns_area(&self, state: &BoardState, width: f32, height: f32) -> Div {
        let lanes = self.lanes_of_state(state).map(Arc::new);
        // The places a card can land: made from the data, read from the tree while one is carried (`places.rs`).
        self.places(state, lanes.as_deref().map(Vec::as_slice));
        let grid = self.board_grid(&Arc::new(state.clone()), lanes);
        let this = self.clone();
        let layer = self.drag_live.region("drag", move || this.drag_layer());
        let on_empty = self.clone();
        let content = div()
            .class("fs-board-content")
            .id(format!("{}-content", self.id))
            .class("fs-board-grid")
            .flex_col()
            .min_w(width)
            .min_h(height)
            .flex_shrink_0()
            .child(grid.build())
            // after the grid: the layer paints over the cells, which have a ground of their own
            .child(div().absolute().left(0.0).top(0.0).child(layer))
            .on_right_click(move |ctx| {
                on_empty.context(&ContextTarget::Board, (ctx.mouse_x, ctx.mouse_y));
            });
        div()
            .id(format!("{}-scrollbox", self.id))
            .flex_1()
            .min_h(0.0)
            .w(width)
            .h(height)
            .child(
                scroll()
                    .bind(&self.scroll)
                    .both_directions()
                    .scrollbar_always()
                    .w_full()
                    .h_full()
                    .content(content),
            )
    }

    // ---- the foot: quick entry ----------------------------------------------------------------------------------------------

    // ---- the carried card ---------------------------------------------------------------------------------------------------

    // ---- under the board ------------------------------------------------------------------------------------------------------

    // ---- the frame at the side --------------------------------------------------------------------------------------------------
}

//! The foot of a column and the layer of the carried card.
//!
//! Pattern: Adapter (split from `view.rs`).

use crate::Operation;
use crate::bench::BoardBench;
use crate::hit::{COLUMN_W, FOOT_H, Geometry};
use crate::state::BoardState;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use std::sync::Arc;
impl BoardBench {
    pub(crate) fn foot(&self, state: &BoardState, column: &str) -> Div {
        let id = &self.id;
        if !self.source.supports(Operation::AddCard) {
            // locked, not left out (ADR 0096 H-K5): it says "soon" and does nothing; it unlocks by itself when the program supports it
            return div()
                .id(format!("{id}-add-{column}"))
                .class("fs-board-foot")
                .class("fs-board-add")
                .class("fs-board-locked")
                .attr("data-locked", "true")
                .attr("aria-disabled", "true")
                .w(COLUMN_W)
                .h(FOOT_H)
                .flex_row()
                .items_center()
                .child(text(format!("+ {}", self.labels.add)).class("fs-board-add-text"))
                .child(
                    text(self.labels.soon.clone())
                        .id(format!("{id}-soon-{column}"))
                        .class("fs-board-soon"),
                );
        }
        if state.quick() == Some(column) {
            let (this, col) = (self.clone(), column.to_owned());
            let entry = self.entry(column).build_with(
                &format!("{id}-quick-{column}"),
                Arc::new(move |text: &str, shift: bool| this.quick_submit(&col, text, shift)),
            );
            return div().class("fs-board-foot").w(COLUMN_W).child(entry);
        }
        let (this, col) = (self.clone(), column.to_owned());
        div()
            .id(format!("{id}-add-{column}"))
            .class("fs-board-foot")
            .class("fs-board-add")
            .w(COLUMN_W)
            .h(FOOT_H)
            .items_center()
            .cursor_pointer()
            .child(text(format!("+ {}", self.labels.add)).class("fs-board-add-text"))
            .on_click(move |_| this.open_quick(&col))
    }

    /// The layer over the columns: the target column lit, a line where the card would land, the card at the pointer, the reason
    /// when the place does not take it. Empty when nothing is carried.
    pub(crate) fn drag_layer(&self) -> Div {
        let state = self.snapshot();
        let layer = div().class("fs-board-drag").absolute().left(0.0).top(0.0);
        let Some((card, over, pointer)) = state.carried() else {
            return layer;
        };
        let mut layer = layer;
        if let Some(spot) = over {
            let blocked = state.blocked().is_some();
            let x = state.geometry().column_x(spot.column);
            let mut line = div()
                .id(format!("{}-drop-line", self.id))
                .class("fs-board-drop-line")
                .absolute()
                .left(x)
                .top(Geometry::line_y(spot))
                .w(COLUMN_W)
                .pointer_events_none();
            if blocked {
                line = line.class("fs-board-drop-blocked");
            }
            layer = layer.child(line);
        }
        let title = state
            .card(card)
            .map(|c| c.title.clone())
            .unwrap_or_default();
        let mut ghost = div()
            .id(format!("{}-ghost", self.id))
            .class("fs-board-ghost")
            .flex_col()
            .absolute()
            .left(pointer.0 - COLUMN_W / 2.0)
            .top(pointer.1 - 20.0)
            .w(COLUMN_W)
            .pointer_events_none()
            .child(text(title).class("fs-board-card-title"));
        if let Some(reason) = state.blocked() {
            ghost = ghost.class("fs-board-ghost-blocked").child(
                text(reason.said().to_owned())
                    .class("fs-board-ghost-reason")
                    .id(format!("{}-reason", self.id)),
            );
        }
        layer.child(ghost)
    }
}

impl BoardBench {
    /// The "..." button of a column head. When the host offers no menu and no table of columns, it has nothing to do: it stays,
    /// locked, and says "soon" to assistive technology (ADR 0096 H-K5).
    pub(crate) fn menu_button(&self, column: &str) -> Div {
        let live = self.source.offers_menu() || self.source.columns_editable();
        let name = self.labels.column_menu.clone();
        let button = div()
            .id(format!("{}-menu-{column}", self.id))
            .class("fs-board-head-menu")
            .attr("role", "button")
            .child(text("\u{22ef}"));
        if live {
            return button.attr("aria-label", name.as_str()).cursor_pointer();
        }
        button
            .class("fs-board-locked")
            .attr("data-locked", "true")
            .attr("aria-disabled", "true")
            .attr(
                "aria-label",
                format!("{name} ({})", self.labels.soon).as_str(),
            )
    }
}

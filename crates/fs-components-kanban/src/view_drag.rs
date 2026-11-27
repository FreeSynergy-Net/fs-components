//! The foot of a column and the layer of the carried card.
//!
//! Pattern: Adapter (split from `view.rs`).

use crate::Operation;
use crate::bench::BoardBench;
use crate::places::key_of;
use crate::sizes::{COLUMN_W, FOOT_H};
use crate::state::BoardState;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_widgets::disabled::{Disabled, Why};
use std::sync::{Arc, PoisonError};

use blinc_layout::selector::query;
use fs_components_widgets::card::layer::{drop_line, ghost, within};

impl BoardBench {
    pub(crate) fn foot(&self, state: &BoardState, column: &str) -> Div {
        let id = &self.id;
        if !self.source.supports(Operation::AddCard) {
            // not left out (ADR 0096 H-K5): same size as the live foot, the reason in the tooltip, it unlocks by itself
            let foot = div()
                .id(format!("{id}-add-{column}"))
                .class("fs-board-foot")
                .class("fs-board-add")
                .w(COLUMN_W)
                .h(FOOT_H)
                .items_center()
                .child(text(format!("+ {}", self.labels.add)).class("fs-board-add-text"));
            return Disabled::new(
                &format!("{id}-add-{column}"),
                Why::Unsupported,
                &self.labels.why,
            )
            .wrap(foot);
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

    /// The layer over the grid: the line where the carried card would land and the card at the pointer, with the reason when the
    /// place does not take it. Empty when nothing is carried. The line and the pointer come in window coordinates; the layer is laid
    /// at the top left of the content (`<id>-layer`), so its own place is taken away.
    pub(crate) fn drag_layer(&self) -> Div {
        let layer = div()
            .id(format!("{}-layer", self.id))
            .class("fs-board-drag")
            .absolute()
            .left(0.0)
            .top(0.0);
        let active = self
            .active
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let Some((card_id, card)) = active else {
            return layer;
        };
        let Some((over, pointer)) = card.over() else {
            return layer;
        };
        let origin = query(&format!("{}-layer", self.id))
            .and_then(|q| q.bounds())
            .map_or((0.0, 0.0), |b| (b.x, b.y));
        let blocked = card.blocked();
        let mut layer = layer;
        let places = self.current_places();
        if let (Some(places), Some(to)) = (&places, &over)
            && let Some(line) = places.line(to)
        {
            layer = layer.child(drop_line(
                &format!("{}-drop-line", self.id),
                within(line, origin),
                blocked.is_some(),
            ));
        }
        let title = self
            .read(|s| s.card(&key_of(card.key())).map(|c| c.title.clone()))
            .unwrap_or_default();
        let size = query(&card_id)
            .and_then(|q| q.bounds())
            .map_or((COLUMN_W, 56.0), |b| (b.width, b.height));
        let at = (pointer.0 - origin.0, pointer.1 - origin.1);
        let parts = vec![
            text(title)
                .class("fs-card-title")
                .class("fs-board-card-title"),
        ];
        layer.child(ghost(
            &format!("{}-ghost", self.id),
            parts.into_iter().map(|t| div().child(t)).collect(),
            at,
            size,
            blocked.as_ref(),
        ))
    }
}

impl BoardBench {
    /// The "..." button of a column head. When the host offers no menu and no table of columns, it has nothing to do: it stays,
    /// disabled with the reason in a tooltip (ADR 0096 H-K5).
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
        Disabled::new(
            &format!("{}-menu-{column}", self.id),
            Why::Unsupported,
            &self.labels.why,
        )
        .wrap(button.attr("aria-label", name.as_str()))
    }
}

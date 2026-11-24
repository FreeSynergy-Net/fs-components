//! The move buttons of the frame at the side: one step to the previous column, one to the next, and "Move to..." with the list of
//! the other columns. They are the commands `board.to:<column>` (the same ones a context menu runs), so the click-to-move gives the
//! same [`MoveRequest`](crate::requests::MoveRequest) as the mouse.
//!
//! Pattern: Adapter (split from `view_line.rs`) + Command (each button runs a command id). No fixed width and no number of columns
//! side by side is assumed: the row wraps inside the element (`flex_wrap`), the list holds as many columns as the board has. A
//! button that has no target (the first column has no "previous") is drawn disabled; without the right to move every button is
//! drawn locked, says "soon" and a press only says so (ADR 0096 H-K5).

use crate::Operation;
use crate::bench::BoardBench;
use crate::commands;
use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

impl BoardBench {
    /// The row of move buttons for `card` (empty when the card is not on the board).
    pub(crate) fn move_buttons(&self, card: &str) -> Div {
        let state = self.snapshot();
        let lanes = state.lanes();
        let id = &self.id;
        let row = div()
            .id(format!("{id}-detail-move"))
            .class("fs-board-detail-move")
            .flex_col()
            .gap_px(8.0);
        let Some(n) = lanes
            .iter()
            .position(|l| l.items.iter().any(|i| i.key == card))
        else {
            return row;
        };
        let live = self.source.supports(Operation::Move);
        let prev = n.checked_sub(1).map(|p| lanes[p].key.clone());
        let next = lanes.get(n + 1).map(|l| l.key.clone());
        let open = state.move_menu_open();
        let mut buttons = div()
            .class("fs-board-detail-move-row")
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap_px(8.0)
            .child(self.move_button("prev", "\u{2190}", card, prev, live))
            .child(self.move_button("next", "\u{2192}", card, next, live));
        let this = self.clone();
        buttons = buttons.child(Self::locked_if(
            div().id(format!("{id}-detail-moveto")).w_fit().child(
                cn::button(self.labels.move_to.clone())
                    .variant(ButtonVariant::Ghost)
                    .on_click(move |_| this.toggle_move_menu(live)),
            ),
            !live,
        ));
        if !live {
            buttons = buttons.child(
                text(self.labels.soon.clone())
                    .id(format!("{id}-detail-move-soon"))
                    .class("fs-board-soon"),
            );
        }
        let mut row = row.child(buttons);
        if open && live {
            let mut list = div()
                .id(format!("{id}-detail-to"))
                .class("fs-board-detail-move-row")
                .flex_row()
                .flex_wrap()
                .gap_px(8.0);
            for (m, head) in state.heads().iter().enumerate() {
                if m != n {
                    list = list.child(self.move_button(
                        &format!("to-{}", head.key),
                        &head.title,
                        card,
                        Some(head.key.clone()),
                        live,
                    ));
                }
            }
            row = row.child(list);
        }
        row
    }

    /// One button: runs `board.to:<target>` for `card`. No target: disabled. No right to move: locked, a press says "soon".
    fn move_button(
        &self,
        tag: &str,
        word: &str,
        card: &str,
        target: Option<String>,
        live: bool,
    ) -> Div {
        let this = self.clone();
        let card = card.to_owned();
        let has_target = target.is_some();
        let button = div().id(format!("{}-detail-{tag}", self.id)).w_fit().child(
            cn::button(word.to_owned())
                .variant(ButtonVariant::Ghost)
                .on_click(move |_| match (&target, live) {
                    (_, false) => this.say_move_soon(),
                    (Some(column), true) => {
                        this.lock().set_move_menu(false);
                        this.run(&commands::to_column(column), &card);
                    }
                    (None, true) => {}
                }),
        );
        if has_target {
            Self::locked_if(button, !live)
        } else {
            button
                .class("fs-board-locked")
                .attr("aria-disabled", "true")
        }
    }

    fn locked_if(button: Div, locked: bool) -> Div {
        if locked {
            button
                .class("fs-board-locked")
                .attr("data-locked", "true")
                .attr("aria-disabled", "true")
        } else {
            button
        }
    }

    fn toggle_move_menu(&self, live: bool) {
        if !live {
            self.say_move_soon();
            return;
        }
        {
            let mut state = self.lock();
            let open = state.move_menu_open();
            state.set_move_menu(!open);
        }
        self.refresh();
    }
}

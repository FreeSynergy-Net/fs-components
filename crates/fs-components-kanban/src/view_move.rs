//! The move buttons of the frame at the side: one step to the previous column, one to the next, and "Move to..." with the list of
//! the other columns. They are the commands `board.to:<column>` (the same ones a context menu runs), so the click-to-move gives the
//! same [`MoveRequest`](crate::requests::MoveRequest) as the mouse.
//!
//! Pattern: Adapter (split from `view_line.rs`) + Command (each button runs a command id). No fixed width and no number of columns
//! side by side is assumed: the row wraps inside the element (`flex_wrap`), the list holds as many columns as the board has. A
//! button that has no target (the first column has no "previous") is drawn disabled with its reason in a tooltip; without the right
//! to move every button is drawn disabled (unsupported) the same way, same size, no extra line (ADR 0096 H-K5, `disabled::wrap`).

use crate::Operation;
use crate::bench::BoardBench;
use crate::commands;
use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use fs_components_widgets::disabled::{Disabled, Why};

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
        let prev = n.checked_sub(1).map(|p| lanes[p].key.as_str());
        let next = lanes.get(n + 1).map(|l| l.key.as_str());
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
        let mut to = cn::button(self.labels.move_to.clone()).variant(ButtonVariant::Ghost);
        if live {
            to = to.on_click(move |_| this.toggle_move_menu(true));
        }
        let to = div().id(format!("{id}-detail-moveto")).w_fit().child(to);
        buttons = buttons.child(if live {
            to
        } else {
            Disabled::new(
                &format!("{id}-detail-moveto"),
                Why::Unsupported,
                &self.labels.why,
            )
            .wrap(to)
        });
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
                        Some(head.key.as_str()),
                        live,
                    ));
                }
            }
            row = row.child(list);
        }
        row
    }

    /// One button: runs `board.to:<target>` for `card`. No target: disabled, the reason is the missing column. No right to move:
    /// disabled (unsupported); the press runs nothing, the tooltip says why.
    fn move_button(
        &self,
        tag: &str,
        word: &str,
        card: &str,
        target: Option<&str>,
        live: bool,
    ) -> Div {
        let id = format!("{}-detail-{tag}", self.id);
        let mut inner = cn::button(word.to_owned()).variant(ButtonVariant::Ghost);
        if let (Some(column), true) = (target, live) {
            let (this, card, column) = (self.clone(), card.to_owned(), column.to_owned());
            inner = inner.on_click(move |_| {
                this.lock().set_move_menu(false);
                this.run(&commands::to_column(&column), &card);
            });
        }
        let button = div().id(&id).w_fit().child(inner);
        let why = match (target.is_some(), live) {
            (_, false) => Why::Unsupported,
            (false, true) if tag == "prev" => Why::Now(self.labels.edge_prev.clone()),
            (false, true) => Why::Now(self.labels.edge_next.clone()),
            (true, true) => return button,
        };
        Disabled::new(&id, why, &self.labels.why).wrap(button)
    }

    fn toggle_move_menu(&self, live: bool) {
        if !live {
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

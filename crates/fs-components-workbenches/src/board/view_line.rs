//! The line under the board and the frame at the side.
//!
//! Pattern: Adapter (split from `view.rs`).

use super::bench::BoardBench;
use super::model::ColumnAction;
use super::state::BoardState;
use super::view::DETAIL_W;
use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use std::sync::Arc;
impl BoardBench {
    pub(super) fn line(&self, state: &BoardState) -> Option<Div> {
        let id = &self.id;
        let button =
            |tag: &str, word: &str, variant: ButtonVariant, f: Box<dyn Fn() + Send + Sync>| {
                div().id(format!("{id}-{tag}")).w_fit().child(
                    cn::button(word.to_string())
                        .variant(variant)
                        .on_click(move |_| f()),
                )
            };
        if let Some((_, question)) = state.question() {
            let (yes, no) = (self.clone(), self.clone());
            return Some(
                div()
                    .id(format!("{id}-question"))
                    .class("fs-board-line")
                    .flex_row()
                    .items_center()
                    .child(text(question.text.clone()).class("fs-board-line-text"))
                    .child(button(
                        "yes",
                        &self.labels.yes,
                        ButtonVariant::Primary,
                        Box::new(move || yes.answer_question(true)),
                    ))
                    .child(button(
                        "no",
                        &self.labels.no,
                        ButtonVariant::Ghost,
                        Box::new(move || no.answer_question(false)),
                    )),
            );
        }
        if let Some(cards) = self.pending_many() {
            let (yes, no) = (self.clone(), self.clone());
            return Some(
                div()
                    .id(format!("{id}-many"))
                    .class("fs-board-line")
                    .flex_row()
                    .items_center()
                    .child(text(self.labels.create_many(cards.len())).class("fs-board-line-text"))
                    .child(button(
                        "many-yes",
                        &self.labels.create,
                        ButtonVariant::Primary,
                        Box::new(move || yes.answer_many(true)),
                    ))
                    .child(button(
                        "many-no",
                        &self.labels.no,
                        ButtonVariant::Ghost,
                        Box::new(move || no.answer_many(false)),
                    )),
            );
        }
        let notice = state.notice()?;
        let mut line = div()
            .id(format!("{id}-notice"))
            .class("fs-board-line")
            .flex_row()
            .items_center()
            .child(text(notice.text.clone()).class("fs-board-line-text"));
        if notice.refused {
            line = line.class("fs-board-line-refused");
        }
        if notice.undo.is_some() {
            let undo = self.clone();
            line = line.child(button(
                "undo",
                &self.labels.undo,
                ButtonVariant::Ghost,
                Box::new(move || undo.undo()),
            ));
        }
        Some(line)
    }

    pub(super) fn detail_frame(
        &self,
        card: &str,
        frame: &super::detail::DetailFrame,
        height: f32,
    ) -> Div {
        let id = &self.id;
        let close = self.clone();
        let mut d = div()
            .id(format!("{id}-detail"))
            .class("fs-board-detail")
            .flex_col()
            .w(DETAIL_W)
            .h(height)
            .flex_shrink_0()
            .child(
                div()
                    .class("fs-board-detail-top")
                    .flex_row()
                    .items_center()
                    .child(text(frame.path.clone()).class("fs-board-detail-path"))
                    .child(div().flex_1())
                    .child(
                        div()
                            .id(format!("{id}-detail-close"))
                            .class("fs-board-detail-close")
                            .attr("role", "button")
                            .attr("aria-label", self.labels.close.as_str())
                            .cursor_pointer()
                            .child(text("\u{2715}"))
                            .on_click(move |_| close.close_detail()),
                    ),
            )
            .child(text(frame.title.clone()).class("fs-board-detail-title"));
        for (name, value) in &frame.properties {
            let value_text = if value.is_empty() {
                text(self.labels.detail_set.clone()).class("fs-board-detail-unset")
            } else {
                text(value.clone()).class("fs-board-detail-value")
            };
            d = d.child(
                div()
                    .class("fs-board-detail-prop")
                    .flex_row()
                    .child(text(name.clone()).class("fs-board-detail-name"))
                    .child(value_text),
            );
        }
        for (i, (name, slot)) in frame.elements.iter().enumerate() {
            let line = slot.line();
            let value_text = if line.is_empty() {
                text(self.labels.detail_set.clone()).class("fs-board-detail-unset")
            } else {
                text(line).class("fs-board-detail-value")
            };
            d = d.child(
                div()
                    .id(format!("{id}-detail-element-{i}"))
                    .class("fs-board-detail-prop")
                    .class("fs-board-detail-element")
                    .flex_row()
                    .child(text(name.clone()).class("fs-board-detail-name"))
                    .child(value_text),
            );
        }
        if let (Some((pid, word)), Some(content)) = (&frame.primary, &self.detail) {
            let (content, card, pid) = (Arc::clone(content), card.to_owned(), pid.clone());
            d = d.child(
                div()
                    .class("fs-board-detail-actions")
                    .id(format!("{id}-detail-primary"))
                    .w_fit()
                    .child(
                        cn::button(word.clone())
                            .variant(ButtonVariant::Primary)
                            .on_click(move |_| content.primary(&card, &pid)),
                    ),
            );
        }
        d
    }

    /// What a column's short menu offers, as actions the program can do: rename, set the limit, fold, edit the columns.
    pub fn column_menu_actions() -> [ColumnAction; 4] {
        [
            ColumnAction::Rename,
            ColumnAction::SetLimit,
            ColumnAction::Collapse,
            ColumnAction::EditColumns,
        ]
    }
}

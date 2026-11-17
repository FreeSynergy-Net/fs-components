//! The board as elements: the title, the columns with their cards, the foot of each column, the layer of the carried card, the
//! frame at the side and the line under the board.
//!
//! Pattern: Adapter (the state in, Blinc layout out) + Template method (`render` fixes the order). Sizes come from `hit.rs`: a
//! card is `CARD_H` high whatever it holds and a column `COLUMN_W` wide, so the rectangles the pointer is tested against are
//! exactly the ones drawn. The cards and the layer of the carried card are two live areas: moving the pointer draws only the
//! layer, never the cards, so Blinc keeps the gesture on the element that was pressed.
//! Look (b7, lessons of the pictures): big column names, readable 15 px titles, a quiet card without coloured badges (one grey line
//! of facts), the work-in-progress mark as plain text (a warning colour and a symbol only above the limit), generous space.

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_kanban::Card;
use fs_components_widgets::keys::bind_keys;

use super::bench::BoardBench;
use super::hit::{CARD_GAP, CARD_H, COLUMN_GAP, COLUMN_W, FOOT_H, Geometry, HEAD_H, PAD};
use super::model::{ColumnAction, ContextTarget};
use super::state::BoardState;
use super::wip::WipMark;

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
        let mut head = div().class("fs-board-title-row").w(main_w);
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
        let (inner_w, inner_h) = geometry.size();
        let (inner_w, inner_h) = (inner_w.max(width), inner_h.max(height));
        let mut columns = div()
            .class("fs-board-columns")
            .flex_row()
            .gap_px(COLUMN_GAP)
            .pl(PAD / 4.0)
            .pt(PAD / 4.0);
        for n in 0..state.heads().len() {
            columns = columns.child(self.column(state, n));
        }
        let this = self.clone();
        let layer = self.drag_live.region("drag", move || this.drag_layer());
        let on_empty = self.clone();
        let content = div()
            .class("fs-board-content")
            .w(inner_w)
            .min_w(inner_w)
            .h(inner_h)
            .flex_shrink_0()
            .child(layer)
            .child(columns)
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

    fn column(&self, state: &BoardState, n: usize) -> Div {
        let head = &state.heads()[n];
        let lane = &state.lanes()[n];
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
        name_row = name_row
            .child(div().flex_1())
            .child(
                div()
                    .id(format!("{id}-menu-{}", head.key))
                    .class("fs-board-head-menu")
                    .attr("role", "button")
                    .attr("aria-label", self.labels.column_menu.as_str())
                    .cursor_pointer()
                    .child(text("\u{22ef}"))
                    .on_click(move |ctx| {
                        menu.context_by_click(
                            &ContextTarget::Column(menu_key.clone()),
                            (ctx.mouse_x, ctx.mouse_y),
                        );
                    }),
            )
            .on_right_click(move |ctx| {
                this.context(
                    &ContextTarget::Column(key.clone()),
                    (ctx.mouse_x, ctx.mouse_y),
                );
            });
        let mut cards = div()
            .class("fs-board-cards")
            .flex_col()
            .gap_px(CARD_GAP)
            .w(COLUMN_W)
            .attr("role", "list");
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
            .class("fs-board-column")
            .flex_col()
            .w(COLUMN_W)
            .flex_shrink_0()
            .attr("role", "group")
            .child(name_row)
            .child(cards)
            .child(self.foot(state, &head.key))
    }

    fn wip(&self, column: &str, mark: &WipMark) -> Div {
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

    fn card(&self, state: &BoardState, card: &Card) -> Div {
        let id = &self.id;
        let key = card.key.clone();
        let mut meta: Vec<String> = Vec::new();
        if let Some(sub) = &card.subtitle {
            meta.push(sub.clone());
        }
        if let Some(due) = &card.due {
            meta.push(due.text.clone());
        }
        if let Some(who) = &card.assignee {
            meta.push(who.clone());
        }
        let carried = state.carried().is_some_and(|(k, ..)| k == card.key);
        let mut d = div()
            .id(format!("{id}-card-{}", card.key))
            .class("fs-board-card")
            .attr("role", "listitem")
            .flex_col()
            .w(COLUMN_W)
            .h(CARD_H)
            .overflow_clip()
            .child(text(card.title.clone()).class("fs-board-card-title"));
        if !meta.is_empty() {
            d = d.child(
                text(meta.join("  \u{b7}  "))
                    .class("fs-board-card-meta")
                    .no_wrap(),
            );
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
        let (k1, k2, k4) = (key.clone(), key.clone(), key);
        d.cursor_pointer()
            .on_mouse_down(move |ctx| {
                if ctx.mouse_button == 0 {
                    down.press(&k1, (ctx.mouse_x, ctx.mouse_y));
                }
            })
            .on_drag(move |ctx| {
                drag.drag(&k2, (ctx.mouse_x, ctx.mouse_y), (ctx.local_x, ctx.local_y));
            })
            .on_drag_end(move |_| end.release())
            // A click without a carry opens the card; after a carry the machine is idle and this does nothing.
            .on_click(move |_| click.release())
            .on_right_click(move |ctx| {
                menu.context(&ContextTarget::Card(k4.clone()), (ctx.mouse_x, ctx.mouse_y));
            })
    }

    // ---- the foot: quick entry ----------------------------------------------------------------------------------------------

    fn foot(&self, state: &BoardState, column: &str) -> Div {
        let id = &self.id;
        if !self.source.supports().quick_add {
            return div().h(FOOT_H);
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

    // ---- the carried card ---------------------------------------------------------------------------------------------------

    /// The layer over the columns: the target column lit, a line where the card would land, the card at the pointer, the reason
    /// when the place does not take it. Empty when nothing is carried.
    fn drag_layer(&self) -> Div {
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
            .absolute()
            .left(pointer.0 - COLUMN_W / 2.0)
            .top(pointer.1 - 20.0)
            .w(COLUMN_W)
            .pointer_events_none()
            .child(text(title).class("fs-board-card-title"));
        if let Some(reason) = state.blocked() {
            ghost = ghost.class("fs-board-ghost-blocked").child(
                text(reason.text.clone())
                    .class("fs-board-ghost-reason")
                    .id(format!("{}-reason", self.id)),
            );
        }
        layer.child(ghost)
    }

    // ---- under the board ------------------------------------------------------------------------------------------------------

    fn line(&self, state: &BoardState) -> Option<Div> {
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

    // ---- the frame at the side --------------------------------------------------------------------------------------------------

    fn detail_frame(&self, card: &str, frame: &super::detail::DetailFrame, height: f32) -> Div {
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

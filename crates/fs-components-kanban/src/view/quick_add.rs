//! The quick-add row at the end of a column: a plus, a field for the title and the button that adds the card.
//!
//! Pattern: State (supported or locked) as one function, like the other controls — without `Operation::AddCard` the row is ONE locked
//! control that says "comes later" and holds no field; with it the row takes the title (Enter in the field or the button adds).

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_styles::Glyph;

use super::controls::control;
use crate::handle::KanbanBoard;
use crate::model::Operation;

/// The quick-add row of column `key`.
pub fn quick_add(b: &KanbanBoard, key: &str) -> Div {
    let (s, pal) = (b.look.style, b.look.palette());
    let supported = b.source.supports(Operation::AddCard);
    if !supported {
        return control(b, &format!("kb-add-card-{key}"), "kanban.add_card", false, true, || {});
    }
    let (add, enter, swallow) = (b.clone(), b.clone(), b.clone());
    let (k_add, k_enter) = (key.to_owned(), key.to_owned());
    // Enter in the field adds, as the button does. The field kept in the board has no handler (the board would hold itself); the
    // copy that is built here has one and goes away with the element.
    let field = b
        .drafts
        .field(key, &b.say.say("kanban.add_card_hint"))
        .on_submit(Arc::new(move || {
            enter.submit_quick(&k_enter);
        }));
    div()
        .id(format!("kb-quick-{key}"))
        .flex_row()
        .items_center()
        .gap_px(s.spacing.xs)
        .w_full()
        .child(b.look.icon(Glyph::Plus, Some(s.text.caption), pal.text_weak))
        .child(
            div()
                .id(format!("kb-quick-field-{key}"))
                .flex_grow_value(1.0)
                .child(field.build()),
        )
        .child(control(
            b,
            &format!("kb-quick-ok-{key}"),
            "kanban.add_card_ok",
            true,
            true,
            move || {
                add.submit_quick(&k_add);
            },
        ))
        // A click inside the row is no click on the column below (that would put a held card down).
        .on_click(move |_| swallow.swallow())
}

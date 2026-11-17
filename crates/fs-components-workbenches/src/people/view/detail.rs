//! The person: read, or the form while editing.
//!
//! Pattern: none (pure functions of what is shown) — a person, the form, a failure or nothing. Actions the source cannot
//! do are not offered.

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Glyph, Tone, icon};

use super::actions::{Act, PeopleActions};
use super::form::form;
use super::person::{head, person};
use crate::input_slots::InputSlots;
use crate::page_scroll::PageScroll;
use crate::people::shown::Shown;
use crate::people::state::PeopleState;

/// A button with an element id on a wrapper, so a program or a test can find it.
pub(super) fn button(id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    let act = std::sync::Arc::clone(act);
    div().id(id).w_fit().child(
        cn::button(label.to_string())
            .variant(variant)
            .on_click(move |_| act()),
    )
}

/// A button named `<name>-<what>`.
pub(in crate::people) fn button_for(
    name: &str,
    what: &str,
    label: &str,
    variant: ButtonVariant,
    act: &Act,
) -> Div {
    button(format!("{name}-{what}"), label, variant, act)
}

/// The area right of the list. The bar and the page scroll together: in a small window there is no room to keep them apart. The
/// question of a delete or a discard stands at the foot.
pub(in crate::people) fn detail(
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    slots: &InputSlots,
    scroller: &PageScroll,
    own_menus: bool,
) -> Div {
    let editing = state.editing().is_some();
    // Editing: the buttons stand on top as always, but come last in the tree, so Tab walks the fields
    // in reading order, then Save, then Cancel (a form's usual order). The column is drawn in reverse
    // for that; so is the bar.
    let mut column = div().class("fs-people-detail").w_full();
    column = if editing {
        column.flex_col_reverse()
    } else {
        column
            .flex_col()
            .child(head(state, name, actions, own_menus))
    };
    let page = div().class("fs-people-page").flex_col().w_full();
    let page = if let Some(edit) = state.editing() {
        form(
            edit,
            state.labels(),
            name,
            actions,
            slots,
            &|key| state.problem(key),
            page,
        )
    } else {
        match state.shown() {
            Shown::Person { sheet, .. } => person(sheet, state, name, actions, page),
            Shown::Failed { reason, .. } => {
                page.child(div().class("fs-people-failed").child(text(reason.clone())))
            }
            _ => page
                .items_center()
                .child(icon(Glyph::Person, 48.0, Tone::Muted))
                .child(text(&state.labels().nothing_selected)),
        }
    };
    let notice = state
        .notice()
        .map(|notice| div().class("fs-people-notice").child(text(notice)));
    let deed = deed(state, name, actions);
    if editing {
        // Tree order; drawn from the bottom up.
        column = column.child(page);
        column = column.children(notice);
        column = column.child(bar(state, name, actions));
    } else {
        column = column.children(notice).child(page);
    }
    let what = match state.editing() {
        Some(edit) => format!("edit-{}", edit.id),
        None => state.selected().unwrap_or_default().to_string(),
    };
    // The question stands at the foot of the area, outside of what scrolls: a menu line pressed twice at the top never reaches it.
    div()
        .flex_col()
        .flex_1()
        .min_h(0.0)
        .w_full()
        .child(
            div()
                .flex_1()
                .min_h(0.0)
                .w_full()
                .child(scroller.scroller(&what).w_full().h_full().content(column)),
        )
        .children(deed)
}

/// The second action of a question, on a line of its own at the foot of the area: a second press at the place of the first (a
/// line of a menu at the top, the Cancel of the bar) can never be it.
fn deed(state: &PeopleState, name: &str, actions: &PeopleActions) -> Option<Div> {
    let labels = state.labels();
    let button = if state.editing().is_some_and(|edit| edit.discard_asked) {
        button(
            format!("{name}-discard"),
            &labels.discard_really,
            ButtonVariant::Destructive,
            &actions.confirm_discard,
        )
    } else if state.can_delete() && state.delete_asked() {
        button(
            format!("{name}-really"),
            &labels.delete_really,
            ButtonVariant::Destructive,
            &actions.confirm_delete,
        )
    } else {
        return None;
    };
    let mut row = div().class("fs-people-really").flex_row().child(button);
    if state.can_delete() && state.delete_asked() && state.editing().is_none() {
        row = row.child(self::button(
            format!("{name}-keep"),
            &labels.delete_keep,
            ButtonVariant::Secondary,
            &actions.keep,
        ));
    }
    Some(row)
}

/// The bar of an edit: Save, Cancel (or Keep editing). They stand at the left, away from the pencil at the right, so the second press of
/// a double click on the pencil lands on nothing.
fn bar(state: &PeopleState, name: &str, actions: &PeopleActions) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div().class("fs-people-bar").flex_row().flex_wrap();
    if let Some(edit) = state.editing() {
        let cancel_or_keep = if edit.discard_asked {
            button(
                id("keep-editing"),
                &labels.discard_keep,
                ButtonVariant::Secondary,
                &actions.keep_editing,
            )
        } else {
            button(
                id("cancel"),
                &labels.cancel,
                ButtonVariant::Secondary,
                &actions.cancel,
            )
        };
        // Save and Cancel stand at the left, the pencil of the person at the right: the second press of a double click on the
        // pencil lands on the empty middle, never on Save. Tree order is drawing order here: Save, Cancel, spacer.
        bar = bar
            .child(button(
                id("save"),
                &labels.save,
                ButtonVariant::Primary,
                &actions.save,
            ))
            .child(cancel_or_keep)
            .child(div().class("fs-people-spacer"));
    }
    bar
}

/// `row` with the markings added.
pub(super) fn with_marks(mut row: Div, marks: &[String]) -> Div {
    for mark in marks {
        row = row.child(div().class("fs-people-mark").child(text(mark)));
    }
    row
}

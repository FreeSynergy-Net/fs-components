//! The person as it is read: the bar with the pencil and the three dots, then the page of the record.
//!
//! Pattern: Adapter — the sheet of the state as the `detail-page` element (face, name, cards), centred. A field says what it holds
//! (`FieldKind`), which gives its actions: write a mail for an address, copy for every value. The elements report presses through
//! [`PeopleActions`]; nothing here changes state.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_commands::CommandId;
use fs_components_widgets::detail_page;
use fs_components_widgets::dropdown::dropdown;
use fs_components_widgets::{CardAction, Glyph, HeadBar, head_button, head_face};

use super::actions::{Act, MenuFor, PeopleActions};
use crate::people::commands::{COPY, EDIT, MAIL};
use crate::people::model::{FieldKind, PersonField, PersonSheet};
use crate::people::shown::Shown;
use crate::people::state::PeopleState;

/// The bar above the person: the pencil and the three dots at the right, nothing at the left.
pub(super) fn head(
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    own_menus: bool,
) -> Div {
    let labels = state.labels();
    let mut bar = HeadBar::new();
    if matches!(state.shown(), Shown::Person { .. }) {
        if state.can_edit() {
            let run = Arc::clone(&actions.run);
            let press: Act = Arc::new(move || run(EDIT, None));
            bar = bar.right(head_button(
                &format!("{name}-edit"),
                Glyph::Edit,
                &labels.edit,
                false,
                &press,
            ));
        }
        bar = bar.right(person_menu(state, name, actions, own_menus));
    }
    bar.build().class("fs-people-head-bar")
}

/// The three dots. With a program's menu (a hook) the press asks the program to open it; else the workbench hangs its own panel on
/// the button, made of the same lines.
fn person_menu(state: &PeopleState, name: &str, actions: &PeopleActions, own_menus: bool) -> Div {
    let labels = state.labels();
    if own_menus {
        let items = state.person_menu();
        if items.is_empty() {
            return div();
        }
        let run = Arc::clone(&actions.run);
        let spoken = labels.person_menu.clone();
        let holder = div().id(format!("{name}-menu-person")).w_fit();
        return holder.child(
            dropdown(move |_| head_face(Glyph::More, &spoken, true))
                .width(220.0)
                .commands(
                    &items,
                    &|key| labels.say(key, state),
                    move |command: &CommandId| run(command.as_str(), None),
                )
                .into_element(),
        );
    }
    let menu = Arc::clone(&actions.menu);
    let press: Act = Arc::new(move || {});
    // The press hands the place of the button to the program, as a menu opened by the keyboard would.
    head_button(
        &format!("{name}-menu-person"),
        Glyph::More,
        &labels.person_menu,
        true,
        &press,
    )
    .on_click(move |ctx| {
        menu(
            MenuFor::Person,
            fs_components_commands::Trigger::MenuKey {
                element: fs_components_commands::Area {
                    x: ctx.bounds_x,
                    y: ctx.bounds_y,
                    width: ctx.bounds_width,
                    height: ctx.bounds_height,
                },
            },
        );
    })
}

/// The actions of the card of a value: write a mail for an address; copy for everything.
fn card_actions(
    field: &PersonField,
    value: &str,
    state: &PeopleState,
    run: &super::actions::RunCmd,
) -> Vec<CardAction> {
    let labels = state.labels();
    let value = value.trim().to_owned();
    let mut out = Vec::new();
    if field.kind == FieldKind::Mail {
        let (run, value) = (Arc::clone(run), value.clone());
        out.push(CardAction::new(
            Glyph::Share,
            labels.mail_write.clone(),
            Arc::new(move || run(MAIL, Some(&value))),
        ));
    }
    let run = Arc::clone(run);
    out.push(CardAction::new(
        Glyph::Copy,
        labels.copy.clone(),
        Arc::new(move || run(COPY, Some(&value))),
    ));
    out
}

/// The person, centred: the page of the record (`detail-page`) with the actions of this workbench on its cards.
pub(super) fn person(
    sheet: &PersonSheet,
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    page: Div,
) -> Div {
    page.items_center().child(detail_page::page(
        sheet,
        name,
        &state.labels().no_details,
        &|field, value| card_actions(field, value, state, &actions.run),
    ))
}

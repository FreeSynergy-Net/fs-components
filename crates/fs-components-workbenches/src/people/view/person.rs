//! The person as it is read: the bar with the pencil and the three dots, a big round face, the name, the fields as cards.
//!
//! Pattern: Adapter — the sheet of the state as `person-face` and `field-card` elements, centred. A field says what it holds
//! (`FieldKind`), which gives its symbol and its actions: write a mail for an address, copy for every value. Fields with no value
//! are not drawn. The elements report presses through [`PeopleActions`]; nothing here changes state.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_commands::CommandId;
use fs_components_widgets::dropdown::dropdown;
use fs_components_widgets::{
    CardAction, FieldCard, Glyph, HeadBar, PersonFace, head_button, head_face,
};

use super::actions::{Act, MenuFor, PeopleActions};
use super::detail::with_marks;
use crate::people::commands::{COPY, EDIT, MAIL};
use crate::people::model::{FieldKind, PersonField, PersonSection, PersonSheet};
use crate::people::shown::Shown;
use crate::people::state::PeopleState;

/// The diameter of the big face, in pixels.
const FACE: f32 = 112.0;

/// The bar above the person: the pencil and the three dots at the right, nothing at the left.
pub(super) fn head(state: &PeopleState, name: &str, actions: &PeopleActions, own_menus: bool) -> Div {
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
        return holder.child(dropdown(move |_| head_face(Glyph::More, &spoken, true))
            .width(220.0)
            .commands(
                &items,
                &|key| labels.say(key, state),
                move |command: &CommandId| run(command.as_str(), None),
            )
            .into_element());
    }
    let menu = Arc::clone(&actions.menu);
    let press: Act = Arc::new(move || {});
    // The press hands the place of the button to the program, as a menu opened by the keyboard would.
    head_button(&format!("{name}-menu-person"), Glyph::More, &labels.person_menu, true, &press).on_click(move |ctx| {
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

/// The symbol of a kind of field.
pub(in crate::people) fn glyph_of(kind: FieldKind) -> Glyph {
    match kind {
        FieldKind::Mail => Glyph::Mail,
        FieldKind::Phone => Glyph::Phone,
        FieldKind::Address => Glyph::Place,
        FieldKind::Date => Glyph::Calendar,
        _ => Glyph::Info,
    }
}

/// The actions of the card of a field: write a mail for an address; copy for everything.
pub(in crate::people) fn card_actions(field: &PersonField, state: &PeopleState, run: &super::actions::RunCmd) -> Vec<CardAction> {
    let labels = state.labels();
    let value = field.value.trim().to_owned();
    let mut out = Vec::new();
    if field.effective_kind() == FieldKind::Mail {
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

/// The person, centred: a big round face, the name, the fields as cards.
pub(super) fn person(
    sheet: &PersonSheet,
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    page: Div,
) -> Div {
    let face = PersonFace::new(&sheet.title)
        .image(sheet.image.clone())
        .size(FACE)
        .build();
    let head = div()
        .class("fs-people-head")
        .flex_col()
        .items_center()
        .child(div().class("fs-people-face").child(face))
        .child(
            div()
                .class("fs-people-title")
                .child(text(&sheet.title).class("fs-people-title-text")),
        );
    let page = page
        .items_center()
        .child(with_marks(head, &sheet.marks));
    let mut cards = div().class("fs-people-cards").flex_col().w_full();
    let mut shown = 0usize;
    for (s, section) in sheet.sections.iter().enumerate() {
        if let Some(element) = section_element(section, state, name, s, actions) {
            cards = cards.child(element);
            shown += 1;
        }
    }
    if shown == 0 {
        cards = cards.child(div().class("fs-people-nodetails").child(text(&state.labels().no_details)));
    }
    page.child(cards)
}

fn section_element(
    section: &PersonSection,
    state: &PeopleState,
    name: &str,
    number: usize,
    actions: &PeopleActions,
) -> Option<Div> {
    let fields: Vec<&PersonField> = section
        .fields
        .iter()
        .filter(|f| !f.value.trim().is_empty())
        .collect();
    if fields.is_empty() {
        return None;
    }
    let mut element = div().class("fs-people-section").flex_col().w_full();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-people-section-title")
                .child(text(&section.title)),
        );
    }
    for (f, field) in fields.into_iter().enumerate() {
        let mut card = FieldCard::new(glyph_of(field.effective_kind()), &field.label, &field.value);
        for action in card_actions(field, state, &actions.run) {
            card = card.action(action);
        }
        element = element.child(card.build(&format!("{name}-field-{number}-{f}")));
    }
    Some(element)
}

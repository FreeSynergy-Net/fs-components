//! The list of people: rows with a round face and the name, as in GNOME Contacts; no table.
//!
//! Pattern: Adapter — the rows of the state as plain elements in one scroll area. Every row has the same height (a theme
//! variable, `--space-14`), so the list reads evenly whatever the names are. A row is a tab stop: Enter chooses it, the menu key and
//! Shift+F10 open its menu at the row, a right click opens it at the pointer. In the mode of choosing a row shows a check
//! circle at the left and a press marks it. The rows are a shared copy, so the list never reaches back into the locked state while
//! it scrolls. The elements are plain, not `cn`, because this is drawn in a live area (see the Blinc notes in `DESIGN.md`).

use std::sync::Arc;
use std::sync::atomic::Ordering;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_commands::{Point, Trigger};
use fs_components_widgets::{Glyph, PersonFace, Tone, icon};

use super::actions::{MenuFor, PeopleActions};
use crate::people::commands::mark_id;
use super::keys::{KeyActs, with_keys};
use crate::page_scroll::PageScroll;
use crate::people::labels::PeopleLabels;
use crate::people::model::PersonRow;
use crate::people::state::PeopleState;

/// The diameter of the face in a row, in pixels.
const FACE: f32 = 40.0;

/// The class of a row: selected rows are told apart by the class, never by a colour alone (the check circle and `aria-selected` say it
/// too).
pub(in crate::people) fn row_class(selected: bool, marked: bool) -> &'static str {
    match (selected, marked) {
        (true, _) => "fs-contact-row fs-contact-row-selected",
        (false, true) => "fs-contact-row fs-contact-row-marked",
        (false, false) => "fs-contact-row",
    }
}

/// The text of a row: the first cell, which is the name.
pub(in crate::people) fn row_name(row: &PersonRow) -> &str {
    row.cells.first().map_or("", String::as_str)
}

/// The title of the list and the rows, or why there are none.
pub(in crate::people) fn list(
    state: &PeopleState,
    name: &str,
    scroller: &PageScroll,
    actions: &PeopleActions,
) -> Div {
    let labels = state.labels();
    let area = div().id(name.to_string()).class("fs-people-list").flex_col().w_full().h_full().min_h(0.0);
    if let (Some(problem), true) = (state.list_problem(), state.rows().is_empty()) {
        return area.child(div().class("fs-people-failed").child(text(problem)));
    }
    let mut column = div().class("fs-people-rows").flex_col().w_full();
    column = column.child(
        div()
            .id(format!("{name}-list-title"))
            .class("fs-people-section-head")
            .child(text(state.list_title()).no_cursor()),
    );
    if state.rows().is_empty() {
        let empty = if state.term().is_empty() {
            labels.nobody.clone()
        } else {
            PeopleLabels::fill(&labels.nobody_found, state.term())
        };
        column = column.child(
            div()
                .class("fs-people-empty")
                .flex_col()
                .items_center()
                .child(icon(Glyph::Person, 40.0, Tone::Muted))
                .child(text(empty)),
        );
    }
    for row in state.rows().iter() {
        column = column.child(one_row(state, name, row, actions));
    }
    // A problem of a further page stands under the rows, and the rows stay.
    if let (Some(problem), false) = (state.list_problem(), state.rows().is_empty()) {
        column = column.child(div().class("fs-people-failed").child(text(problem)));
    }
    if state.has_more() {
        let more = Arc::clone(&actions.more);
        column = column.child(
            div()
                .id(format!("{name}-more"))
                .class("fs-people-more-button")
                // A tab stop: the window turns Enter and Space on it into this click.
                .attr("tabindex", "0")
                .on_click(move |_| more())
                .child(text(&labels.more).no_cursor()),
        );
    }
    let empty_menu = Arc::clone(&actions.menu);
    let claimed = Arc::clone(&actions.claimed);
    let scroll = scroller
        .scroller(&format!("list-{}", state.list_generation()))
        .w_full()
        .h_full()
        .content(column);
    area.child(
        div()
            .w_full()
            .h_full()
            .min_h(0.0)
            // A right click on space between and below the rows: the menu of empty space.
            .on_right_click(move |ctx| {
                if claimed.swap(false, Ordering::Relaxed) {
                    return;
                }
                empty_menu(
                    MenuFor::Empty,
                    Trigger::Pointer {
                        at: Point {
                            x: ctx.mouse_x,
                            y: ctx.mouse_y,
                        },
                        shift: ctx.shift,
                    },
                );
            })
            .child(scroll),
    )
}

fn one_row(state: &PeopleState, name: &str, row: &PersonRow, actions: &PeopleActions) -> Div {
    let selected = state.selected() == Some(row.key.as_str());
    let marked = state.is_marked(&row.key);
    let title = row_name(row);
    let mut line = div()
        .id(format!("{name}-{}", row.key))
        .class(row_class(selected, marked))
        .flex_row()
        .items_center()
        .w_full()
        .attr("role", "option")
        .attr("aria-selected", if selected || marked { "true" } else { "false" });
    if state.marking() {
        line = line.child(check_circle(marked));
    }
    line = line.child(
        div()
            .class("fs-contact-face")
            .child(PersonFace::new(title).size(FACE).build()),
    );
    line = line.child(div().class("fs-contact-name").flex_1().child(text(title).no_cursor()));
    for mark in &row.marks {
        line = line.child(div().class("fs-people-mark").child(text(mark)));
    }
    let key = row.key.clone();
    let marking = state.marking();
    let (click, run, right, claim) = (
        Arc::clone(&actions.select),
        Arc::clone(&actions.run),
        Arc::clone(&actions.menu),
        Arc::clone(&actions.claimed),
    );
    let (k_click, k_right) = (key.clone(), key.clone());
    let line = line
        .on_click(move |_| {
            if marking {
                run(&mark_id(&k_click), None);
            } else {
                click(&k_click);
            }
        })
        .on_right_click(move |ctx| {
            // The press travels on to the list around the rows, which must not open its own menu over this one.
            claim.store(true, Ordering::Relaxed);
            right(
                MenuFor::Row(k_right.clone()),
                Trigger::Pointer {
                    at: Point {
                        x: ctx.mouse_x,
                        y: ctx.mouse_y,
                    },
                    shift: ctx.shift,
                },
            );
        });
    let (menu, press, enter) = (
        Arc::clone(&actions.menu),
        Arc::clone(&actions.key),
        Arc::clone(&actions.select),
    );
    let (k_menu, k_press, k_enter) = (key.clone(), key.clone(), key);
    with_keys(
        line,
        KeyActs {
            menu: Arc::new(move |trigger| menu(MenuFor::Row(k_menu.clone()), trigger)),
            shortcut: Arc::new(move |event| press(Some(k_press.as_str()), event)),
            activate: Arc::new(move || enter(&k_enter)),
        },
    )
}

/// The check circle of the mode of choosing.
fn check_circle(on: bool) -> Div {
    let circle = div()
        .class(if on {
            "fs-contact-check fs-contact-check-on"
        } else {
            "fs-contact-check"
        })
        .flex_row()
        .items_center()
        .justify_center();
    if on {
        circle.child(icon(Glyph::Check, 14.0, Tone::Normal))
    } else {
        circle
    }
}

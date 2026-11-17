//! The bar above the list of the people widget: plus, title, choose, menu.
//!
//! Pattern: none (one method that builds the bar from a snapshot). With a program's hook the menu button asks the program to open the
//! menu; without one the workbench hangs its own panel on the button, made of the same lines of the command model.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_commands::Trigger;
use fs_components_widgets::dropdown::dropdown;
use fs_components_widgets::head_bar::Press;
use fs_components_widgets::{Glyph, HeadBar, head_button, head_face};

use super::commands;
use super::state::PeopleState;
use super::view::{MenuFor, PeopleActions};

/// The bar above the list: plus, title, choose, menu.
pub(super) fn list_head(
    state: &PeopleState,
    name: &str,
    actions: &PeopleActions,
    own_menus: bool,
) -> Div {
    let labels = state.labels();
    let mut bar = HeadBar::new().title(labels.title.clone());
    if state.can_create() {
        let run = Arc::clone(&actions.run);
        let press: Press = Arc::new(move || run(commands::NEW, None));
        bar = bar.left(head_button(
            &format!("{name}-create"),
            Glyph::Plus,
            &labels.create,
            false,
            &press,
        ));
    }
    let run = Arc::clone(&actions.run);
    let select_press: Press = Arc::new(move || run(commands::SELECT, None));
    let select = head_button(
        &format!("{name}-select"),
        Glyph::Check,
        &labels.select,
        false,
        &select_press,
    );
    bar = bar.right(if state.marking() {
        select.class("fs-head-button-on")
    } else {
        select
    });
    let items = state.list_menu();
    let id = format!("{name}-menu");
    if own_menus {
        let run = Arc::clone(&actions.run);
        let spoken = labels.list_menu.clone();
        bar = bar.right(div().id(id).w_fit().child(
            dropdown(move |_| head_face(Glyph::ViewList, &spoken, false))
                .width(220.0)
                .align(fs_components_widgets::dropdown::Align::End)
                .commands(
                    &items,
                    &|key| labels.say(key, state),
                    move |command| run(command.as_str(), None),
                )
                .into_element(),
        ));
    } else {
        let menu = Arc::clone(&actions.menu);
        let none: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
        bar = bar.right(
            head_button(&id, Glyph::ViewList, &labels.list_menu, false, &none).on_click(
                move |ctx| {
                    menu(
                        MenuFor::List,
                        Trigger::MenuKey {
                            element: fs_components_commands::Area {
                                x: ctx.bounds_x,
                                y: ctx.bounds_y,
                                width: ctx.bounds_width,
                                height: ctx.bounds_height,
                            },
                        },
                    );
                },
            ),
        );
    }
    bar.build()
}

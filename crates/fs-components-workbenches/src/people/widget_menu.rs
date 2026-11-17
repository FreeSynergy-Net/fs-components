//! Commands, menus and keys of the people widget.
//!
//! Pattern: Command (a menu line, a shortcut and a card press all end in [`PeopleWidget::run`]) + Mediator (the widget decides what a
//! press on a row means for the menu: the model's rule *target = selection*). Nothing here holds the lock while a hook of the
//! program runs, so a hook may call back into the widget.

use fs_components_commands::{FocusContext, Outcome, Platform, Target, Trigger, target_for};
use fs_components_widgets::keys::KeyEvent;

use super::commands::{KIND, Ran};
use super::keypress::press_from;
use super::view::MenuFor;
use super::widget::{PeopleWidget, lock_slot, locked, rebuild};

impl PeopleWidget {
    /// Run a command by id (a chosen menu line or a shortcut); `value` is what a card hands. The window draws again.
    pub fn run(&self, command: &str, value: Option<&str>) {
        let ran = locked(&self.state).run_command(command, value);
        match ran {
            Ran::Copy(text) => {
                let done = blinc_layout::widgets::text_edit::clipboard_write(&text);
                if done {
                    locked(&self.state).copied(&text);
                }
            }
            Ran::Hand(request) => {
                let hook = lock_slot(&self.request).clone();
                if let Some(hook) = hook {
                    hook(&request);
                }
            }
            _ => {}
        }
        rebuild();
    }

    /// A menu was asked for: choose what it acts on and the lines, then hand it to the program. A press on a row first makes that
    /// row the selection (the model's rule: a row in the selection acts on all of it, a row outside becomes the selection).
    pub fn open_menu(&self, what: &MenuFor, trigger: Trigger) {
        let (target, items) = {
            let mut state = locked(&self.state);
            let target = match what {
                MenuFor::Row(key) => {
                    let target = target_for(KIND, state.marked(), Some(key.as_str()));
                    if target.ids.len() == 1 && state.selected() != Some(key.as_str()) {
                        state.select(key);
                    }
                    target
                }
                MenuFor::Person => Target {
                    kind: KIND.to_owned(),
                    ids: state.selection(),
                },
                MenuFor::Empty | MenuFor::List => target_for(KIND, &[], None),
            };
            let items = match what {
                MenuFor::Row(_) | MenuFor::Person => state.person_menu(),
                MenuFor::Empty => state.empty_menu(),
                MenuFor::List => state.list_menu(),
            };
            (target, items)
        };
        let hook = lock_slot(&self.context).clone();
        match hook {
            Some(hook) if !items.is_empty() => hook(&trigger, &target, &items),
            _ => {}
        }
        rebuild();
    }

    /// A key the window got that the list did not use itself: is it a shortcut of the keymap? `row` is the row that had the keyboard, if
    /// any; it becomes the person shown first, so a shortcut acts on what the person sees.
    pub fn key(&self, row: Option<&str>, event: &KeyEvent) {
        let Some(press) = press_from(event) else {
            return;
        };
        let outcome = {
            let mut state = locked(&self.state);
            if let Some(row) = row
                && state.selected() != Some(row)
            {
                state.select(row);
            }
            state
                .keymap(Platform::current())
                .dispatch(&press, FocusContext::List)
        };
        if let Outcome::Run(id) = outcome {
            self.run(id.as_str(), None);
        }
    }
}

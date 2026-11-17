//! Shortcuts of a window: the keymap, the focus context and the global key hook that runs a command.
//!
//! Pattern: Adapter + Facade over the dispatcher of `fs-components-commands`. The window's root element listens to every key press
//! (Blinc broadcasts `KEY_DOWN` to all handlers), turns it into a [`KeyPress`], asks the [`Keymap`] and acts on the [`Outcome`]: a command
//! runs through the **one** handler of `set_menus` (the same as a click on its line: one handler, never fired twice for one
//! gesture); a locked command shows its reason, a conflict shows both commands, both in the status line (never silent). The context
//! is "text field" while a Blinc text field has the keyboard, else the one a list or tree set with [`AppWidget::set_focus_context`].

use std::sync::Arc;

use blinc_layout::widgets::text_input::has_focused_text_input;
use fs_components_commands::{CommandId, FocusContext, KeyPress, Keymap, Outcome, TextKey};
use fs_components_words::{Args, Locale, TextId, Words};

use crate::key_events::press_from_code;
use crate::menu_overlay;
use crate::shell::Status;
use crate::texts::shell_texts;
use crate::widget::{AppWidget, locked};

/// What the shell needs besides the keymap to answer a press.
pub(crate) struct Shortcuts {
    pub(crate) keymap: Keymap,
    /// The text of a key of the program (the reason of a locked command).
    say: Arc<dyn Fn(&TextKey) -> String + Send + Sync>,
    locale: Locale,
}

impl AppWidget {
    /// Install the shortcuts of the window: `keymap` (built from the same resolved items as the menu, `Keymap::from_resolved`),
    /// `say` for the reasons of locked commands (the program's MF2 texts) and the language of the shell's own messages. A command
    /// that a shortcut runs goes to the handler of [`AppWidget::set_menus`] with its id, like a chosen menu line; set the menus
    /// first. Call again when states change.
    pub fn set_shortcuts(
        &self,
        keymap: Keymap,
        say: impl Fn(&TextKey) -> String + Send + Sync + 'static,
        locale: Locale,
    ) {
        locked(&self.shell).shortcuts = Some(Shortcuts {
            keymap,
            say: Arc::new(say),
            locale,
        });
    }

    /// Where the keyboard is, for the shortcuts that count only there (a list sets `List` when it gets the keyboard and
    /// `Global` when it loses it). A text field needs no call: it is found.
    pub fn set_focus_context(&self, context: FocusContext) {
        locked(&self.shell).focus = context;
    }

    /// Answer a key press as the global hook does and act on it; returns what it led to. `in_text` says that a text field has the
    /// keyboard. Runs the command with no lock held.
    pub fn press(&self, press: &KeyPress, in_text: bool) -> Outcome {
        let (outcome, handler, say, locale) = {
            let shell = locked(&self.shell);
            let Some(shortcuts) = &shell.shortcuts else {
                return Outcome::NoMatch;
            };
            let context = if in_text {
                FocusContext::TextField
            } else {
                shell.focus
            };
            (
                shortcuts.keymap.dispatch(press, context),
                shell.handler(),
                Arc::clone(&shortcuts.say),
                shortcuts.locale.clone(),
            )
        };
        match &outcome {
            Outcome::Run(id) => {
                if self.builtin(id.as_str()) {
                    menu_overlay::close(&self.shell);
                } else if let Some(handler) = handler {
                    // First close an open menu, as a chosen line does: what the handler opens must not go down with it.
                    menu_overlay::close(&self.shell);
                    handler(id.as_str());
                }
            }
            Outcome::Disabled { reason, .. } => {
                self.set_status(Some(Status::Info(say(reason))));
            }
            Outcome::Conflict(ids) => {
                self.set_status(Some(Status::Problem(conflict_text(press, ids, &locale))));
            }
            _ => {}
        }
        outcome
    }

    /// A key went down: `true` if it is key repeat, that is the same key again with no release in between. Blinc's
    /// event carries no repeat flag (pin `fs-blinc-3.1`: `KeyboardEvent` has none), so the shell remembers the held key
    /// until its `KEY_UP`, or until another key goes down. A command runs once per press, not once per repeat.
    fn key_went_down(&self, code: u32) -> bool {
        let mut shell = locked(&self.shell);
        let repeat = shell.held == Some(code);
        shell.held = Some(code);
        repeat
    }

    /// A key came up: it can go down again.
    fn key_went_up(&self, code: u32) {
        let mut shell = locked(&self.shell);
        if shell.held == Some(code) {
            shell.held = None;
        }
    }

    /// The handlers of the window's root element: every key press goes through [`AppWidget::press`] (no repeat), every
    /// release clears the held key.
    pub(crate) fn key_hooks(&self) -> (Hook, Hook) {
        let (down, up) = (self.clone(), self.clone());
        (
            Box::new(move |ctx| {
                if down.key_went_down(ctx.key_code) {
                    return;
                }
                let mods = (ctx.shift, ctx.ctrl, ctx.alt, ctx.meta);
                if let Some(press) = press_from_code(ctx.key_code, mods) {
                    down.press(&press, has_focused_text_input());
                }
            }),
            Box::new(move |ctx| up.key_went_up(ctx.key_code)),
        )
    }
}

/// A handler of a key event of the root element.
pub(crate) type Hook = Box<dyn Fn(&blinc_layout::event_handler::EventContext) + 'static>;

fn conflict_text(press: &KeyPress, ids: &[CommandId], locale: &Locale) -> String {
    let names: Vec<&str> = ids.iter().map(CommandId::as_str).collect();
    shell_texts(locale)
        .say(
            &TextId::new("shell-shortcut-conflict"),
            &Args::new()
                .text("keys", press.shortcut().aria())
                .text("commands", names.join(", ")),
        )
        .text()
        .to_owned()
}

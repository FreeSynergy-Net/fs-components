//! The central dispatcher: a key press becomes the command to run, or the reason it is not run.
//!
//! Pattern: Command dispatcher (Chain of Responsibility in one function: composition, then the text field, then the table).
//! Pure logic without Blinc; the Blinc adapter (`fs-components-app`) only turns an event into a [`KeyPress`] and acts on the
//! [`Outcome`]. The table is built from the same resolved items the menu is built from ([`Keymap::from_resolved`]), so a shortcut that
//! a menu line shows is the one that runs it, and a command that is locked never runs.

use crate::command::{CommandId, State, TextKey};
use crate::commands::Commands;
use crate::display::Platform;
use crate::keypress::{FocusContext, KeyPress};
use crate::resolve::Resolved;
use crate::shortcut::Shortcut;
use crate::text_keys::owned_by_text;

/// What a key press leads to.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Run this command.
    Run(CommandId),
    /// The command has this shortcut but is locked now: nothing runs, the reason is shown (never silent).
    Disabled {
        /// The locked command.
        command: CommandId,
        /// The key of the text that says why.
        reason: TextKey,
    },
    /// No command has this shortcut here, or the press belongs to the text field or an input method: the window does nothing.
    NoMatch,
    /// Several commands have this shortcut in this context: none runs, the conflict is shown.
    Conflict(Vec<CommandId>),
}

#[derive(Clone, Debug)]
struct Binding {
    id: CommandId,
    shortcut: Shortcut,
    state: State,
    scope: Option<FocusContext>,
}

/// The table of shortcuts of a window: which command each key combination runs, in which focus context.
#[derive(Clone, Debug)]
pub struct Keymap {
    platform: Platform,
    bindings: Vec<Binding>,
}

impl Keymap {
    /// The shortcuts of all `commands`, each valid in every focus context unless [`Keymap::only_in`] narrows it.
    #[must_use]
    pub fn from_commands(commands: &Commands, platform: Platform) -> Self {
        Self::collect(commands.iter(), platform)
    }

    /// The shortcuts of the commands in these resolved items (the arrangement of the active place, as `Menu::from_model` shows
    /// it): a command that is not offered there has no shortcut there. A command that stands in two places counts once.
    #[must_use]
    pub fn from_resolved(items: &[Resolved], platform: Platform) -> Self {
        let mut commands = Vec::new();
        collect_commands(items, &mut commands);
        Self::collect(commands.into_iter(), platform)
    }

    fn collect<'a>(commands: impl Iterator<Item = &'a crate::Command>, platform: Platform) -> Self {
        let mut seen: Vec<&CommandId> = Vec::new();
        let mut bindings = Vec::new();
        for command in commands {
            if seen.contains(&&command.id) {
                continue;
            }
            seen.push(&command.id);
            for shortcut in &command.shortcuts {
                bindings.push(Binding {
                    id: command.id.clone(),
                    shortcut: shortcut.clone(),
                    state: command.state.clone(),
                    scope: None,
                });
            }
        }
        Self { platform, bindings }
    }

    /// The command `id` counts only while the keyboard is in `context` (Delete for the rows of a list, F2 for a tree).
    /// Two commands with one key may share it when their contexts differ.
    #[must_use]
    pub fn only_in(mut self, id: &CommandId, context: FocusContext) -> Self {
        for binding in self.bindings.iter_mut().filter(|b| &b.id == id) {
            binding.scope = Some(context);
        }
        self
    }

    /// The platform the keymap resolves `Primary` for.
    #[must_use]
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// What `press` leads to with the keyboard in `context`.
    ///
    /// Order: an input method that composes owns every key; in a text field the text keeps its own keys ([`FocusContext::TextField`]);
    /// then the commands whose shortcut is this key and whose context fits are looked up: none is `NoMatch`, more than one is a
    /// `Conflict` (nothing runs, it is never decided silently), one that is locked is `Disabled`, else `Run`.
    #[must_use]
    pub fn dispatch(&self, press: &KeyPress, context: FocusContext) -> Outcome {
        if press.is_composing()
            || (context == FocusContext::TextField && owned_by_text(press, self.platform))
        {
            return Outcome::NoMatch;
        }
        let mut hits: Vec<&Binding> = Vec::new();
        for binding in &self.bindings {
            let fits = binding.scope.is_none_or(|scope| scope == context);
            if fits
                && binding.shortcut.same_as(press.shortcut(), self.platform)
                && !hits.iter().any(|h| h.id == binding.id)
            {
                hits.push(binding);
            }
        }
        match hits.as_slice() {
            [] => Outcome::NoMatch,
            [one] => match one.state.reason() {
                Some(reason) => Outcome::Disabled {
                    command: one.id.clone(),
                    reason: reason.clone(),
                },
                None => Outcome::Run(one.id.clone()),
            },
            many => Outcome::Conflict(many.iter().map(|b| b.id.clone()).collect()),
        }
    }
}

fn collect_commands<'a>(items: &'a [Resolved], out: &mut Vec<&'a crate::Command>) {
    for item in items {
        match item {
            Resolved::Command(command) => out.push(command),
            Resolved::Submenu { children, .. } => collect_commands(children, out),
            Resolved::IconRow(row) => out.extend(row.iter()),
            _ => {}
        }
    }
}

//! The commands of the people workbench: what a menu line, a shortcut and a workflow block can name.
//!
//! Pattern: Command (ids as plain text of the command model, `fs-components-commands`) + Strategy (the rules of which line is
//! offered, locked, or not there are pure functions of the state). The workbench runs what it can by itself (new, edit, delete,
//! choose, group, copy) and hands the rest to the program as a [`PeopleRequest`] (write a mail, export, merge), because the
//! program owns mail clients, files and the merging of people. The model is checked against the capabilities of the source: a
//! line the source cannot do is not there (ADR 0037); a line that cannot be run just now is there and says why (never silently
//! greyed out). Shortcuts are the model's: [`PeopleState::keymap`] is what the window asks, [`press_from`] turns a key of the
//! window into the model's key press.

use fs_components_commands::{
    Command, CommandId, FocusContext, Keymap, Platform, Resolved, Shortcut, TextKey,
};

use super::labels::PeopleLabels;
use super::state::PeopleState;

/// The kind of object the context menu is for: the arrangement is `context:person`.
pub const KIND: &str = "person";
/// Make a new person.
pub const NEW: &str = "people.new";
/// Edit the person shown.
pub const EDIT: &str = "people.edit";
/// Delete the person shown (asks first).
pub const DELETE: &str = "people.delete";
/// Hand the person or the chosen people out (the program does it).
pub const EXPORT: &str = "people.export";
/// Merge the chosen people (the program does it).
pub const MERGE: &str = "people.merge";
/// Turn the choosing of several people on or off.
pub const SELECT: &str = "people.select";
/// Copy a value (the text of a card).
pub const COPY: &str = "people.copy";
/// Write a mail to an address (the program opens it).
pub const MAIL: &str = "people.mail";
/// Prefix of "narrow the list to this group": `people.group:<key>`; the key empty = everyone.
pub const GROUP_PREFIX: &str = "people.group:";

/// Prefix of "mark or unmark this person" in the mode of choosing: `people.mark:<key>`.
pub const MARK_PREFIX: &str = "people.mark:";

/// The id that marks `key`.
pub fn mark_id(key: &str) -> String {
    format!("{MARK_PREFIX}{key}")
}

/// The id that narrows the list to `group`.
pub fn group_id(group: Option<&str>) -> String {
    format!("{GROUP_PREFIX}{}", group.unwrap_or_default())
}

/// The key of the text of a group's line.
fn group_key(group: Option<&str>) -> String {
    format!("people-group:{}", group.unwrap_or_default())
}

/// What the program has to do: the workbench cannot, it does not know mail clients, files or merging.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleRequest {
    /// The command id (`people.mail`, `people.export`, `people.merge`).
    pub id: String,
    /// The people it acts on, by key.
    pub keys: Vec<String>,
    /// The value it acts on (the address of a mail).
    pub value: Option<String>,
}

/// What came of running a command.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ran {
    /// The workbench did it (the window draws again).
    Done,
    /// This text goes to the clipboard; the workbench says so to the person.
    Copy(String),
    /// The program does it.
    Hand(PeopleRequest),
    /// Nothing happened: the id is not one of ours, or the line was locked.
    Nothing,
}

fn id(name: &str) -> CommandId {
    CommandId::new(name)
}

fn key(name: &str) -> TextKey {
    TextKey::new(name)
}

/// `command` with the shortcut `text`. The texts are literals of this file; a wrong one leaves the command without a shortcut, and
/// the test of the keymap fails.
fn with_shortcut(command: Command, text: &str) -> Command {
    match Shortcut::parse(text) {
        Ok(shortcut) => command.shortcut(shortcut),
        Err(_) => command,
    }
}

impl PeopleState {
    /// The command model of the workbench as it is now: only what the source can do, locked with a reason where it cannot be run
    /// now. The one list the menus and the keymap are made of.
    pub fn commands(&self) -> Vec<Command> {
        let mut all = Vec::new();
        if self.can_create() {
            all.push(with_shortcut(
                Command::new(id(NEW), key("people-new")).icon("list-add"),
                "Primary+N",
            ));
        }
        if self.can_edit() {
            let edit = with_shortcut(
                Command::new(id(EDIT), key("people-edit")).icon("document-edit"),
                "Primary+E",
            );
            all.push(if self.selected().is_some() && self.editing().is_none() {
                edit
            } else {
                edit.disabled("people-nothing-to-edit")
            });
        }
        if self.can_delete() {
            all.push(with_shortcut(
                Command::new(id(DELETE), key("people-delete")).icon("edit-delete"),
                "Delete",
            ));
        }
        if self.capabilities.export {
            all.push(with_shortcut(
                Command::new(id(EXPORT), key("people-export")).icon("document-save-as"),
                "Primary+Shift+E",
            ));
        }
        if self.capabilities.merge {
            let merge = with_shortcut(
                Command::new(id(MERGE), key("people-merge")).icon("merge"),
                "Primary+Shift+M",
            );
            all.push(if self.marked().len() >= 2 {
                merge
            } else {
                merge.disabled("people-merge-needs-two")
            });
        }
        all.push(
            Command::new(id(SELECT), key("people-select"))
                .icon("object-select")
                .check(self.marking()),
        );
        all
    }

    fn line(&self, name: &str) -> Option<Resolved> {
        self.commands()
            .into_iter()
            .find(|c| c.id.as_str() == name)
            .map(Resolved::Command)
    }

    /// The lines of the menu of people (the three dots, and the context menu on a person): Edit, Export, Merge, then Delete.
    /// Several marked people: only what makes sense for many (Export, Merge, Delete).
    pub fn person_menu(&self) -> Vec<Resolved> {
        let many = self.marking() && !self.marked().is_empty();
        let mut items = Vec::new();
        if !many && let Some(edit) = self.line(EDIT) {
            items.push(edit);
        }
        let mut middle: Vec<Resolved> = [EXPORT, MERGE]
            .iter()
            .filter_map(|n| self.line(n))
            .collect();
        if !middle.is_empty() {
            if !items.is_empty() {
                items.push(Resolved::Separator);
            }
            items.append(&mut middle);
        }
        if let Some(delete) = self.line(DELETE) {
            if !items.is_empty() {
                items.push(Resolved::Separator);
            }
            items.push(delete);
        }
        items
    }

    /// The lines of the menu on empty space of the list: New, Select.
    pub fn empty_menu(&self) -> Vec<Resolved> {
        [NEW, SELECT].iter().filter_map(|n| self.line(n)).collect()
    }

    /// The lines of the menu of the list (the three lines in its bar): Select, then the groups as a choice of one (everyone first).
    pub fn list_menu(&self) -> Vec<Resolved> {
        let mut items: Vec<Resolved> = self.line(SELECT).into_iter().collect();
        if self.groups().is_empty() {
            return items;
        }
        items.push(Resolved::Separator);
        let choice = |group: Option<&str>| {
            Resolved::Command(
                Command::new(id(&group_id(group)), key(&group_key(group)))
                    .radio("people-group", self.group() == group),
            )
        };
        items.push(choice(None));
        for g in self.groups() {
            items.push(choice(Some(&g.key)));
        }
        items
    }

    /// The shortcuts of the workbench. Delete counts only while a list has the keyboard, so a text field keeps its own.
    pub fn keymap(&self, platform: Platform) -> Keymap {
        let mut all = fs_components_commands::Commands::new();
        for command in self.commands() {
            all = all.with(command);
        }
        Keymap::from_commands(&all, platform).only_in(&id(DELETE), FocusContext::List)
    }

    /// Run a command: `value` is what a card hands (the text to copy, the address to write to). Choosing and groups work on the
    /// state; editing and deleting start the usual way (delete asks first).
    pub fn run_command(&mut self, command: &str, value: Option<&str>) -> Ran {
        if let Some(group) = command.strip_prefix(GROUP_PREFIX) {
            self.set_group((!group.is_empty()).then_some(group));
            return Ran::Done;
        }
        if let Some(key) = command.strip_prefix(MARK_PREFIX) {
            self.toggle_mark(key);
            return Ran::Done;
        }
        // A line that is not offered (the source cannot do it, or it makes no sense now: a second Edit while editing) or is locked does
        // not run.
        if matches!(command, NEW | EDIT | DELETE | EXPORT | MERGE | SELECT) {
            let offered = self.commands();
            let runs = offered
                .iter()
                .find(|c| c.id.as_str() == command)
                .is_some_and(|c| !c.state.is_locked());
            if !runs {
                return Ran::Nothing;
            }
        }
        match command {
            NEW => {
                self.begin_create();
                Ran::Done
            }
            EDIT => {
                self.begin_edit();
                Ran::Done
            }
            DELETE => {
                self.ask_delete();
                Ran::Done
            }
            SELECT => {
                self.toggle_marking();
                Ran::Done
            }
            COPY => value.map_or(Ran::Nothing, |v| Ran::Copy(v.to_owned())),
            MAIL => value.map_or(Ran::Nothing, |v| self.hand(command, Some(v))),
            EXPORT | MERGE => self.hand(command, None),
            _ => Ran::Nothing,
        }
    }

    fn hand(&self, command: &str, value: Option<&str>) -> Ran {
        Ran::Hand(PeopleRequest {
            id: command.to_owned(),
            keys: self.selection(),
            value: value.map(str::to_owned),
        })
    }

    /// Tell the person a value was copied.
    pub fn copied(&mut self, value: &str) {
        self.notice = Some(PeopleLabels::fill(&self.labels.copied, value));
    }
}

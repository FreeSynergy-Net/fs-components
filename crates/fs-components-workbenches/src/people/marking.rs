//! Choosing several people and narrowing the list to a group.
//!
//! Pattern: State — more transitions of [`PeopleState`]. Marking is a mode of the list (the check mark in its bar): while it is on,
//! a press on a row marks or unmarks it and the person shown does not change. Leaving the mode forgets the marks. A group is a
//! filter the source applies ([`PeopleSource::rows_in`](super::PeopleSource::rows_in)); the list starts over when it changes.

use super::model::GroupFilter;
use super::state::{PAGE, PeopleState};

impl PeopleState {
    /// The groups the list can be narrowed to.
    pub fn groups(&self) -> &[GroupFilter] {
        &self.groups
    }

    /// The group the list is narrowed to, if any.
    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    /// The title of the group the list is narrowed to.
    pub fn group_title(&self) -> Option<&str> {
        let key = self.group.as_deref()?;
        self.groups
            .iter()
            .find(|g| g.key == key)
            .map(|g| g.title.as_str())
    }

    /// The title of the list: the group, or "All contacts".
    pub fn list_title(&self) -> String {
        self.group_title()
            .map_or_else(|| self.labels.all_contacts.clone(), str::to_owned)
    }

    /// Narrow the list to `group` (`None`: everyone). The list starts over; the person shown stays.
    pub fn set_group(&mut self, group: Option<&str>) {
        if self.group.as_deref() == group {
            return;
        }
        self.group = group.map(str::to_owned);
        self.list_generation_up();
        self.reload(PAGE);
        let rows = std::sync::Arc::clone(self.rows());
        self.marked.retain(|k| rows.iter().any(|r| &r.key == k));
    }

    /// Is the list in the mode of choosing several people?
    pub fn marking(&self) -> bool {
        self.marking
    }

    /// Turn the mode of choosing several people on or off; off forgets the marks.
    pub fn toggle_marking(&mut self) {
        self.marking = !self.marking;
        self.marked.clear();
    }

    /// Mark `key`, or unmark it if it is marked. Only in the mode of choosing; otherwise nothing happens.
    pub fn toggle_mark(&mut self, key: &str) {
        if !self.marking {
            return;
        }
        match self.marked.iter().position(|k| k == key) {
            Some(at) => {
                self.marked.remove(at);
            }
            None => self.marked.push(key.to_owned()),
        }
    }

    /// The marked people, in the order they were marked.
    pub fn marked(&self) -> &[String] {
        &self.marked
    }

    /// Is `key` marked?
    pub fn is_marked(&self, key: &str) -> bool {
        self.marked.iter().any(|k| k == key)
    }

    /// The people a command acts on: the marked ones while there are any, else the person shown.
    pub fn selection(&self) -> Vec<String> {
        if self.marking && !self.marked.is_empty() {
            return self.marked.clone();
        }
        self.selected().map(str::to_owned).into_iter().collect()
    }
}

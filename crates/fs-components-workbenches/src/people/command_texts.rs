//! The words of the lines of the people command model.
//!
//! Pattern: none — one lookup from the key of a text (`people-edit`, ...) to the word in [`PeopleLabels`]; the caller's words, never ours
//! (ADR 0053). The line of a group has the key `people-group:<key>` and says the title the source gave.

use fs_components_commands::TextKey;

use super::labels::PeopleLabels;
use super::state::PeopleState;

impl PeopleLabels {
    /// The words of a key of the command model of this workbench; an unknown key is shown as it is.
    pub fn say(&self, key: &TextKey, state: &PeopleState) -> String {
        let name = key.as_str();
        if let Some(group) = name.strip_prefix("people-group:") {
            if group.is_empty() {
                return self.all_contacts.clone();
            }
            return state
                .groups()
                .iter()
                .find(|g| g.key == group)
                .map_or_else(|| group.to_owned(), |g| g.title.clone());
        }
        match name {
            "people-new" => self.create.clone(),
            "people-edit" => self.edit.clone(),
            "people-delete" => self.delete.clone(),
            "people-export" => self.export.clone(),
            "people-merge" => self.merge.clone(),
            "people-select" => self.select.clone(),
            "people-groups" => self.groups.clone(),
            "people-nothing-to-edit" => self.nothing_selected.clone(),
            "people-merge-needs-two" => self.merge_needs_two.clone(),
            other => other.to_owned(),
        }
    }
}


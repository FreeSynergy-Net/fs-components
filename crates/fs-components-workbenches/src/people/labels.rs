//! The words the people workbench shows.
//!
//! Pattern: none — the workbench does not know its audience, so every text comes from the
//! caller (translation, ADR 0053, is the caller's). `{}` is replaced by what the text talks about.
//! The defaults are neutral English.

/// Texts of the people workbench.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleLabels {
    /// Placeholder of the search field.
    pub search: String,
    /// The headings of the list; each row has one text per column.
    pub columns: Vec<String>,
    /// Button: load more rows.
    pub more: String,
    /// Button: a new person.
    pub create: String,
    /// Button: edit.
    pub edit: String,
    /// Button: save.
    pub save: String,
    /// Button: cancel the edit.
    pub cancel: String,
    /// Button: delete.
    pub delete: String,
    /// Button: the second press that really deletes.
    pub delete_really: String,
    /// Button: keep the person after all.
    pub delete_keep: String,
    /// Button: add a field to a group; `{}` = the group.
    pub add_field: String,
    /// Shown when nobody is selected.
    pub nothing_selected: String,
    /// The list is empty.
    pub nobody: String,
    /// The search found nobody; `{}` = the search text.
    pub nobody_found: String,
    /// The first press of Delete; `{}` = the person.
    pub delete_confirm: String,
    /// The person shown is no longer there.
    pub gone: String,
    /// Going elsewhere while there are unsaved changes.
    pub unsaved: String,
    /// Cancel with unsaved changes.
    pub discard_confirm: String,
    /// Button: go on editing after being asked about discarding.
    pub discard_keep: String,
    /// Button: the second action that really discards.
    pub discard_really: String,
    /// After a save.
    pub saved: String,
    /// After making a new person.
    pub created: String,
    /// After a delete; `{}` = the person.
    pub deleted: String,
    /// Save without changes.
    pub no_changes: String,
    /// `{}` = the reason.
    pub not_saved: String,
    /// Shown instead of saving while a field holds something invalid.
    pub fix_fields: String,
    /// Shown in an empty e-mail field.
    pub email_placeholder: String,
    /// Shown in an empty phone field.
    pub phone_placeholder: String,
    /// `{}` = the reason.
    pub not_deleted: String,
    /// `{}` = the reason.
    pub cannot_edit: String,
    /// `{}` = the reason.
    pub unreadable: String,
    /// `{}` = the reason.
    pub list_failed: String,
    /// Title of the bar above the list.
    pub title: String,
    /// Title of the list when it shows everyone.
    pub all_contacts: String,
    /// Button: choose several people (the check mark in the bar of the list).
    pub select: String,
    /// Button: the menu of the list (the three lines in the bar of the list).
    pub list_menu: String,
    /// Button: the menu of the person (the three dots in the bar of the person).
    pub person_menu: String,
    /// Line: hand the person out.
    pub export: String,
    /// Line: merge the chosen people into one.
    pub merge: String,
    /// Heading of the groups in the menu of the list.
    pub groups: String,
    /// Action on a card: write a mail.
    pub mail_write: String,
    /// Action on a card: copy the value.
    pub copy: String,
    /// After copying; `{}` = what was copied.
    pub copied: String,
    /// Merging needs at least two chosen people.
    pub merge_needs_two: String,
    /// Shown for a person with no details.
    pub no_details: String,
    /// The list is narrowed to a group; `{}` = the group.
    pub in_group: String,
    /// After a conflict; `{}` = what happened. Pressing Save again writes over it knowingly.
    pub conflict_confirm: String,
}

impl Default for PeopleLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            search: s("Search"),
            columns: vec![s("Name")],
            more: s("Show more"),
            create: s("New"),
            edit: s("Edit"),
            save: s("Save"),
            cancel: s("Cancel"),
            delete: s("Delete"),
            delete_really: s("Really delete"),
            delete_keep: s("Keep"),
            add_field: s("Add: {}"),
            nothing_selected: s("Nobody selected."),
            nobody: s("Nobody here yet."),
            nobody_found: s("Nobody found for \"{}\"."),
            delete_confirm: s("Delete {}? This cannot be taken back here."),
            gone: s("This person no longer exists."),
            unsaved: s("Save or cancel your changes first."),
            discard_confirm: s("You have unsaved changes. Keep editing, or discard them."),
            discard_keep: s("Keep editing"),
            discard_really: s("Discard changes"),
            saved: s("Saved."),
            created: s("Created."),
            deleted: s("{} was deleted."),
            no_changes: s("Nothing was changed."),
            not_saved: s("Not saved: {}"),
            fix_fields: s("Some fields are not valid. Correct them first."),
            email_placeholder: s("name@example.org"),
            phone_placeholder: s("+49 30 1234567"),
            not_deleted: s("Not deleted: {}"),
            cannot_edit: s("Cannot edit this: {}"),
            unreadable: s("This could not be read: {}"),
            list_failed: s("The list failed: {}"),
            title: s("Contacts"),
            all_contacts: s("All contacts"),
            select: s("Select"),
            list_menu: s("Menu"),
            person_menu: s("More"),
            export: s("Export"),
            merge: s("Merge"),
            groups: s("Groups"),
            mail_write: s("Write mail"),
            copy: s("Copy"),
            copied: s("Copied: {}"),
            merge_needs_two: s("Choose at least two people to merge."),
            no_details: s("No details yet."),
            in_group: s("Group: {}"),
            conflict_confirm: s("{} Your text is kept. Press Save again to write it anyway."),
        }
    }
}

impl PeopleLabels {
    /// `template` with its `{}` replaced by `value`.
    pub fn fill(template: &str, value: &str) -> String {
        template.replacen("{}", value, 1)
    }
}

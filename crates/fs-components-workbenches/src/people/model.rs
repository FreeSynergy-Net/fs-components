//! What the people workbench and its source talk about.
//!
//! Pattern: none — neutral rows and a draft of labelled fields. The sheet that is read (sections of labelled fields) is the page of
//! one record of `fs-components-widgets::detail_page`, the same for every program. Nothing here knows what a person is made of.

pub use super::capabilities::PeopleCapabilities;
pub use super::kinds::GroupFilter;
pub use fs_components_widgets::detail_page::FieldKind;

/// One row of the list.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonRow {
    /// Names the person for the source; opaque to the workbench.
    pub key: String,
    /// One text per column of [`PeopleLabels::columns`](super::PeopleLabels::columns).
    pub cells: Vec<String>,
    /// Short markings that must be seen in the list, in the source's words.
    pub marks: Vec<String>,
    /// Shown only when the person chooses this row, never opened by the workbench on its own.
    pub only_when_chosen: bool,
}

impl PersonRow {
    /// A row.
    pub fn new(key: impl Into<String>, cells: Vec<String>) -> Self {
        Self {
            key: key.into(),
            cells,
            marks: Vec::new(),
            only_when_chosen: false,
        }
    }

    /// The row of someone to be marked in the list and opened only when chosen (ADR 0029).
    #[must_use]
    pub fn guarded(mut self, mark: impl Into<String>) -> Self {
        self.marks.push(mark.into());
        self.only_when_chosen = true;
        self
    }
}

pub use fs_components_widgets::detail_page::{
    Field as PersonField, Section as PersonSection, Sheet as PersonSheet,
};

/// What a field holds, so the form can check it as the person types and a shell can choose the
/// keyboard and the shape of the field. The check itself is the source's (it owns the rule).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldCheck {
    /// Any text.
    #[default]
    Text,
    /// An e-mail address.
    Email,
    /// An international phone number.
    Phone,
}

/// One field of a draft.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftField {
    /// Names the field for the source; unique within the draft.
    pub key: String,
    /// What it asks for.
    pub label: String,
    /// The text as it is now.
    pub value: String,
    /// The text when the draft was made; the source writes only what differs from it, so whatever
    /// a form does not show stays as it is.
    pub original: String,
    /// Added in this edit: it has no counterpart in the source yet.
    pub added: bool,
    /// What it holds; the source is asked whether the text is valid.
    pub check: FieldCheck,
}

impl DraftField {
    /// A field holding `value`.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            key: key.into(),
            label: label.into(),
            original: value.clone(),
            value,
            added: false,
            check: FieldCheck::Text,
        }
    }

    /// This field, holding `check`.
    #[must_use]
    pub fn with_check(mut self, check: FieldCheck) -> Self {
        self.check = check;
        self
    }

    /// Has the person changed it?
    pub fn changed(&self) -> bool {
        self.value != self.original
    }
}

/// Fields that belong together; a group that may grow offers an Add action.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftGroup {
    /// Names the group for the source.
    pub key: String,
    /// The heading, and the label of a field added to it.
    pub label: String,
    /// May the person add fields?
    pub can_add: bool,
    /// The fields.
    pub fields: Vec<DraftField>,
    /// What a field added to this group holds.
    pub check: FieldCheck,
}

impl DraftGroup {
    /// A group.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        can_add: bool,
        fields: Vec<DraftField>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            can_add,
            fields,
            check: FieldCheck::Text,
        }
    }

    /// This group, whose fields hold `check` (also those added later).
    #[must_use]
    pub fn with_check(mut self, check: FieldCheck) -> Self {
        for field in &mut self.fields {
            field.check = check;
        }
        self.check = check;
        self
    }
}

/// A person as a form. A field emptied by the person means: remove this value.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonDraft {
    /// Opaque marker of the state the draft was made from; a save from an old one is a conflict.
    pub revision: Option<String>,
    /// Short markings that must stay in view while editing, in the source's words.
    pub marks: Vec<String>,
    /// The fields, by group.
    pub groups: Vec<DraftGroup>,
}

impl PersonDraft {
    /// A draft.
    pub fn new(revision: Option<String>, groups: Vec<DraftGroup>) -> Self {
        Self {
            revision,
            marks: Vec::new(),
            groups,
        }
    }

    /// Every field, in order.
    pub fn fields(&self) -> impl Iterator<Item = &DraftField> {
        self.groups.iter().flat_map(|g| g.fields.iter())
    }

    /// Has the person changed anything?
    pub fn changed(&self) -> bool {
        self.fields().any(DraftField::changed)
    }
}

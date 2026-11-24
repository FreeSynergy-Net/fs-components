//! What a board or list is bound to: the type, the grouping, and the fields that stand in the slots of the card, the frame and the
//! columns of the list. The names come from the description of the page (data), never from here.
//!
//! Pattern: Value objects (a Builder-style chain). No program name, no field name, no block name is fixed in this crate.

use fs_components_kanban::board::{DEFAULT_ELEMENTS, DEFAULT_FACTS, Limit};

/// The role a field plays in a place (the four roles of the field components).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// Short, on a card.
    Compact,
    /// Read in the frame at the side.
    Display,
    /// Edited (not used while reading only).
    Input,
    /// A cell of a table.
    Cell,
}

/// One slot: a field, the name the reader sees, the role and the building block the catalog chose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The top-level field.
    pub field: String,
    /// The name the reader sees (already in her language).
    pub label: String,
    /// The role.
    pub role: Role,
    /// The building block `BlockCatalog::choose` named for this field and role (`field-email`); from the plan, never chosen here.
    pub block: String,
}

impl Slot {
    /// A slot.
    pub fn new(
        field: impl Into<String>,
        label: impl Into<String>,
        role: Role,
        block: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
            role,
            block: block.into(),
        }
    }
}

/// The binding of one board or list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordsBinding {
    /// The data type.
    pub data_type: String,
    /// The container record, if the view is of one.
    pub scope: Option<String>,
    /// The grouping that makes the columns.
    pub grouping: String,
    /// The named order (`rank`); `None` = the provider's default.
    pub order: Option<String>,
    /// The field of the card title.
    pub title: String,
    /// The field of the second line, if any.
    pub subtitle: Option<String>,
    /// Facts on the card (role `Compact`), at most [`RecordsBinding::facts_limit`] are drawn, the rest is counted ("+ N more").
    pub facts: Vec<Slot>,
    /// Slots of the frame at the side (role `Display`).
    pub detail: Vec<Slot>,
    /// Columns of the list (role `Cell`).
    pub cells: Vec<Slot>,
    /// The page size asked for.
    pub page_size: u32,
    /// How many pages are read at most; more is said, never cut silently.
    pub max_pages: u32,
    /// How many facts a card takes: the effective setting (the host reads it), [`fs_components_kanban::board::DEFAULT_FACTS`] by default.
    pub facts_limit: Limit,
    /// How many elements the frame takes: the effective setting, [`fs_components_kanban::board::DEFAULT_ELEMENTS`] by default.
    pub elements_limit: Limit,
}

impl RecordsBinding {
    /// A binding with a type, a grouping and a title field.
    pub fn new(
        data_type: impl Into<String>,
        grouping: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            data_type: data_type.into(),
            scope: None,
            grouping: grouping.into(),
            order: None,
            title: title.into(),
            subtitle: None,
            facts: Vec::new(),
            detail: Vec::new(),
            cells: Vec::new(),
            page_size: 100,
            max_pages: 10,
            facts_limit: DEFAULT_FACTS,
            elements_limit: DEFAULT_ELEMENTS,
        }
    }

    /// Of one container.
    #[must_use]
    pub fn scoped(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// With a named order.
    #[must_use]
    pub fn ordered(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// With a second line.
    #[must_use]
    pub fn subtitle(mut self, field: impl Into<String>) -> Self {
        self.subtitle = Some(field.into());
        self
    }

    /// With a slot in a place.
    #[must_use]
    pub fn slot(mut self, slot: Slot) -> Self {
        match slot.role {
            Role::Compact => self.facts.push(slot),
            Role::Display | Role::Input => self.detail.push(slot),
            Role::Cell => self.cells.push(slot),
        }
        self
    }

    /// With the limits of the card and the frame (the effective settings).
    #[must_use]
    pub fn limits(mut self, facts: Limit, elements: Limit) -> Self {
        self.facts_limit = facts;
        self.elements_limit = elements;
        self
    }

    /// Every field one `find` must ask for, each once, in order.
    pub fn fields(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let all = std::iter::once(self.title.as_str())
            .chain(self.subtitle.as_deref())
            .chain(self.facts.iter().map(|s| s.field.as_str()))
            .chain(self.detail.iter().map(|s| s.field.as_str()))
            .chain(self.cells.iter().map(|s| s.field.as_str()));
        for f in all {
            if !out.iter().any(|o| o == f) {
                out.push(f.to_owned());
            }
        }
        out
    }
}

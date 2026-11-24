//! What a list is bound to: the type, the grouping, and the fields that stand in the columns of the list. The names come from the
//! description of the page (data), never from here.
//!
//! Pattern: Value objects (a Builder-style chain). No program name, no field name, no block name is fixed in this crate.

/// One slot: a field, the name the reader sees and the building block the catalog chose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    /// The top-level field.
    pub field: String,
    /// The name the reader sees (already in her language).
    pub label: String,
    /// The building block `BlockCatalog::choose` named for this field in the role of a table cell (`field-email`); from the plan,
    /// never chosen here.
    pub block: String,
}

impl Slot {
    /// A slot.
    pub fn new(
        field: impl Into<String>,
        label: impl Into<String>,
        block: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
            block: block.into(),
        }
    }
}

/// The binding of one list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordsBinding {
    /// The data type.
    pub data_type: String,
    /// The container record, if the view is of one.
    pub scope: Option<String>,
    /// The grouping the records are asked in.
    pub grouping: String,
    /// The named order (`rank`); `None` = the provider's default.
    pub order: Option<String>,
    /// Columns of the list (role cell).
    pub cells: Vec<Slot>,
    /// The page size asked for.
    pub page_size: u32,
    /// How many pages are read at most; more is said, never cut silently.
    pub max_pages: u32,
}

impl RecordsBinding {
    /// A binding with a type and a grouping.
    pub fn new(data_type: impl Into<String>, grouping: impl Into<String>) -> Self {
        Self {
            data_type: data_type.into(),
            scope: None,
            grouping: grouping.into(),
            order: None,
            cells: Vec::new(),
            page_size: 100,
            max_pages: 10,
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

    /// With a column of the list.
    #[must_use]
    pub fn cell(mut self, slot: Slot) -> Self {
        self.cells.push(slot);
        self
    }

    /// Every field one `find` must ask for, each once, in order.
    pub fn fields(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for f in self.cells.iter().map(|s| s.field.as_str()) {
            if !out.iter().any(|o| o == f) {
                out.push(f.to_owned());
            }
        }
        out
    }
}

//! The groups the list can be narrowed to.
//!
//! Pattern: none — plain values. (The kind of a field, `FieldKind`, belongs to the page of a record in `fs-components-widgets`.)

/// A group the list can be narrowed to (the filter in the menu of the list).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupFilter {
    /// Names the group for the source; opaque to the workbench.
    pub key: String,
    /// What to call it.
    pub title: String,
}

impl GroupFilter {
    /// A group.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
        }
    }
}

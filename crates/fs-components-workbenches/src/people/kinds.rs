//! What kind of thing a field holds, and the groups the list can be narrowed to.
//!
//! Pattern: none — plain values. The kind of a field only chooses its symbol and its actions on the card. The source says it (the kind
//! comes from the data and the contract); the workbench never guesses it from the value.

/// What kind of thing a field holds, so its card has the right symbol and the right actions.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldKind {
    /// Not said: the card has the plain symbol and only the copy action.
    #[default]
    Text,
    /// An e-mail address: write a mail, copy.
    Mail,
    /// A phone number: copy.
    Phone,
    /// A postal address: copy.
    Address,
    /// A date (birthday, ...): copy.
    Date,
}

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

//! What kind of thing a field holds, and the groups the list can be narrowed to.
//!
//! Pattern: none — plain values. The kind of a field only chooses its symbol and its actions on the card; a guess from the value is
//! offered for a source that says nothing, and what the source says always wins.

/// What kind of thing a field holds, so its card has the right symbol and the right actions.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldKind {
    /// Not said: [`PersonField::effective_kind`] looks at the value.
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

impl FieldKind {
    /// The kind a value looks like: `@` between two words and no space is a mail address, digits with `+ - ( ) /` and spaces (at least
    /// five digits) a phone number, anything else plain text. Only a guess for a source that does not say.
    pub fn guess(value: &str) -> Self {
        let v = value.trim();
        if let Some((local, domain)) = v.split_once('@')
            && !local.is_empty()
            && domain.contains('.')
            && !v.contains(char::is_whitespace)
        {
            return Self::Mail;
        }
        let digits = v.chars().filter(char::is_ascii_digit).count();
        let only_phone = v
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '(' | ')' | '/' | ' ' | '.'));
        if digits >= 5 && only_phone {
            return Self::Phone;
        }
        Self::Text
    }
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

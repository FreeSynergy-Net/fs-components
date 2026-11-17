//! What a people source can do.
//!
//! Pattern: none — a plain value; a menu line the source cannot do is not offered (ADR 0037).

/// What a source can do; an action it cannot do is not offered.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PeopleCapabilities {
    /// Change a person.
    pub edit: bool,
    /// Make a new person.
    pub create: bool,
    /// Delete a person.
    pub delete: bool,
    /// Hand people out (vCard, CSV): the menu offers Export, the program does it.
    pub export: bool,
    /// Merge several people into one: the menu offers Merge, the program does it.
    pub merge: bool,
}

impl PeopleCapabilities {
    /// Reading only.
    pub fn read_only() -> Self {
        Self::default()
    }

    /// Everything.
    pub fn all() -> Self {
        Self {
            edit: true,
            create: true,
            delete: true,
            export: false,
            merge: false,
        }
    }

    /// This, and the program can export people.
    #[must_use]
    pub fn with_export(mut self) -> Self {
        self.export = true;
        self
    }

    /// This, and the program can merge people.
    #[must_use]
    pub fn with_merge(mut self) -> Self {
        self.merge = true;
        self
    }
}

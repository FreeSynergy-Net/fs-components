//! What a tile talks about.
//!
//! Pattern: Value object. The facts are the fields of the detail page ([`Field`]: one label, one or several values, what it holds), so
//! a fact and a field of a page are the same thing, read by one rule.

use crate::detail_page::Field;

/// One tile.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    /// What the caller gets back; part of the ids.
    pub key: String,
    /// The number in the head (its place, or what the source says in `order`).
    pub number: String,
    /// What to call the record.
    pub title: String,
    /// The round picture of the record: a `data:` URI, a local file or a URL. `None` = no picture: the face shows the initials of the
    /// title (a picture that cannot be loaded is never drawn). Only set when the source's block for the preview is `person-face`.
    pub image: Option<String>,
    /// Whether the body shows the face at all (the preview is a face; a record with another kind of preview has none here).
    pub face: bool,
    /// The facts, in order.
    pub facts: Vec<Field>,
    /// The state in the module's words (empty = none).
    pub state: String,
}

impl Tile {
    /// A tile with the face of `title`.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            face: true,
            ..Self::default()
        }
    }

    /// This tile with its number.
    #[must_use]
    pub fn numbered(mut self, number: impl Into<String>) -> Self {
        self.number = number.into();
        self
    }

    /// This tile with a picture.
    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// This tile with facts.
    #[must_use]
    pub fn with_facts(mut self, facts: Vec<Field>) -> Self {
        self.facts = facts;
        self
    }

    /// This tile with a state.
    #[must_use]
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }

    /// This tile without a face (another kind of preview, which this element does not draw).
    #[must_use]
    pub fn without_face(mut self) -> Self {
        self.face = false;
        self
    }
}

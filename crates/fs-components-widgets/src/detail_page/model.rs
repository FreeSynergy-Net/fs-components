//! What the page of one record talks about.
//!
//! Pattern: Value objects, neutral: sections of labelled fields. Nothing here knows what a record is made of (a person, a task, a
//! program); the kind of a field only chooses its symbol and its actions on the card.

use crate::icon::Glyph;

/// What kind of thing a field holds, so its card has the right symbol and the right actions. The source says it (from the type of
/// the field, the block of the contract); the page never guesses it from the value.
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

impl FieldKind {
    /// The kind a building block of the contract (`field-email`, `field-phone` ...) stands for; any other block is plain text.
    #[must_use]
    pub fn of_block(block: &str) -> Self {
        match block {
            "field-email" => Self::Mail,
            "field-phone" => Self::Phone,
            "field-address" => Self::Address,
            "field-date" | "field-date-time" => Self::Date,
            _ => Self::Text,
        }
    }

    /// The symbol of the kind.
    #[must_use]
    pub fn glyph(self) -> Glyph {
        match self {
            Self::Mail => Glyph::Mail,
            Self::Phone => Glyph::Phone,
            Self::Address => Glyph::Place,
            Self::Date => Glyph::Calendar,
            Self::Text => Glyph::Info,
        }
    }
}

/// One labelled field of a record: one label, one or several values (RFC 6350 / RFC 9553: `emails` has one entry per address).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// What the field is.
    pub label: String,
    /// The values, in order; empty ones are not drawn.
    pub values: Vec<String>,
    /// What it holds; `Text` = not said.
    pub kind: FieldKind,
}

impl Field {
    /// A field with one value.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self::many(label, vec![value.into()])
    }

    /// A field with several values (one card each, under the one label).
    pub fn many(label: impl Into<String>, values: Vec<String>) -> Self {
        Self {
            label: label.into(),
            values,
            kind: FieldKind::Text,
        }
    }

    /// This field, saying what it holds.
    #[must_use]
    pub fn kind(mut self, kind: FieldKind) -> Self {
        self.kind = kind;
        self
    }

    /// The values that are drawn: not empty after trimming.
    pub fn shown(&self) -> impl Iterator<Item = &str> {
        self.values
            .iter()
            .map(String::as_str)
            .filter(|v| !v.trim().is_empty())
    }
}

/// Fields that belong together, under a heading.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// The heading; an empty text shows none.
    pub title: String,
    /// The fields.
    pub fields: Vec<Field>,
}

impl Section {
    /// A section.
    pub fn new(title: impl Into<String>, fields: Vec<Field>) -> Self {
        Self {
            title: title.into(),
            fields,
        }
    }
}

/// One record, to be read.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sheet {
    /// What to call the record.
    pub title: String,
    /// A short text under the title (empty = none).
    pub summary: String,
    /// The state of the record in the module's words (empty = none).
    pub state: String,
    /// Short markings that must be seen at once (an AI, sensitive data), in the source's words.
    pub marks: Vec<String>,
    /// The details.
    pub sections: Vec<Section>,
    /// A picture: a `data:` URI, a local file or a URL. Without one (or one that cannot be loaded) the sheet shows the initials of
    /// the title.
    pub image: Option<String>,
}

impl Sheet {
    /// A sheet.
    pub fn new(title: impl Into<String>, sections: Vec<Section>) -> Self {
        Self {
            title: title.into(),
            sections,
            ..Self::default()
        }
    }

    /// This sheet with a picture.
    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// This sheet with a short text under the title.
    #[must_use]
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// This sheet with a state.
    #[must_use]
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }
}

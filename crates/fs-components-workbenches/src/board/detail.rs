//! The frame at the side that shows one card: where it is, its title, its properties, one main action.
//!
//! Pattern: Template method (the frame is the workbench's) + Strategy (`DetailContent`: what stands in it is the program's).
//! The frame is not modal: the board stays usable beside it. An empty value is drawn as the grey word "set" (a way to fill it),
//! never as a blank.

/// What one element slot of the frame shows (K6): a value, nothing yet, or the reason there is no value. A reason is never a blank
/// and never the value itself.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotValue {
    /// The value, worded for the reader.
    Text(String),
    /// The field has no value (drawn as the grey word "set").
    Unset,
    /// The reader may not see the field: the reason, not the value.
    Hidden(super::model::Reason),
    /// The field or its source is not there (a missing component, a source that does not answer): the code and the sentence.
    Missing(super::model::Reason),
}

/// One element slot: a name and what stands in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailSlot {
    /// The field it comes from (for tests and for the program; not shown).
    pub field: String,
    /// The name the reader sees.
    pub name: String,
    /// What stands there.
    pub value: SlotValue,
}

/// What the program puts into the frame for one card.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DetailFrame {
    /// The path above the title: "List > Column", as the program words it.
    pub path: String,
    /// The title.
    pub title: String,
    /// Name and value; an empty value means "not set".
    pub properties: Vec<(String, String)>,
    /// Element slots (K6) after the properties: a value or the reason there is none.
    pub slots: Vec<DetailSlot>,
    /// The main button (for example "Done"): its id and its text.
    pub primary: Option<(String, String)>,
}

impl DetailFrame {
    /// A frame with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// With the path.
    #[must_use]
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// With one more property.
    #[must_use]
    pub fn property(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((name.into(), value.into()));
        self
    }

    /// With one more element slot.
    #[must_use]
    pub fn slot(mut self, slot: DetailSlot) -> Self {
        self.slots.push(slot);
        self
    }

    /// With the main button.
    #[must_use]
    pub fn primary(mut self, id: impl Into<String>, text: impl Into<String>) -> Self {
        self.primary = Some((id.into(), text.into()));
        self
    }
}

/// The program's content of the frame.
pub trait DetailContent: Send + Sync {
    /// The frame of `card`, or `None` when there is nothing to show (the card is gone).
    fn frame(&self, card: &str) -> Option<DetailFrame>;

    /// The main button was pressed.
    fn primary(&self, _card: &str, _id: &str) {}
}

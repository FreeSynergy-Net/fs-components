//! The frame at the side that shows one card: where it is, its title, its properties, one main action.
//!
//! Pattern: Template method (the frame is the workbench's) + Strategy (`DetailContent`: what stands in it is the program's).
//! The frame is not modal: the board stays usable beside it. An empty value is drawn as the grey word "set" (a way to fill it),
//! never as a blank.

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

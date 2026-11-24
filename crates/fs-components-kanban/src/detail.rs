//! The frame at the side that shows one card: where it is, its title, its properties, one main action.
//!
//! Pattern: Template method (the frame is the workbench's) + Strategy (`DetailContent`: what stands in it is the program's).
//! The frame is not modal: the board stays usable beside it. An empty value is drawn as the grey word "set" (a way to fill it),
//! never as a blank. Besides the plain `properties` (name and text) the frame takes `elements`: a field value for the slot
//! `properties` of the block `board-detail`, drawn by the display component of the block the catalog chose.

use crate::{FieldSlot, Limit};

use crate::limits::DEFAULT_ELEMENTS;

/// What the program puts into the frame for one card.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailFrame {
    /// The path above the title: "List > Column", as the program words it.
    pub path: String,
    /// The title.
    pub title: String,
    /// Name and value; an empty value means "not set".
    pub properties: Vec<(String, String)>,
    /// Name (already in the reader's language) and the field value; an empty value means "not set". Private: only
    /// [`DetailFrame::element`] adds, and it checks the limit of the frame.
    elements: Vec<(String, FieldSlot)>,
    /// How many elements the frame takes: the setting of the instance (default [`DEFAULT_ELEMENTS`]).
    max_elements: Limit,
    /// How many elements were handed in beyond the limit ([`DetailFrame::element_or_more`]): the frame says "+ N more".
    more_elements: usize,
    /// The main button (for example "Done"): its id and its text.
    pub primary: Option<(String, String)>,
}

impl DetailFrame {
    /// A frame with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self::limited(title, DEFAULT_ELEMENTS)
    }

    /// A frame that takes up to `max_elements` elements (the effective value of the setting
    /// `fs.workbenches.board-detail-elements`, see [`crate::ELEMENTS_SETTING`]).
    pub fn limited(title: impl Into<String>, max_elements: Limit) -> Self {
        Self {
            path: String::new(),
            title: title.into(),
            properties: Vec::new(),
            elements: Vec::new(),
            max_elements,
            more_elements: 0,
            primary: None,
        }
    }

    /// The elements, in order.
    pub fn elements(&self) -> &[(String, FieldSlot)] {
        &self.elements
    }

    /// How many elements the frame takes.
    pub fn max_elements(&self) -> usize {
        self.max_elements.get()
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

    /// How many elements did not fit (the frame shows "+ N more").
    pub fn more_elements(&self) -> usize {
        self.more_elements
    }

    /// With one more element, or counted as "more" when the frame is full: nothing is cut silently.
    #[must_use]
    pub fn element_or_more(mut self, name: impl Into<String>, slot: FieldSlot) -> Self {
        if self.elements.len() >= self.max_elements.get() {
            self.more_elements += 1;
        } else {
            self.elements.push((name.into(), slot));
        }
        self
    }

    /// With one more element: a field value in the words of its component.
    ///
    /// # Errors
    /// The slot comes back when the frame already has [`DetailFrame::max_elements`] elements: nothing is dropped silently.
    pub fn element(mut self, name: impl Into<String>, slot: FieldSlot) -> Result<Self, FieldSlot> {
        if self.elements.len() >= self.max_elements.get() {
            return Err(slot);
        }
        self.elements.push((name.into(), slot));
        Ok(self)
    }

    /// With the main button.
    #[must_use]
    pub fn primary(mut self, id: impl Into<String>, text: impl Into<String>) -> Self {
        self.primary = Some((id.into(), text.into()));
        self
    }
}

/// Pattern: Strategy - the program decides what the frame of a card shows; the board only places it.
///
/// The program's content of the frame.
pub trait DetailContent: Send + Sync {
    /// The frame of `card`, or `None` when there is nothing to show (the card is gone).
    fn frame(&self, card: &str) -> Option<DetailFrame>;

    /// The main button was pressed.
    fn primary(&self, _card: &str, _id: &str) {}
}

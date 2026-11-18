//! One field value as a slot of a larger block (a fact on a card, an element in a detail frame): the name of the building
//! block the catalog chose, the label key, and the stored value. The slot draws nothing by itself and chooses nothing: the
//! block name comes from `BlockCatalog::choose`, the host hands it over, and the frame only calls [`FieldSlot::compact_view`]
//! or [`FieldSlot::display_view`].
//!
//! Pattern: Value object + Factory (block name -> component of [`Kind`]). A block without a component here (its type is not
//! done yet) is shown as the canonical text of the value, the same placeholder the Admin frontend uses; it is never dropped.

use fs_components_widgets::Element;

use crate::kinds::Kind;
use crate::view::{Look, Say, compact, display};

/// A value for a slot of a card or a frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSlot {
    /// The building block the catalog chose (`field-email` ...).
    pub block: String,
    /// The label as a translation key (`coding.md` 2a), never a sentence.
    pub label: String,
    /// The value as the program stores it (the canonical text of the type).
    pub value: String,
}

impl FieldSlot {
    /// A slot.
    pub fn new(block: &str, label: &str, value: &str) -> Self {
        Self {
            block: block.to_owned(),
            label: label.to_owned(),
            value: value.to_owned(),
        }
    }

    /// The kind that draws it, if its block has a component here.
    pub fn kind(&self) -> Option<Kind> {
        Kind::from_block(&self.block)
    }

    /// The parts of the value (one part for a block without a component).
    pub fn parts(&self) -> Vec<String> {
        match self.kind() {
            Some(kind) => kind.field().split(&self.value),
            None => vec![self.value.clone()],
        }
    }

    /// The value in one line, as the compact component says it (the value itself for a block without a component).
    pub fn line(&self) -> String {
        match self.kind() {
            Some(kind) => kind.field().compact(&self.parts()),
            None => self.value.clone(),
        }
    }

    /// The compact component (a fact of a card).
    pub fn compact_view(&self, look: &Look) -> Box<dyn Element> {
        match self.kind() {
            Some(kind) => Box::new(compact(kind, &self.parts(), look)),
            None => Box::new(look.text(self.line(), look.style.text.body, look.palette().text)),
        }
    }

    /// The display component (an element of a detail frame).
    pub fn display_view(&self, look: &Look, say: &dyn Say) -> Box<dyn Element> {
        match self.kind() {
            Some(kind) => Box::new(display(kind, &self.parts(), look, say)),
            None => Box::new(look.text(self.line(), look.style.text.body, look.palette().text)),
        }
    }
}

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

/// Why a slot shows no value: the ONE reason type of the building blocks, defined in `fs-components-core` (`answer`) because the card
/// (a widget below this crate) says "why not" with the same type. Re-exported under its old name: no second type, no conversion.
pub use fs_components_widgets::answer::Reason as SlotReason;

/// What a slot stands for. The state is said by the plan; the slot draws it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SlotState {
    /// A value (it may be an empty text: then the frame shows the grey word "set").
    #[default]
    Value,
    /// The field has no value (an invitation to fill it, where the frame allows it).
    Unset,
    /// The reader may not see the field: the reason, never the value (the slot carries none).
    Hidden(Box<SlotReason>),
    /// The field or its building block is not there: the reason.
    Missing(Box<SlotReason>),
}

/// A value for a slot of a card or a frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldSlot {
    /// The state; `Value` for a slot made by [`FieldSlot::new`].
    pub state: SlotState,
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
            state: SlotState::Value,
            block: block.to_owned(),
            label: label.to_owned(),
            value: value.to_owned(),
        }
    }

    /// A slot of a field without a value.
    pub fn unset(block: &str, label: &str) -> Self {
        Self {
            state: SlotState::Unset,
            ..Self::new(block, label, "")
        }
    }

    /// A slot of a field the reader may not see. It carries NO value, so none can be drawn.
    pub fn hidden(block: &str, label: &str, reason: SlotReason) -> Self {
        Self {
            state: SlotState::Hidden(Box::new(reason)),
            ..Self::new(block, label, "")
        }
    }

    /// A slot of a field or block that is not there. It carries no value.
    pub fn missing(block: &str, label: &str, reason: SlotReason) -> Self {
        Self {
            state: SlotState::Missing(Box::new(reason)),
            ..Self::new(block, label, "")
        }
    }

    /// The reason, when the slot is hidden or missing.
    pub fn reason(&self) -> Option<&SlotReason> {
        match &self.state {
            SlotState::Hidden(r) | SlotState::Missing(r) => Some(r),
            _ => None,
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
        if let Some(reason) = self.reason() {
            return reason.said().to_owned();
        }
        match self.kind() {
            Some(kind) => kind.field().compact(&self.parts()),
            None => self.value.clone(),
        }
    }

    /// The compact component (a fact of a card).
    pub fn compact_view(&self, look: &Look) -> Box<dyn Element> {
        if self.reason().is_some() {
            return Box::new(look.text(self.line(), look.style.text.body, look.palette().text));
        }
        match self.kind() {
            Some(kind) => Box::new(compact(kind, &self.parts(), look)),
            None => Box::new(look.text(self.line(), look.style.text.body, look.palette().text)),
        }
    }

    /// The width [`FieldSlot::compact_view`] takes in `look` (icon, gap, text): the measure for sharing a room between facts.
    pub fn compact_width(&self, look: &Look) -> f32 {
        match (self.reason(), self.kind()) {
            (None, Some(kind)) => crate::view::compact_width(kind, &self.parts(), look),
            _ => crate::view::text_width(&self.line(), look),
        }
    }

    /// The compact component with `text` instead of the value (a value cut to its room): same look as [`FieldSlot::compact_view`].
    pub fn compact_view_text(&self, look: &Look, text: &str) -> Box<dyn Element> {
        match (self.reason(), self.kind()) {
            (None, Some(kind)) => Box::new(crate::view::compact_text(kind, text, look)),
            _ => Box::new(look.text(text, look.style.text.body, look.palette().text)),
        }
    }

    /// The display component (an element of a detail frame).
    pub fn display_view(&self, look: &Look, say: &dyn Say) -> Box<dyn Element> {
        if self.reason().is_some() {
            return Box::new(look.text(self.line(), look.style.text.body, look.palette().text));
        }
        match self.kind() {
            Some(kind) => Box::new(display(kind, &self.parts(), look, say)),
            None => Box::new(look.text(self.line(), look.style.text.body, look.palette().text)),
        }
    }
}

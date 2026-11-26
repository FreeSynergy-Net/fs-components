//! The field card as a [`FormView`] of the form `rows`: one card per row, the first value of `columns` is the label (the role), the
//! channel `label` the value (the person).
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, cards out).

use std::sync::{Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::icon::Glyph;

use super::FieldCard;

/// The widget this view is (`widget` of the block `field-card`).
pub const WIDGET: &str = "FieldCard";

/// The view of the block `field-card`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, _ctx: DrawCtx) -> Box<dyn Mounted> {
        Box::new(MountedCards(Mutex::new(input)))
    }
}

struct MountedCards(Mutex<FormInput>);

impl Mounted for MountedCards {
    fn build(&self, width: f32, _height: f32) -> Div {
        let input = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let mut out = div().w(width).flex_col();
        for (n, row) in input.rows.iter().enumerate() {
            // the card is labelled by the value of the FIRST field of the channel `columns` (found by its path, not by position)
            let label = input
                .channel("columns")
                .and_then(|c| c.fields.first())
                .and_then(|f| row.field("columns", &f.name).first().copied())
                .unwrap_or("");
            let card = FieldCard::new(Glyph::Person, label, row.get("label").unwrap_or(&row.key));
            out = out.child(card.build(&format!("field-card-{n}")));
        }
        out
    }

    fn refresh(&self, input: FormInput) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = input;
    }
}

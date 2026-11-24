//! `field-card`: one field of a person as a rounded card, the symbol of the kind of field at the left and the actions at the right
//! (write a mail, copy), as in GNOME Contacts.
//!
//! Pattern: Composite (symbol, text, actions) + Command (each action is a callback with its own words). The card knows no
//! person: the caller gives what the field is called, what it holds and which actions it offers. The label is small and
//! above the value; an empty label shows only the value. Every action is a tab stop with its words as `aria-label`.

mod card;
pub mod form_view;

pub use card::{Act, CardAction, FieldCard, STYLES};

//! The element: the sheet as a centred page.
//!
//! Pattern: Composite — head (face, title, summary, state, marks), then the sections, then the fields as `field-card`s. A field with
//! several values makes one card per value, the label only on the first; a field without a value is not drawn. The card actions
//! come from the caller (it knows the commands; the page knows none).

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::{Field, Section, Sheet};
use crate::field_card::{CardAction, FieldCard};
use crate::person_face::PersonFace;

/// The diameter of the big face, in pixels.
pub const FACE: f32 = 112.0;

/// The style sheet of the page.
pub const STYLES: &str = r"
.fs-detail-head { align-items: center; gap: var(--space-3); }
.fs-detail-page { gap: var(--space-5); }
.fs-detail-frame { padding: var(--space-6) var(--space-4); }
.fs-detail-face { flex-shrink: 0; }
.fs-detail-title-text { font-size: var(--text-2xl); font-weight: 700; text-align: center; }
.fs-detail-summary { color: var(--text-secondary); font-size: var(--text-base); text-align: center; }
.fs-detail-state { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-detail-mark { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); background: var(--warning-bg); color: var(--text-primary); font-size: var(--text-sm); }
.fs-detail-cards { max-width: 520px; gap: var(--space-4); }
.fs-detail-section { gap: var(--space-2); }
.fs-detail-section-title { color: var(--text-secondary); font-size: var(--text-sm); font-weight: 600; }
.fs-detail-empty { color: var(--text-secondary); padding: var(--space-4); }
";

/// The page of `sheet`, centred, to be put into a column of the caller. `name` is the base of the ids of the cards
/// (`<name>-field-<section>-<field>`, a further value of the same field adds `-<n>`); `actions` gives the actions of the card of
/// one value; `empty` is what to say when no field has a value (empty = nothing).
pub fn page(
    sheet: &Sheet,
    name: &str,
    empty: &str,
    actions: &dyn Fn(&Field, &str) -> Vec<CardAction>,
) -> Div {
    let mut cards = div().class("fs-detail-cards").flex_col().w_full();
    let mut shown = 0usize;
    for (s, section) in sheet.sections.iter().enumerate() {
        if let Some(element) = section_element(section, name, s, actions) {
            cards = cards.child(element);
            shown += 1;
        }
    }
    if shown == 0 && !empty.is_empty() {
        cards = cards.child(div().class("fs-detail-empty").child(text(empty)));
    }
    div()
        .class("fs-detail-page")
        .flex_col()
        .items_center()
        .w_full()
        .child(head(sheet))
        .child(cards)
}

fn head(sheet: &Sheet) -> Div {
    let face = PersonFace::new(&sheet.title)
        .image(sheet.image.clone())
        .size(FACE)
        .build();
    let mut head = div()
        .class("fs-detail-head")
        .flex_col()
        .items_center()
        .child(div().class("fs-detail-face").child(face))
        .child(div().child(text(&sheet.title).class("fs-detail-title-text")));
    for (class, line) in [
        ("fs-detail-summary", &sheet.summary),
        ("fs-detail-state", &sheet.state),
    ] {
        if !line.trim().is_empty() {
            head = head.child(div().class(class).child(text(line)));
        }
    }
    for mark in &sheet.marks {
        head = head.child(div().class("fs-detail-mark").child(text(mark)));
    }
    head
}

fn section_element(
    section: &Section,
    name: &str,
    number: usize,
    actions: &dyn Fn(&Field, &str) -> Vec<CardAction>,
) -> Option<Div> {
    let fields: Vec<&Field> = section
        .fields
        .iter()
        .filter(|f| f.shown().next().is_some())
        .collect();
    if fields.is_empty() {
        return None;
    }
    let mut element = div().class("fs-detail-section").flex_col().w_full();
    if !section.title.is_empty() {
        element = element.child(
            div()
                .class("fs-detail-section-title")
                .child(text(&section.title)),
        );
    }
    for (f, field) in fields.into_iter().enumerate() {
        for (v, value) in field.shown().enumerate() {
            // several values of one field: ONE label, on the first card
            let label = if v == 0 { field.label.as_str() } else { "" };
            let mut card = FieldCard::new(field.kind.glyph(), label, value.trim());
            for action in actions(field, value.trim()) {
                card = card.action(action);
            }
            let id = if v == 0 {
                format!("{name}-field-{number}-{f}")
            } else {
                format!("{name}-field-{number}-{f}-{v}")
            };
            element = element.child(card.build(&id));
        }
    }
    Some(element)
}

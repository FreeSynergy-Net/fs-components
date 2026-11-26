//! One tile as a card: head (number, title), body (face, facts), foot (state).
//!
//! Pattern: Adapter over the general `card` (variant `shared-lines`): the parts are made here, the frame, the pointer and the keys are
//! the card's. A card without targets is only clickable.

use std::fmt::Write;

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::Tile;
use crate::card::{Card, CardRef};
use crate::person_face::PersonFace;

/// The diameter of the face in a tile, in pixels.
pub const FACE: f32 = 48.0;

/// The reference a tile stands for: its key as the IRI of a `urn:` (the key is percent-encoded, so any key is an absolute IRI).
fn card_ref(key: &str) -> CardRef {
    let mut iri = String::from("urn:fs:tile:");
    for b in key.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            iri.push(char::from(b));
        } else {
            let _ = write!(iri, "%{b:02X}");
        }
    }
    CardRef::parse(&iri, "application/activity+json")
        .expect("a urn of encoded text is an absolute IRI")
}

/// The element of `tile`; `id` is the id of the card.
pub fn element(tile: &Tile, id: &str) -> Div {
    let head = div()
        .class("fs-card-header")
        .gap(8.0)
        .child(text(&tile.number).class("fs-tile-number").no_wrap())
        .child(text(&tile.title).class("fs-card-title"));
    let mut body = div().class("fs-card-body").flex_row().gap(12.0);
    if tile.face {
        body = body.child(
            div().class("fs-tile-face").child(
                PersonFace::new(&tile.title)
                    .image(tile.image.clone())
                    .size(FACE)
                    .build(),
            ),
        );
    }
    body = body.child(facts(tile));
    let mut parts = vec![head, body];
    if !tile.state.trim().is_empty() {
        parts.push(
            div()
                .class("fs-card-footer")
                .child(text(&tile.state).class("fs-tile-state")),
        );
    }
    Card::new(id, card_ref(&tile.key))
        .shared_lines()
        .build(parts)
}

/// The facts: each under its label (once, even with several values), a fact without a value is not drawn.
fn facts(tile: &Tile) -> Div {
    let mut out = div().class("fs-tile-facts").flex_col().flex_grow().gap(2.0);
    for fact in &tile.facts {
        for (i, value) in fact.shown().enumerate() {
            let mut line = div().flex_col();
            if i == 0 && !fact.label.is_empty() {
                line = line.child(text(&fact.label).class("fs-tile-fact-label"));
            }
            out = out.child(line.child(text(value.trim()).class("fs-tile-fact")));
        }
    }
    out
}

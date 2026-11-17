//! P1 probe: a person at a keyboard, without a screen: the window's router and key broadcast, fed by the test.
//!
//! Tests: helper of `festzeichen.rs`.

use std::sync::Arc;

use blinc_core::events::event_types::{KEY_DOWN, KEY_UP};
use blinc_layout::prelude::RenderTree;

use super::address::{AddressField, Mutation};
use super::common::{Input, frame, layout};

pub const W: f32 = 500.0;
pub const H: f32 = 120.0;

pub struct Typist {
    pub field: Arc<AddressField>,
    pub tree: RenderTree,
    input: Input,
}

impl Typist {
    pub fn new(m: Mutation) -> Self {
        Self::with(m)
    }

    pub fn with(m: Mutation) -> Self {
        let field = AddressField::new(m, &["org", "com", "de", "example"]).unwrap();
        let tree = layout(Arc::clone(&field).view(false), W, H);
        Self {
            field,
            tree,
            input: Input::new(),
        }
    }

    /// Click a part (first part when nothing was clicked yet) so that it has the keyboard, then type.
    pub fn click(&mut self, id: &str) {
        self.input.click(&mut self.tree, id);
        frame(&mut self.tree, W, H);
    }

    pub fn text(&mut self, s: &str) {
        if self.field.focus().is_none() {
            self.field.go(0, Some(0));
        }
        for c in s.chars() {
            self.tree
                .broadcast_text_input_event(c, false, false, false, false);
            frame(&mut self.tree, W, H);
        }
    }

    pub fn press(&mut self, code: u32, ctrl: bool) {
        self.tree
            .broadcast_key_event(KEY_DOWN, code, false, ctrl, false, false);
        frame(&mut self.tree, W, H);
        // a window broadcasts the release of every key too
        self.tree
            .broadcast_key_event(KEY_UP, code, false, ctrl, false, false);
        frame(&mut self.tree, W, H);
    }

    /// A key in the order of a window: navigation (Tab) first, the broadcast only if it did not take the key.
    pub fn window_key(&mut self, code: u32) {
        self.input.key(&mut self.tree, code);
        frame(&mut self.tree, W, H);
    }

    pub fn keys(&mut self, code: u32, times: usize) {
        for _ in 0..times {
            self.press(code, false);
        }
    }
}

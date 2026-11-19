//! The entries that are not a plain text field: the switch of a yes/no type and the wrapper of a number field.
//!
//! Pattern: Adapter over the `Switch` of the controls and over Blinc's `text_input` (the number field itself is built in
//! `input.rs`; Blinc's `cn::number_input` is not used: its value is an `f64`, which loses a 64-bit integer above 2^53, its heights
//! are fixed at three sizes, and it sets no ARIA role). The role `spinbutton` sits on the element around the field, because a
//! Blinc `text_input` takes no attribute; `aria-valuenow` is the text at the time the form was drawn.

use std::sync::Arc;

use blinc_layout::div::{Div, div};

use super::input::Inner;
use crate::controls::{Switch, SwitchState};

/// The part `i` of a yes/no type: a switch with the words `label.flag.flag`; flipping it answers.
pub(super) fn switch_row(inner: &Arc<Inner>, i: usize) -> Div {
    let name = inner.name.as_deref().unwrap_or("flag");
    let on = Arc::clone(inner);
    let yes = inner.parts()[i] == "true";
    let switch = Switch::new(&inner.say.say("label.flag.flag"))
        .id(&format!("{name}-flag"))
        .state_is(if yes {
            SwitchState::On
        } else {
            SwitchState::Off
        })
        .on_change(move |state| on.answered(state == SwitchState::On));
    div().child(switch.build())
}

/// `cell` (the field in its row) as a spinbutton: role, the range, the number at the time of drawing.
pub(super) fn number_wrap(cell: Div, id: Option<&str>, signed: bool, now: &str) -> Div {
    let mut wrap = div().attr("role", "spinbutton");
    if let Some(id) = id {
        wrap = wrap.id(format!("{id}-spin"));
    }
    if !signed {
        wrap = wrap.attr("aria-valuemin", "0");
    }
    if !now.trim().is_empty() {
        wrap = wrap.attr("aria-valuenow", now.trim());
    }
    wrap.w_full().flex_grow_value(1.0).min_w(0.0).child(cell)
}

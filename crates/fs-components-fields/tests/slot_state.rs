//! The state of a slot (`SlotState`): hidden and missing carry a reason from the plan and NO value; they are drawn as the reason,
//! never as an empty text and never as the value.
//!
//! Mutants (each makes the named test fail): S1 `hidden` keeps a value -> `hidden_and_missing_carry_no_value`; S2 `line` of a hidden
//! slot is the value -> `the_line_of_a_slot_without_value_is_its_reason`; S3 `compact_view`/`display_view` of a hidden slot draw nothing
//! -> `a_reason_is_drawn`; S4 `unset` gets a reason -> `unset_has_no_reason_and_no_text`.
//! Pattern: none; tests. Layout only, no graphics adapter.

#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use blinc_layout::div::div;
use blinc_layout::prelude::ElementBuilder;
use common::{layout, serial};
use fs_components_fields::{FieldSlot, KeysAsText, Look, SlotReason, SlotState};
use fs_components_styles::StyleId;

fn reason(text: &str) -> SlotReason {
    SlotReason::new("bind-error-field-hidden", text)
}

#[test]
fn hidden_and_missing_carry_no_value() {
    let h = FieldSlot::hidden("field-email", "l", reason("hidden: no right"));
    let m = FieldSlot::missing("field-email", "l", reason("not built"));
    assert_eq!((h.value.as_str(), m.value.as_str()), ("", ""));
    assert!(matches!(h.state, SlotState::Hidden(_)) && matches!(m.state, SlotState::Missing(_)));
    assert_eq!(
        h.reason().map(|r| r.code.as_str()),
        Some("bind-error-field-hidden")
    );
}

#[test]
fn the_line_of_a_slot_without_value_is_its_reason() {
    let h = FieldSlot::hidden("field-email", "l", reason("hidden: no right"));
    assert_eq!(h.line(), "hidden: no right");
    assert_eq!(
        FieldSlot::missing("field-integer", "l", reason("not built")).line(),
        "not built"
    );
    assert_eq!(FieldSlot::new("field-integer", "l", "7").line(), "7");
}

#[test]
fn unset_has_no_reason_and_no_text() {
    let u = FieldSlot::unset("field-email", "l");
    assert_eq!(
        (u.state.clone(), u.reason(), u.line().as_str()),
        (SlotState::Unset, None, "")
    );
}

#[test]
fn an_unset_slot_carries_no_value_whatever_its_block() {
    // mutant X3: `unset` carries the value " " -> the frame would show a blank instead of the grey "set" (the line is not empty).
    // Checked for blocks with a component (they may trim) and for one without (its line is its value as it is).
    for block in [
        "field-email",
        "field-integer",
        "field-text",
        "a-block-nobody-draws",
    ] {
        let u = FieldSlot::unset(block, "l");
        assert_eq!(u.value, "", "{block}");
        assert_eq!(
            u.line(),
            "",
            "{block}: an empty line is what makes the frame say \"set\""
        );
        assert_eq!(u.state, SlotState::Unset);
    }
}

fn width_of(ui: Box<dyn ElementBuilder>) -> f32 {
    let tree = layout(
        div().id("slot-host").flex_row().w_fit().child(ui),
        600.0,
        200.0,
    );
    let node = tree.query_by_id("slot-host").expect("host");
    tree.get_absolute_bounds(node).expect("bounds").width
}

#[test]
fn a_reason_is_drawn() {
    let _g = serial();
    let look = Look::new(StyleId::Admin, false);
    for block in ["field-email", "field-integer"] {
        let h = FieldSlot::hidden(block, "l", reason("hidden: you may not see this"));
        let (c, d) = (
            width_of(h.compact_view(&look)),
            width_of(h.display_view(&look, &KeysAsText)),
        );
        assert!(c > 20.0 && d > 20.0, "{block}: {c} {d}");
    }
}

#[test]
fn a_slot_stays_small_enough_to_come_back_in_an_err() {
    // `Card::fact` and `DetailFrame::element` return the slot in their `Err`; clippy (`result_large_err`) allows 128 bytes.
    assert!(
        std::mem::size_of::<FieldSlot>() <= 128,
        "{}",
        std::mem::size_of::<FieldSlot>()
    );
}

#[test]
fn a_reason_without_a_sentence_is_said_by_its_code() {
    // mutant: `said` returns the text as it is -> an empty line, the reason vanishes
    let r = SlotReason::new("bind-error-field-hidden", "");
    assert_eq!(r.said(), "bind-error-field-hidden");
    assert_eq!(
        FieldSlot::hidden("field-email", "l", r.clone()).line(),
        "bind-error-field-hidden"
    );
    assert_eq!(
        FieldSlot::missing("field-email", "l", r).line(),
        "bind-error-field-hidden"
    );
    assert_eq!(SlotReason::new("c", "sentence").said(), "sentence");
}

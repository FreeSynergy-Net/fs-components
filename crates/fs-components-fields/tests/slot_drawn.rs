//! O14: a block without a component draws the value, never an empty text (`FieldSlot::compact_view` / `display_view`).
//! The element is laid out and measured: the width of the drawn text grows with the value (an empty text has no width).
//!
//! Pattern: none; tests. Layout only, no graphics adapter.

#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use blinc_layout::div::div;
use blinc_layout::prelude::ElementBuilder;
use common::{layout, serial};
use fs_components_fields::{FieldSlot, KeysAsText, Look};
use fs_components_styles::StyleId;

fn width_of(ui: Box<dyn ElementBuilder>) -> f32 {
    let tree = layout(
        div().id("slot-host").flex_row().w_fit().child(ui),
        600.0,
        200.0,
    );
    let node = tree.query_by_id("slot-host").expect("host");
    tree.get_absolute_bounds(node).expect("bounds").width
}

fn compact(value: &str) -> f32 {
    let look = Look::new(StyleId::Admin, false);
    width_of(FieldSlot::new("field-integer", "l", value).compact_view(&look))
}

fn display(value: &str) -> f32 {
    let look = Look::new(StyleId::Admin, false);
    width_of(FieldSlot::new("field-integer", "l", value).display_view(&look, &KeysAsText))
}

#[test]
fn a_compact_view_without_a_component_draws_the_value() {
    let _g = serial();
    let (short, long) = (compact("7"), compact("seven hundred and seven"));
    assert!(
        short > 1.0,
        "the value is drawn, not an empty text: {short}"
    );
    assert!(long > short + 20.0, "{long} > {short}");
}

#[test]
fn a_display_view_without_a_component_draws_the_value() {
    let _g = serial();
    let (short, long) = (display("7"), display("seven hundred and seven"));
    assert!(short > 1.0, "{short}");
    assert!(long > short + 20.0, "{long} > {short}");
}

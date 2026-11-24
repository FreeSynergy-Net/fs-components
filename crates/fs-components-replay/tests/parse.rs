//! Tests of the reading: a valid Perform Actions body becomes a plan; what is not played is an error with a code, never skipped.
//!
//! Pattern: none (tests). No graphics adapter needed.

use fs_components_replay::{Button, Code, Key, Named, Origin, Plan, Step, parse};

/// The shape of the example in the W3C text of Perform Actions: a mouse source and a key source side by side.
const SPEC_BODY: &str = r#"{"actions":[
 {"type":"pointer","id":"mouse1","parameters":{"pointerType":"mouse"},"actions":[
   {"type":"pointerMove","duration":0,"x":10,"y":20,"origin":"viewport"},
   {"type":"pointerDown","button":0},
   {"type":"pause","duration":100},
   {"type":"pointerUp","button":0}]},
 {"type":"key","id":"kbd","actions":[
   {"type":"pause"},{"type":"pause"},{"type":"keyDown","value":"a"},{"type":"pause"}]}
]}"#;

fn code_of(body: &str) -> Code {
    parse(body).expect_err("an error").code
}

fn pointer_body(action: &str) -> String {
    format!(
        r#"{{"actions":[{{"type":"pointer","id":"m","parameters":{{"pointerType":"mouse"}},"actions":[{action}]}}]}}"#
    )
}

fn key_body(action: &str) -> String {
    format!(r#"{{"actions":[{{"type":"key","id":"k","actions":[{action}]}}]}}"#)
}

#[test]
fn the_spec_body_becomes_ticks_with_one_step_per_source() {
    let plan = parse(SPEC_BODY).expect("valid");
    assert_eq!(plan.ticks.len(), 4);
    assert_eq!(
        plan.ticks[0].steps,
        [
            Step::Move {
                origin: Origin::Viewport,
                x: 10.0,
                y: 20.0,
                duration_ms: 0
            },
            Step::Pause
        ]
    );
    assert_eq!(
        plan.ticks[1].steps[0],
        Step::Button {
            button: Button::Left,
            down: true
        }
    );
    assert_eq!(plan.ticks[1].steps[1], Step::Pause);
    // the longest action of a tick is its length
    assert_eq!(plan.ticks[2].duration_ms, 100);
    assert_eq!(
        plan.ticks[2].steps,
        [
            Step::Pause,
            Step::Key {
                key: Key::Char('a'),
                down: true
            }
        ]
    );
    assert_eq!(
        plan.ticks[3].steps,
        [
            Step::Button {
                button: Button::Left,
                down: false
            },
            Step::Pause
        ]
    );
}

#[test]
fn an_empty_list_is_a_valid_plan_that_does_nothing() {
    assert_eq!(parse(r#"{"actions":[]}"#), Ok(Plan::default()));
}

#[test]
fn an_element_origin_carries_the_id_of_our_ui() {
    let body = pointer_body(
        r#"{"type":"pointerMove","origin":{"element-6066-11e4-a52e-4f735466cecf":"kb-card-a"},"x":0,"y":-20,"duration":300}"#,
    );
    let plan = parse(&body).expect("valid");
    assert_eq!(
        plan.ticks[0].steps[0],
        Step::Move {
            origin: Origin::Element("kb-card-a".into()),
            x: 0.0,
            y: -20.0,
            duration_ms: 300
        }
    );
    assert_eq!(plan.ticks[0].duration_ms, 300);
}

#[test]
fn special_keys_map_to_named_keys() {
    let plan = parse(&key_body(
        r#"{"type":"keyDown","value":""},{"type":"keyDown","value":""},{"type":"keyUp","value":""}"#,
    ))
    .expect("valid");
    assert_eq!(
        plan.ticks[0].steps,
        [Step::Key {
            key: Key::Named(Named::Enter),
            down: true
        }]
    );
    assert_eq!(
        plan.ticks[2].steps,
        [Step::Key {
            key: Key::Named(Named::Shift),
            down: false
        }]
    );
}

#[test]
fn a_wheel_scroll_is_read_and_a_pointer_origin_on_it_is_refused() {
    let wheel = |origin: &str| {
        format!(
            r#"{{"actions":[{{"type":"wheel","id":"w","actions":[{{"type":"scroll","x":5,"y":6,"deltaX":0,"deltaY":120,"origin":{origin}}}]}}]}}"#
        )
    };
    let plan = parse(&wheel(r#""viewport""#)).expect("valid");
    assert_eq!(
        plan.ticks[0].steps[0],
        Step::Wheel {
            origin: Origin::Viewport,
            x: 5.0,
            y: 6.0,
            dx: 0.0,
            dy: 120.0,
            duration_ms: 0
        }
    );
    assert_eq!(code_of(&wheel(r#""pointer""#)), Code::Unsupported);
}

#[test]
fn an_unknown_action_is_an_invalid_body() {
    assert_eq!(
        code_of(&pointer_body(r#"{"type":"pointerTeleport"}"#)),
        Code::Invalid
    );
}

#[test]
fn pointer_cancel_pen_touch_and_other_buttons_are_unsupported_with_a_name() {
    let e = parse(&pointer_body(r#"{"type":"pointerCancel"}"#)).expect_err("error");
    assert_eq!(
        (e.code, e.detail.as_str()),
        (Code::Unsupported, "pointerCancel")
    );
    let touch = r#"{"actions":[{"type":"pointer","id":"t","parameters":{"pointerType":"touch"},"actions":[]}]}"#;
    assert_eq!(code_of(touch), Code::Unsupported);
    assert_eq!(
        code_of(&pointer_body(r#"{"type":"pointerDown","button":3}"#)),
        Code::Unsupported
    );
}

#[test]
fn keys_outside_the_table_are_refused() {
    // F1 (U+E031) is special but not in our table; two characters are no key
    assert_eq!(
        code_of(&key_body(r#"{"type":"keyDown","value":""}"#)),
        Code::Key
    );
    assert_eq!(
        code_of(&key_body(r#"{"type":"keyDown","value":"ab"}"#)),
        Code::Invalid
    );
}

#[test]
fn a_key_up_is_read_and_the_driver_decides() {
    // reading keeps it; `perform` refuses it for a driver without key release (see perform.rs)
    let plan = parse(&key_body(r#"{"type":"keyUp","value":"a"}"#)).expect("valid");
    assert_eq!(
        plan.ticks[0].steps,
        [Step::Key {
            key: Key::Char('a'),
            down: false
        }]
    );
}

#[test]
fn bad_bodies_and_absurd_durations_are_invalid() {
    assert_eq!(code_of("not json"), Code::Invalid);
    assert_eq!(code_of("{}"), Code::Invalid);
    assert_eq!(code_of(r#"{"actions":null}"#), Code::Invalid);
    assert_eq!(
        code_of(&pointer_body(r#"{"type":"pause","duration":3600000}"#)),
        Code::Invalid
    );
    let scroll =
        r#"{"actions":[{"type":"wheel","id":"w","actions":[{"type":"scroll","x":1,"y":1}]}]}"#;
    assert_eq!(code_of(scroll), Code::Invalid);
}

#[test]
fn the_codes_have_stable_texts() {
    assert_eq!(Code::KeyUp.as_str(), "steps-key-up-unsupported");
    assert_eq!(Code::ElementMissing.as_str(), "steps-element-missing");
}

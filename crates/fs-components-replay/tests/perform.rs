//! Tests of the playing: ticks in order, frames of a move, origins, and a plan the driver cannot do is refused before it starts.
//!
//! Pattern: none (tests). A fake driver writes down what it was told; no graphics adapter needed.
#![allow(clippy::float_cmp)]

use std::path::Path;

use fs_components_replay::{Button, Caps, Code, Driver, Key, ReplayError, Step, parse, perform};

#[derive(Default)]
struct Fake {
    caps: Option<Caps>,
    at: (f32, f32),
    log: Vec<String>,
}

impl Driver for Fake {
    fn caps(&self) -> Caps {
        self.caps.unwrap_or(Caps::ALL)
    }
    fn pointer(&self) -> (f32, f32) {
        self.at
    }
    fn centre(&self, id: &str) -> Result<(f32, f32), ReplayError> {
        match id {
            "card" => Ok((100.0, 50.0)),
            _ => Err(ReplayError::new(
                Code::ElementMissing,
                format!("no element with id {id}"),
            )),
        }
    }
    fn move_to(&mut self, x: f32, y: f32) -> Result<(), ReplayError> {
        self.at = (x, y);
        self.log.push(format!("move {x} {y}"));
        Ok(())
    }
    fn button(&mut self, button: Button, down: bool) -> Result<(), ReplayError> {
        self.log.push(format!("button {button:?} {down}"));
        Ok(())
    }
    fn key_down(&mut self, key: &Key) -> Result<(), ReplayError> {
        self.log.push(format!("down {key:?}"));
        Ok(())
    }
    fn key_up(&mut self, key: &Key) -> Result<(), ReplayError> {
        self.log.push(format!("up {key:?}"));
        Ok(())
    }
    fn wheel(&mut self, dx: f32, dy: f32) -> Result<(), ReplayError> {
        self.log.push(format!("wheel {dx} {dy}"));
        Ok(())
    }
    fn advance(&mut self, ms: u32) {
        self.log.push(format!("advance {ms}"));
    }
    fn snapshot(&mut self, _path: &Path) -> Result<(), ReplayError> {
        Ok(())
    }
}

fn pointer(actions: &str) -> String {
    format!(
        r#"{{"actions":[{{"type":"pointer","id":"m","parameters":{{"pointerType":"mouse"}},"actions":[{actions}]}}]}}"#
    )
}

fn play(body: &str, d: &mut Fake) -> Result<(), ReplayError> {
    perform(&parse(body).expect("valid body"), d)
}

#[test]
fn a_press_move_release_runs_in_order_with_frames_on_the_way() {
    let mut d = Fake::default();
    let body = pointer(
        r#"{"type":"pointerMove","x":0,"y":0},{"type":"pointerDown","button":0},
           {"type":"pointerMove","x":64,"y":0,"duration":64},{"type":"pointerUp","button":0},{"type":"pause","duration":200}"#,
    );
    play(&body, &mut d).expect("played");
    let moved = [
        "move 16 0",
        "advance 16",
        "move 32 0",
        "advance 16",
        "move 48 0",
        "advance 16",
        "move 64 0",
        "advance 16",
    ];
    let mut want = vec!["move 0 0", "button Left true"];
    want.extend(moved);
    want.extend(["button Left false", "advance 200"]);
    assert_eq!(d.log, want);
}

#[test]
fn an_element_origin_counts_from_its_centre_and_a_pointer_origin_from_the_pointer() {
    let mut d = Fake::default();
    let body = pointer(
        r#"{"type":"pointerMove","origin":{"element-6066-11e4-a52e-4f735466cecf":"card"},"x":0,"y":-20},
           {"type":"pointerMove","origin":"pointer","x":5,"y":5}"#,
    );
    play(&body, &mut d).expect("played");
    assert_eq!(d.log, ["move 100 30", "move 105 35"]);
}

#[test]
fn a_missing_element_is_an_error_with_its_name_not_a_panic() {
    let mut d = Fake::default();
    let body = pointer(
        r#"{"type":"pointerMove","origin":{"element-6066-11e4-a52e-4f735466cecf":"nope"},"x":0,"y":0}"#,
    );
    let e = play(&body, &mut d).expect_err("error");
    assert_eq!(e.code, Code::ElementMissing);
    assert!(e.detail.contains("nope"));
}

#[test]
fn a_plan_the_driver_cannot_do_is_refused_before_the_first_step() {
    let no_key_up = Caps {
        key_up: false,
        ..Caps::ALL
    };
    let mut d = Fake {
        caps: Some(no_key_up),
        ..Fake::default()
    };
    let body = r#"{"actions":[{"type":"key","id":"k","actions":[{"type":"keyDown","value":"a"},{"type":"keyUp","value":"a"}]}]}"#;
    let e = play(body, &mut d).expect_err("error");
    assert_eq!(e.code, Code::KeyUp);
    assert!(d.log.is_empty(), "nothing was played: {:?}", d.log);

    // Shift is released by the driver itself, so it passes
    let shift = r#"{"actions":[{"type":"key","id":"k","actions":[{"type":"keyDown","value":""},{"type":"keyUp","value":""}]}]}"#;
    play(shift, &mut d).expect("shift is fine");

    let ctrl =
        r#"{"actions":[{"type":"key","id":"k","actions":[{"type":"keyDown","value":""}]}]}"#;
    let mut d = Fake {
        caps: Some(Caps {
            modifiers: false,
            ..Caps::ALL
        }),
        ..Fake::default()
    };
    assert_eq!(play(ctrl, &mut d).expect_err("error").code, Code::Modifier);

    let right = pointer(r#"{"type":"pointerDown","button":2}"#);
    let mut d = Fake {
        caps: Some(Caps {
            extra_buttons: false,
            ..Caps::ALL
        }),
        ..Fake::default()
    };
    assert_eq!(
        play(&right, &mut d).expect_err("error").code,
        Code::Unsupported
    );
}

#[test]
fn a_wheel_moves_to_its_point_and_scrolls_then_waits() {
    let mut d = Fake::default();
    let body = r#"{"actions":[{"type":"wheel","id":"w","actions":[{"type":"scroll","x":7,"y":8,"deltaX":0,"deltaY":-40,"duration":50}]}]}"#;
    play(body, &mut d).expect("played");
    assert_eq!(d.log, ["move 7 8", "wheel 0 -40", "advance 50"]);
}

#[test]
fn an_empty_plan_plays_nothing() {
    let mut d = Fake::default();
    play(r#"{"actions":[]}"#, &mut d).expect("played");
    assert_eq!(d.log.len(), 0);
    let _ = Step::Pause; // the step type is public for drivers and tools
}

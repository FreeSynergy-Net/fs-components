//! Move a card of the kanban board by a described input (W3C Perform Actions), on a real offscreen UI.
//!
//! Pattern: none (tests). Like `cn_tabs_reorder` of `blinc_recorder`, but the input is the JSON body, played through `Driver`.
//! Needs a graphics adapter (software Vulkan on the runner) and the feature `headless`; without an adapter the tests say so and
//! return. The board has no mouse drag: a card moves by selecting it, the grab button, then a click on the target column (the "drag-free move").
#![cfg(feature = "headless")]
#![allow(clippy::pedantic)]

#[path = "../../fs-components-kanban/tests/support/mod.rs"]
mod host;

use blinc_recorder::Headless;
use blinc_theme::ColorScheme;
use fs_components_replay::{Code, HeadlessDriver, parse, perform};
use fs_components_styles::StyleId;
use host::{H, W, board, cards_in, host, last};

const ELEMENT: &str = "element-6066-11e4-a52e-4f735466cecf";

/// Blinc keeps theme and reactive state in process-wide singletons: one surface at a time (nextest runs a test per process).
static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn surface() -> Option<Headless> {
    if !Headless::available() {
        eprintln!("no GPU adapter: skipped");
        return None;
    }
    fs_components_widgets::live::set_min_gap_ms(0);
    let mut ui = Headless::new(W as u32, H as u32).ok()?;
    ui.theme(
        blinc_cn::cn_bundle().with_css(fs_components_widgets::styles()),
        ColorScheme::Light,
    );
    Some(ui)
}

/// A click on the centre of an element: move, press, release, let the clock run.
fn click(id: &str) -> String {
    format!(
        r#"{{"type":"pointerMove","origin":{{"{ELEMENT}":"{id}"}},"x":0,"y":0}},{{"type":"pointerDown","button":0}},{{"type":"pointerUp","button":0}},{{"type":"pause","duration":100}}"#
    )
}

fn body(actions: &str) -> String {
    format!(
        r#"{{"actions":[{{"type":"pointer","id":"mouse","parameters":{{"pointerType":"mouse"}},"actions":[{actions}]}}]}}"#
    )
}

#[test]
fn a_described_input_moves_a_card_to_another_column() {
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(mut ui) = surface() else { return };
    let h = host(&fs_components_kanban::Operation::ALL);
    let b = board(&h, StyleId::Apple);
    ui.mount(move |_| b.build(W, H));
    ui.settle();

    let plan = parse(&body(&format!(
        "{},{},{}",
        click("kb-card-a"),
        click("kb-grab-a"),
        click("kb-col-doing")
    )))
    .expect("valid");
    let mut driver = HeadlessDriver::new(&mut ui);
    perform(&plan, &mut driver).expect("played");
    ui.settle();

    assert_eq!(
        cards_in(&h, "doing"),
        ["a"],
        "the card did not move: {}",
        last(&h)
    );
    assert_eq!(last(&h), "move a todo doing");
}

#[test]
fn a_missing_element_and_a_key_release_are_errors_not_panics() {
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(mut ui) = surface() else { return };
    let h = host(&fs_components_kanban::Operation::ALL);
    let b = board(&h, StyleId::Apple);
    ui.mount(move |_| b.build(W, H));
    ui.settle();
    let mut driver = HeadlessDriver::new(&mut ui);

    let missing = parse(&body(&click("kb-card-nope"))).expect("valid");
    let e = perform(&missing, &mut driver).expect_err("error");
    assert_eq!(e.code, Code::ElementMissing);
    assert!(e.detail.contains("kb-card-nope"));

    let up = r#"{"actions":[{"type":"key","id":"k","actions":[{"type":"keyDown","value":"a"},{"type":"keyUp","value":"a"}]}]}"#;
    let e = perform(&parse(up).expect("valid"), &mut driver).expect_err("error");
    assert_eq!(e.code, Code::KeyUp);
    assert_eq!(last(&h), "", "nothing was played before the refusal");
}

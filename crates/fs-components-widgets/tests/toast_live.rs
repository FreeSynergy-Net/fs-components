//! `toast`, the part WAI-ARIA APG and WCAG ask for: live attributes, times as a setting, a pause while the keyboard is on a toast.
//!
//! Tests: attribute reads on the built stack, model tests with explicit milliseconds, a focus event on the close mark.

mod common;

use common::{Input, attr, layout, serial};
use fs_components_widgets::states::Severity;
use fs_components_widgets::toast::{Labels, Times, Toast, Toasts, now_ms};

fn stack() -> Toasts {
    Toasts::new(&Labels::default())
}

#[test]
fn every_toast_is_a_live_region_that_reads_whole() {
    let _g = serial();
    let t = stack();
    let a = t.push_at(Toast::new(Severity::Info, "Saved"), 0);
    let b = t.push_at(Toast::new(Severity::Warn, "Low disk"), 0);
    let tree = layout(t.build(), 420.0, 300.0);
    let id = |n: u64| format!("toasts-{n}");
    assert_eq!(attr(&tree, &id(a), "role").as_deref(), Some("status"));
    assert_eq!(attr(&tree, &id(a), "aria-live").as_deref(), Some("polite"));
    assert_eq!(attr(&tree, &id(b), "role").as_deref(), Some("alert"));
    assert_eq!(
        attr(&tree, &id(b), "aria-live").as_deref(),
        Some("assertive")
    );
    assert_eq!(attr(&tree, &id(a), "aria-atomic").as_deref(), Some("true"));
}

#[test]
fn the_time_of_each_kind_is_a_setting() {
    let _g = serial();
    let t = stack().times(Times {
        info: Some(100),
        error: Some(200),
        ..Times::default()
    });
    t.push_at(Toast::new(Severity::Info, "a"), 0);
    t.push_at(Toast::new(Severity::Error, "b"), 0);
    let own = t.push_at(Toast::new(Severity::Info, "c").lasts(10_000), 0);
    assert_eq!(t.tick(100), 1, "info goes after the set 100 ms");
    assert_eq!(
        t.tick(200),
        1,
        "an error may go too when the setting says so"
    );
    assert_eq!(t.tick(9_999), 0, "its own time wins over the setting");
    assert!(t.dismiss(own));
    let stay = stack().times(Times::STAY);
    stay.push_at(Toast::new(Severity::Ok, "d"), 0);
    assert_eq!(stay.tick(1_000_000_000), 0, "nothing goes by itself");
}

#[test]
fn the_keyboard_on_a_toast_holds_its_time_and_leaving_releases_it() {
    let _g = serial();
    let t = stack();
    let id = t.push(Toast::new(Severity::Info, "a").lasts(10));
    let mut tree = layout(t.build(), 420.0, 200.0);
    let mut input = Input::new();
    let close = format!("toasts-{id}-close");
    input.focus_events(&mut tree, &close, true);
    assert_eq!(
        t.tick(now_ms() + 10_000),
        0,
        "held while the keyboard is on it"
    );
    input.focus_events(&mut tree, &close, false);
    assert_eq!(t.tick(now_ms() + 20_000), 1, "released");
}

//! `enable_motion_promotion` is the one place that sets the variable, and only when the person has not.
//!
//! The first test changes the environment of the test program; the second touches no environment.

use fs_components_app::{MOTION_PROMOTION, enable_motion_promotion};

#[test]
fn the_call_sets_the_variable_only_when_it_is_unset() {
    // SAFETY: the other test of this file reads source text only; no thread of ours reads the environment now.
    unsafe { std::env::remove_var(MOTION_PROMOTION) };
    // SAFETY: as above.
    unsafe { enable_motion_promotion() };
    assert_eq!(std::env::var(MOTION_PROMOTION).as_deref(), Ok("1"));

    // SAFETY: as above.
    unsafe { std::env::set_var(MOTION_PROMOTION, "0") };
    // SAFETY: as above.
    unsafe { enable_motion_promotion() };
    assert_eq!(
        std::env::var(MOTION_PROMOTION).as_deref(),
        Ok("0"),
        "a value the person set stays"
    );
    // SAFETY: as above.
    unsafe { std::env::remove_var(MOTION_PROMOTION) };
}

/// `launch` and the reader's `launch` set nothing: the only `set_var` of the two window files is the one in `enable_motion_promotion`.
#[test]
fn only_the_enable_function_sets_the_variable() {
    let window = include_str!("../src/window.rs");
    assert_eq!(
        window.matches("set_var").count(),
        1,
        "one set_var in window.rs"
    );
    let at = window.find("set_var").expect("set_var");
    let before = &window[..at];
    let function = before
        .rfind("pub unsafe fn enable_motion_promotion")
        .expect("in the function");
    assert!(
        !before[function..].contains("pub fn launch"),
        "not inside launch"
    );
    let launch = &window[window.find("pub fn launch").expect("launch")..];
    assert!(!launch[..launch.find("\n}\n").expect("end")].contains("set_var"));
    let reader = include_str!("../../fs-components-workbenches/src/shell/windowed.rs");
    assert!(
        !reader.contains("set_var"),
        "the reader's launch sets nothing"
    );
}

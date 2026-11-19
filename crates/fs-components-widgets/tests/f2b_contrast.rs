//! Contrast of the colour pairs the checklist sheet names, in EVERY style, light and dark, read from the sheet
//! (`common/sheet_contrast.rs`), with the intended token of every rule pinned and a counter-test.
//!
//! Tests: model tests over the colours of the theme of each style; text 4.5:1, an edge or ring 3:1 (WCAG 1.4.11).

mod common;
#[path = "common/sheet_contrast.rs"]
mod sheet_contrast;
#[path = "common/styled.rs"]
mod styled;

use blinc_theme::ColorToken as T;
use common::serial;
use fs_components_widgets::checklist;
use sheet_contrast::{Ground, MARK, Pair, TEXT, measure, token_of};

#[test]
fn the_checklist_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 7] = [
        (
            (".fs-check-text", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-check-text", "color"),
            Ground::RuleOver(".fs-check-item:hover", "background"),
            TEXT,
        ),
        (
            (".fs-check-box", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-check-item-done .fs-check-text", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-check-remove-mark", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-check-item-current", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-checklist:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
    ];
    let bad = measure(checklist::STYLES, &pairs);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_sheet_uses_the_intended_tokens() {
    let t = |rule: &str, prop: &str| token_of(checklist::STYLES, rule, prop);
    assert_eq!(t(".fs-check-text", "color"), T::TextPrimary);
    assert_eq!(t(".fs-check-box", "color"), T::TextPrimary);
    assert_eq!(
        t(".fs-check-item-done .fs-check-text", "color"),
        T::TextSecondary
    );
    assert_eq!(t(".fs-check-remove-mark", "color"), T::TextSecondary);
    assert_eq!(t(".fs-check-item:hover", "background"), T::AccentSubtle);
    assert_eq!(t(".fs-check-remove:hover", "background"), T::AccentSubtle);
    assert_eq!(t(".fs-check-item-current", "border-color"), T::BorderFocus);
    assert_eq!(t(".fs-checklist:focus", "outline-color"), T::BorderFocus);
}

#[test]
fn the_measure_can_fail() {
    let _g = serial();
    let bad = measure(
        checklist::STYLES,
        &[(
            (".fs-check-remove-mark", "color"),
            Ground::Token("background"),
            20.0,
        )],
    );
    assert!(!bad.is_empty(), "the measure cannot fail");
}

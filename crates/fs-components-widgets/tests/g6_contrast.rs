//! Contrast of the colour pairs the G6 sheets name (carousel, chip), in EVERY style, light and dark, read from the sheets
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
use fs_components_widgets::{carousel, chip};
use sheet_contrast::{Ground, MARK, Pair, TEXT, measure, token_of};

#[test]
fn the_carousel_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 6] = [
        (
            (".fs-carousel-button-text", "color"),
            Ground::Rule(".fs-carousel-button"),
            TEXT,
        ),
        (
            (".fs-carousel-button-text", "color"),
            Ground::RuleOver(".fs-carousel-button:hover", "surface"),
            TEXT,
        ),
        (
            (".fs-carousel-position", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-carousel-button:focus", "border-color"),
            Ground::Rule(".fs-carousel-button"),
            MARK,
        ),
        (
            (".fs-carousel:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-carousel-slide", "background"),
            Ground::Token("background"),
            1.0,
        ),
    ];
    let bad = measure(carousel::STYLES, &pairs);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_chip_row_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 5] = [
        ((".fs-chip-label", "color"), Ground::Rule(".fs-chip"), TEXT),
        (
            (".fs-chip-remove-mark", "color"),
            Ground::Rule(".fs-chip"),
            TEXT,
        ),
        (
            (".fs-chip-current", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-chips:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-chip-label", "color"),
            Ground::RuleOver(".fs-chip-remove:hover", "accent-subtle"),
            1.0,
        ),
    ];
    let bad = measure(chip::STYLES, &pairs);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_sheets_use_the_intended_tokens() {
    let t = |sheet: &str, rule: &str, prop: &str| token_of(sheet, rule, prop);
    let cr = carousel::STYLES;
    assert_eq!(t(cr, ".fs-carousel-button-text", "color"), T::TextPrimary);
    assert_eq!(t(cr, ".fs-carousel-button", "background"), T::Surface);
    assert_eq!(
        t(cr, ".fs-carousel-button:hover", "background"),
        T::AccentSubtle
    );
    assert_eq!(
        t(cr, ".fs-carousel-button:focus", "border-color"),
        T::BorderFocus
    );
    assert_eq!(t(cr, ".fs-carousel-position", "color"), T::TextSecondary);
    assert_eq!(t(cr, ".fs-carousel:focus", "outline-color"), T::BorderFocus);
    let ch = chip::STYLES;
    assert_eq!(t(ch, ".fs-chip-label", "color"), T::TextPrimary);
    assert_eq!(t(ch, ".fs-chip", "background"), T::AccentSubtle);
    assert_eq!(t(ch, ".fs-chip-remove-mark", "color"), T::TextSecondary);
    assert_eq!(t(ch, ".fs-chip-current", "border-color"), T::BorderFocus);
    assert_eq!(t(ch, ".fs-chips:focus", "outline-color"), T::BorderFocus);
}

#[test]
fn the_measure_can_fail() {
    let _g = serial();
    let bad = measure(
        chip::STYLES,
        &[(
            (".fs-chip-remove-mark", "color"),
            Ground::Rule(".fs-chip"),
            20.0,
        )],
    );
    assert!(!bad.is_empty(), "the measure cannot fail");
}

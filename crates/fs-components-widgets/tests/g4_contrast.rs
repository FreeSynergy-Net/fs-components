//! Contrast of the colour pairs the G4 sheets name (slider and range, meter, progress, splitter), in EVERY style of the set, light and
//! dark. The pairs are read from the sheets themselves (`common/sheet_contrast.rs`), so a changed sheet is measured, and the intended
//! token of every rule is pinned.
//!
//! Tests: model tests over the colours of the theme of each style (`fs_components_styles::Rgba::contrast`), no drawing; text needs
//! 4.5:1, a mark that carries a value (the fill, the thumb's edge, a ring) 3:1 (WCAG 1.4.11). The track is decoration (its colour
//! is only measured against the fill so that the fill can be told from it).

mod common;
#[path = "common/sheet_contrast.rs"]
mod sheet_contrast;
#[path = "common/styled.rs"]
mod styled;

use blinc_theme::ColorToken as T;
use common::serial;
use fs_components_widgets::{meter, progress, slider, splitter};
use sheet_contrast::{Ground, MARK, Pair, TEXT, measure, token_of};

fn report(name: &str, sheet: &str, pairs: &[Pair]) {
    let bad = measure(sheet, pairs);
    assert!(bad.is_empty(), "{name}:\n{}", bad.join("\n"));
}

#[test]
fn the_slider_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 6] = [
        (
            (".fs-slider-fill", "background"),
            Ground::Rule(".fs-slider-rail"),
            MARK,
        ),
        (
            (".fs-slider-thumb", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-slider-thumb-focus", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-slider-keys:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-slider-value", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-slider-thumb", "background"),
            Ground::Token("background"),
            1.0,
        ),
    ];
    report("slider", slider::STYLES, &pairs);
}

#[test]
fn the_meter_reads_in_every_style() {
    let _g = serial();
    // the zone colours alone are not 3:1 against the track in every style (green on the Apple tint is 2.0:1), so the length of the
    // fill is carried by its edge (the text colour); the colour is a second cue next to the word
    let pairs: [Pair; 3] = [
        (
            (".fs-meter-text", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-meter-fill", "border-color"),
            Ground::Rule(".fs-meter-track"),
            MARK,
        ),
        (
            (".fs-meter-fill", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
    ];
    report("meter", meter::STYLES, &pairs);
}

#[test]
fn progress_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 4] = [
        (
            (".fs-progress-text", "color"),
            Ground::Token("background"),
            TEXT,
        ),
        (
            (".fs-progress-fill", "background"),
            Ground::Rule(".fs-progress-track"),
            MARK,
        ),
        (
            (".fs-progress-unknown .fs-progress-fill", "border-color"),
            Ground::Rule(".fs-progress-track"),
            MARK,
        ),
        (
            (".fs-progress-done .fs-progress-fill", "border-color"),
            Ground::Rule(".fs-progress-track"),
            MARK,
        ),
    ];
    report("progress", progress::STYLES, &pairs);
}

#[test]
fn the_splitter_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 3] = [
        (
            (".fs-splitter-bar:focus", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-splitter-bar:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-splitter-bar", "border-color"),
            Ground::Token("background"),
            1.2,
        ),
    ];
    report("splitter", splitter::STYLES, &pairs);
}

#[test]
fn the_sheets_use_the_intended_tokens() {
    let t = |sheet: &str, rule: &str, prop: &str| token_of(sheet, rule, prop);
    let sl = slider::STYLES;
    assert_eq!(t(sl, ".fs-slider-rail", "background"), T::AccentSubtle);
    assert_eq!(t(sl, ".fs-slider-fill", "background"), T::Accent);
    assert_eq!(t(sl, ".fs-slider-thumb", "border-color"), T::Accent);
    assert_eq!(t(sl, ".fs-slider-thumb", "background"), T::Surface);
    assert_eq!(
        t(sl, ".fs-slider-thumb-focus", "border-color"),
        T::BorderFocus
    );
    assert_eq!(
        t(sl, ".fs-slider-keys:focus", "outline-color"),
        T::BorderFocus
    );
    assert_eq!(t(sl, ".fs-slider-value", "color"), T::TextPrimary);
    let me = meter::STYLES;
    assert_eq!(
        t(me, ".fs-meter-good .fs-meter-fill", "background"),
        T::Success
    );
    assert_eq!(
        t(me, ".fs-meter-fair .fs-meter-fill", "background"),
        T::Warning
    );
    assert_eq!(
        t(me, ".fs-meter-poor .fs-meter-fill", "background"),
        T::Error
    );
    assert_eq!(t(me, ".fs-meter-text", "color"), T::TextPrimary);
    assert_eq!(t(me, ".fs-meter-track", "background"), T::AccentSubtle);
    assert_eq!(t(me, ".fs-meter-fill", "border-color"), T::TextPrimary);
    let pr = progress::STYLES;
    assert_eq!(t(pr, ".fs-progress-fill", "background"), T::Accent);
    assert_eq!(t(pr, ".fs-progress-track", "background"), T::AccentSubtle);
    assert_eq!(
        t(pr, ".fs-progress-unknown .fs-progress-fill", "border-color"),
        T::Accent
    );
    assert_eq!(
        t(pr, ".fs-progress-done .fs-progress-fill", "border-color"),
        T::TextPrimary
    );
    assert_eq!(
        t(pr, ".fs-progress-unknown .fs-progress-fill", "background"),
        T::AccentSubtle
    );
    assert_eq!(
        t(pr, ".fs-progress-done .fs-progress-fill", "background"),
        T::Success
    );
    assert_eq!(t(pr, ".fs-progress-text", "color"), T::TextPrimary);
    let sp = splitter::STYLES;
    assert_eq!(
        t(sp, ".fs-splitter-bar:focus", "border-color"),
        T::BorderFocus
    );
    assert_eq!(t(sp, ".fs-splitter-bar-active", "background"), T::Accent);
    assert_eq!(
        t(sp, ".fs-splitter-bar:hover", "background"),
        T::AccentSubtle
    );
}

#[test]
fn the_measure_can_fail() {
    let _g = serial();
    let bad = measure(
        slider::STYLES,
        &[(
            (".fs-slider-fill", "background"),
            Ground::Rule(".fs-slider-thumb"),
            4.5,
        )],
    );
    assert!(!bad.is_empty(), "the measure cannot fail");
}

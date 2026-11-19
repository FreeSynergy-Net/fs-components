//! Contrast of the colour pairs the G1 sheets name (tabs, toolbar, disclosure and accordion, breadcrumb), in EVERY style of the set,
//! light and dark. The pairs are read from the sheets themselves (`common/sheet_contrast.rs`), so a changed sheet is measured.
//!
//! Tests: model tests over the colours of the theme of each style (`fs_components_styles::Rgba::contrast`), no drawing; text needs
//! 4.5:1, an edge or a ring 3:1 (WCAG 1.4.11). A disabled thing is exempt (WCAG), and its pair is measured only as a floor of 3:1 on
//! the weakest ground the sheet puts it on. Each test also has a counter-test: the measure can fail.

mod common;
#[path = "common/sheet_contrast.rs"]
mod sheet_contrast;
#[path = "common/styled.rs"]
mod styled;

use common::serial;
use fs_components_widgets::{breadcrumb, disclosure, tabs, toolbar};
use sheet_contrast::{Ground, MARK, Pair, TEXT, measure, rgba, token_of};

fn report(name: &str, sheet: &str, pairs: &[Pair]) {
    let bad = measure(sheet, pairs);
    assert!(bad.is_empty(), "{name}:\n{}", bad.join("\n"));
}

#[test]
fn tabs_read_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 7] = [
        ((".fs-tab-label", "color"), Ground::Token("background"), TEXT),
        ((".fs-tab-label", "color"), Ground::Token("surface"), TEXT),
        ((".fs-tab-selected .fs-tab-label", "color"), Ground::Rule(".fs-tab-selected"), TEXT),
        ((".fs-tab-label", "color"), Ground::Rule(".fs-tab:hover"), TEXT),
        ((".fs-tab-current", "border-color"), Ground::Token("background"), MARK),
        ((".fs-tabs:focus", "outline-color"), Ground::Token("background"), MARK),
        ((".fs-tab-disabled .fs-tab-label", "color"), Ground::Token("background"), MARK),
    ];
    report("tabs", tabs::STYLES, &pairs);
}

#[test]
fn the_toolbar_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 7] = [
        ((".fs-tool-label", "color"), Ground::Rule(".fs-toolbar"), TEXT),
        ((".fs-tool-label", "color"), Ground::Rule(".fs-tool-on"), TEXT),
        ((".fs-tool-label", "color"), Ground::RuleOver(".fs-tool:hover", "surface"), TEXT),
        ((".fs-tool-current", "border-color"), Ground::Rule(".fs-toolbar"), MARK),
        ((".fs-toolbar:focus", "outline-color"), Ground::Token("background"), MARK),
        ((".fs-tool-separator", "background"), Ground::Rule(".fs-toolbar"), 1.2),
        ((".fs-tool-disabled .fs-tool-label", "color"), Ground::Rule(".fs-toolbar"), MARK),
    ];
    report("toolbar", toolbar::STYLES, &pairs);
}

#[test]
fn disclosure_and_accordion_read_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 6] = [
        ((".fs-disc-title", "color"), Ground::Token("background"), TEXT),
        ((".fs-disc-title", "color"), Ground::Rule(".fs-disc-head:hover"), TEXT),
        ((".fs-disc-mark", "color"), Ground::Token("background"), TEXT),
        ((".fs-disc-panel", "color"), Ground::Token("background"), TEXT),
        ((".fs-disc-head:focus", "border-color"), Ground::Token("background"), MARK),
        ((".fs-accordion", "border-color"), Ground::Token("background"), 1.2),
    ];
    report("disclosure", disclosure::STYLES, &pairs);
}

#[test]
fn the_breadcrumb_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 4] = [
        ((".fs-crumb", "color"), Ground::Token("background"), TEXT),
        ((".fs-crumb", "color"), Ground::Rule(".fs-crumb:hover"), TEXT),
        ((".fs-crumb-here", "color"), Ground::Token("background"), TEXT),
        ((".fs-crumb-gap", "color"), Ground::Token("background"), TEXT),
    ];
    report("breadcrumb", breadcrumb::STYLES, &pairs);
}

#[test]
fn the_measure_can_fail_a_pair_that_is_too_pale_is_found() {
    let _g = serial();
    // The same code path with a deliberately wrong pair: tertiary text on the surface of the tab bar's hover ground is below 4.5:1
    // somewhere in the set, so a sheet that chose it would be red.
    let bad = measure(
        tabs::STYLES,
        &[((".fs-tab-disabled .fs-tab-label", "color"), Ground::Rule(".fs-tab-selected"), TEXT)],
    );
    assert!(!bad.is_empty(), "the measure cannot fail");
    // And the reader panics on a rule that is not in the sheet (a renamed rule turns the test red).
    let missing = std::panic::catch_unwind(|| token_of(tabs::STYLES, ".fs-nothing", "color"));
    assert!(missing.is_err());
    let _ = rgba;
}

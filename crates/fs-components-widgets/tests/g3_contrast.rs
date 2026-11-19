//! Contrast of the colour pairs the G3 sheets name (listbox, tree, grid and treegrid, table, feed), in EVERY style of the set, light
//! and dark. The pairs are read from the sheets themselves (`common/sheet_contrast.rs`), so a changed sheet is measured.
//!
//! Tests: model tests over the colours of the theme of each style (`fs_components_styles::Rgba::contrast`), no drawing; text needs
//! 4.5:1, an edge or a ring 3:1 (WCAG 1.4.11). A disabled thing is exempt (WCAG) and measured as a floor of 3:1. Each test also has a
//! counter-test: the measure can fail.

mod common;
#[path = "common/sheet_contrast.rs"]
mod sheet_contrast;
#[path = "common/styled.rs"]
mod styled;

use blinc_theme::ColorToken as T;
use common::serial;
use fs_components_widgets::{feed, grid, listbox, table, tree};
use sheet_contrast::{Ground, MARK, Pair, TEXT, measure, token_of};

fn report(name: &str, sheet: &str, pairs: &[Pair]) {
    let bad = measure(sheet, pairs);
    assert!(bad.is_empty(), "{name}:\n{}", bad.join("\n"));
}

#[test]
fn the_listbox_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 7] = [
        (
            (".fs-option-label", "color"),
            Ground::Rule(".fs-listbox"),
            TEXT,
        ),
        (
            (".fs-option-label", "color"),
            Ground::RuleOver(".fs-option-picked", "surface"),
            TEXT,
        ),
        (
            (".fs-option-label", "color"),
            Ground::RuleOver(".fs-option:hover", "surface"),
            TEXT,
        ),
        (
            (".fs-option-current", "border-color"),
            Ground::Rule(".fs-listbox"),
            MARK,
        ),
        (
            (".fs-listbox:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-option-disabled .fs-option-label", "color"),
            Ground::Rule(".fs-listbox"),
            MARK,
        ),
        (
            (".fs-listbox", "border-color"),
            Ground::Token("background"),
            1.2,
        ),
    ];
    report("listbox", listbox::STYLES, &pairs);
}

#[test]
fn the_tree_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 7] = [
        (
            (".fs-treeitem-label", "color"),
            Ground::Rule(".fs-tree-root"),
            TEXT,
        ),
        (
            (".fs-treeitem-mark", "color"),
            Ground::Rule(".fs-tree-root"),
            TEXT,
        ),
        (
            (".fs-treeitem-label", "color"),
            Ground::RuleOver(".fs-treeitem-picked", "surface"),
            TEXT,
        ),
        (
            (".fs-treeitem-label", "color"),
            Ground::RuleOver(".fs-treeitem-row:hover", "surface"),
            TEXT,
        ),
        (
            (".fs-treeitem-current", "border-color"),
            Ground::Rule(".fs-tree-root"),
            MARK,
        ),
        (
            (".fs-tree-root:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-tree-root", "border-color"),
            Ground::Token("background"),
            1.2,
        ),
    ];
    report("tree", tree::STYLES, &pairs);
}

#[test]
fn the_grid_and_the_treegrid_read_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 7] = [
        (
            (".fs-grid-cell-text", "color"),
            Ground::Rule(".fs-grid"),
            TEXT,
        ),
        (
            (".fs-grid-headcell-text", "color"),
            Ground::Rule(".fs-grid"),
            TEXT,
        ),
        (
            (".fs-grid-cell-text", "color"),
            Ground::RuleOver(".fs-grid-row:hover", "surface"),
            TEXT,
        ),
        ((".fs-grid-mark", "color"), Ground::Rule(".fs-grid"), TEXT),
        (
            (".fs-grid-cell-current", "border-color"),
            Ground::Rule(".fs-grid"),
            MARK,
        ),
        (
            (".fs-grid:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-grid-head", "border-bottom-color"),
            Ground::Rule(".fs-grid"),
            1.2,
        ),
    ];
    report("grid", grid::STYLES, &pairs);
}

#[test]
fn the_table_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 5] = [
        (
            (".fs-table-cell-text", "color"),
            Ground::Rule(".fs-table"),
            TEXT,
        ),
        (
            (".fs-table-headcell-text", "color"),
            Ground::Rule(".fs-table"),
            TEXT,
        ),
        (
            (".fs-table-rowhead-text", "color"),
            Ground::Rule(".fs-table"),
            TEXT,
        ),
        (
            (".fs-table-caption", "color"),
            Ground::Rule(".fs-table"),
            TEXT,
        ),
        (
            (".fs-table-cell-text", "color"),
            Ground::RuleOver(".fs-table-row:hover", "surface"),
            TEXT,
        ),
    ];
    report("table", table::STYLES, &pairs);
}

#[test]
fn the_feed_reads_in_every_style() {
    let _g = serial();
    let pairs: [Pair; 5] = [
        (
            (".fs-article-title", "color"),
            Ground::Rule(".fs-article"),
            TEXT,
        ),
        (
            (".fs-article-body", "color"),
            Ground::Rule(".fs-article"),
            TEXT,
        ),
        (
            (".fs-article-current", "border-color"),
            Ground::Rule(".fs-article"),
            MARK,
        ),
        (
            (".fs-article-current", "border-color"),
            Ground::Token("background"),
            MARK,
        ),
        (
            (".fs-feed:focus", "outline-color"),
            Ground::Token("background"),
            MARK,
        ),
    ];
    report("feed", feed::STYLES, &pairs);
}

#[test]
fn the_measure_can_fail_and_a_renamed_rule_turns_it_red() {
    let _g = serial();
    // tertiary text on the picked ground is below 4.5:1 somewhere in the set
    let bad = measure(
        listbox::STYLES,
        &[(
            (".fs-option-disabled .fs-option-label", "color"),
            Ground::RuleOver(".fs-option-picked", "surface"),
            TEXT,
        )],
    );
    assert!(!bad.is_empty(), "the measure cannot fail");
    let missing = std::panic::catch_unwind(|| token_of(tree::STYLES, ".fs-nothing", "color"));
    assert!(missing.is_err());
}

/// The pairs are a contract of the sheets: the contrast test passes for any token that is readable enough, so the intended token of
/// each rule is pinned here (a sheet that moved to a weaker token would pass the measure but break the style's order of weight).
#[test]
fn the_sheets_use_the_intended_tokens() {
    let t = |sheet: &str, rule: &str, prop: &str| token_of(sheet, rule, prop);
    let lb = listbox::STYLES;
    assert_eq!(t(lb, ".fs-option-label", "color"), T::TextPrimary);
    assert_eq!(t(lb, ".fs-listbox", "background"), T::Surface);
    assert_eq!(t(lb, ".fs-option-picked", "background"), T::AccentSubtle);
    assert_eq!(t(lb, ".fs-option:hover", "background"), T::AccentSubtle);
    assert_eq!(t(lb, ".fs-option-current", "border-color"), T::BorderFocus);
    assert_eq!(t(lb, ".fs-listbox:focus", "outline-color"), T::BorderFocus);
    assert_eq!(
        t(lb, ".fs-option-disabled .fs-option-label", "color"),
        T::TextTertiary
    );
    let tr = tree::STYLES;
    assert_eq!(t(tr, ".fs-treeitem-label", "color"), T::TextPrimary);
    assert_eq!(t(tr, ".fs-treeitem-mark", "color"), T::TextSecondary);
    assert_eq!(t(tr, ".fs-treeitem-picked", "background"), T::AccentSubtle);
    assert_eq!(
        t(tr, ".fs-treeitem-current", "border-color"),
        T::BorderFocus
    );
    assert_eq!(
        t(tr, ".fs-tree-root:focus", "outline-color"),
        T::BorderFocus
    );
    assert_eq!(t(tr, ".fs-tree-root", "background"), T::Surface);
    let gr = grid::STYLES;
    assert_eq!(t(gr, ".fs-grid-cell-text", "color"), T::TextPrimary);
    assert_eq!(t(gr, ".fs-grid-headcell-text", "color"), T::TextPrimary);
    assert_eq!(t(gr, ".fs-grid-mark", "color"), T::TextSecondary);
    assert_eq!(t(gr, ".fs-grid-row:hover", "background"), T::AccentSubtle);
    assert_eq!(
        t(gr, ".fs-grid-cell-current", "border-color"),
        T::BorderFocus
    );
    assert_eq!(t(gr, ".fs-grid:focus", "outline-color"), T::BorderFocus);
    assert_eq!(t(gr, ".fs-grid", "background"), T::Surface);
    let tb = table::STYLES;
    assert_eq!(t(tb, ".fs-table-cell-text", "color"), T::TextPrimary);
    assert_eq!(t(tb, ".fs-table-headcell-text", "color"), T::TextPrimary);
    assert_eq!(t(tb, ".fs-table-rowhead-text", "color"), T::TextPrimary);
    assert_eq!(t(tb, ".fs-table-caption", "color"), T::TextSecondary);
    assert_eq!(t(tb, ".fs-table-row:hover", "background"), T::AccentSubtle);
    let fd = feed::STYLES;
    assert_eq!(t(fd, ".fs-article-title", "color"), T::TextPrimary);
    assert_eq!(t(fd, ".fs-article-body", "color"), T::TextPrimary);
    assert_eq!(t(fd, ".fs-article", "background"), T::Surface);
    assert_eq!(t(fd, ".fs-article-current", "border-color"), T::BorderFocus);
    assert_eq!(t(fd, ".fs-feed:focus", "outline-color"), T::BorderFocus);
}

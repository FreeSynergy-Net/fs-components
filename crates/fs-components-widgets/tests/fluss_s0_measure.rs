//! Spike S0 (B) and (C), measuring only (ENTWURF-fluss-panels.md section C, "Messung zuerst").
//! (B) how many rebuilds a second a page of 12 real components survives, headless (no GPU, no event loop): `Mounted::build` for
//! 12 `FieldCard` panels of 6 rows, then the render tree built and laid out, at a sliding width as when the window edge is dragged.
//! (C) a row of `w(x).flex_shrink_0()` keeps its widths to 1 logical px in a container that is too narrow (and, as the mutant,
//! the same row without `flex_shrink_0` does not).
//! Run `--nocapture` for the numbers (B); they are printed, not asserted, except a floor that catches a hang.
//!
//! Pattern: none; tests (measure the laid-out tree, never a guessed number).
#![allow(clippy::pedantic)]

mod common;

use std::sync::Arc;
use std::time::Instant;

use blinc_layout::div::{div, Div};
use blinc_layout::prelude::RenderTree;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::field_card::form_view::View;
use fs_components_words::Locale;

fn input() -> FormInput {
    FormInput {
        form: "rows@1.0.0".to_owned(),
        rows: (0..6)
            .map(|n| {
                FormRow::of(
                    &format!("k{n}"),
                    &[("columns", "Role"), ("label", &format!("Person {n}"))],
                )
            })
            .collect(),
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
    }
}

fn mounted() -> Box<dyn Mounted> {
    let ctx = DrawCtx {
        look: (StyleId::Admin, false),
        say: Arc::new(|k: &str| k.to_owned()),
        locale: Locale::english(),
        write: Arc::new(NoWrite),
        reread: Arc::new(input),
    };
    View.mount(input(), ctx)
}

/// 12 panels in 3 rows of 4 at window width `w`, each built with its own width as the flow would.
fn page(panels: &[Box<dyn Mounted>], w: f32) -> Div {
    let each = ((w - 3.0 * 8.0) / 4.0).floor();
    let mut col = div().w(w).flex_col().gap_px(8.0);
    for row in panels.chunks(4) {
        let mut r = div().flex_row().gap_px(8.0);
        for p in row {
            r = r.child(div().w(each).flex_shrink_0().child(p.build(each, 400.0)));
        }
        col = col.child(r);
    }
    col
}

fn rate(label: &str, n: usize, f: &mut dyn FnMut(usize)) -> f64 {
    for i in 0..5 {
        f(i); // warm up
    }
    let t = Instant::now();
    for i in 0..n {
        f(i);
    }
    let per_s = n as f64 / t.elapsed().as_secs_f64();
    println!(
        "B  {label}: {n} rebuilds in {:?} = {per_s:.0} per second",
        t.elapsed()
    );
    per_s
}

#[test]
fn b_rebuilds_per_second_of_a_page_of_12_panels() {
    let _g = common::serial();
    blinc_layout::init_text_measurer();
    let panels: Vec<_> = (0..12).map(|_| mounted()).collect();
    let width = |i: usize| 900.0 + (i % 400) as f32; // the edge slides one pixel at a time
    let a = rate("build only (12 x Mounted::build)", 300, &mut |i| {
        let _ = page(&panels, width(i));
    });
    let b = rate(
        "build + RenderTree::from_element + compute_layout",
        300,
        &mut |i| {
            let ui = page(&panels, width(i));
            let mut tree = RenderTree::from_element(&ui);
            tree.compute_layout(width(i), 900.0);
        },
    );
    // as a window does: the style sheet is parsed once, every rebuild applies it to the new tree
    let bundle = blinc_cn::cn_bundle().with_css(fs_components_widgets::styles());
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let sheet = blinc_layout::css_parser::Stylesheet::parse_with_variables(
        &bundle.css_sources.join("\n"),
        &vars,
    )
    .expect("sheet");
    let d = rate(
        "build + tree + style sheet applied (parsed once) + layout = what a window does",
        100,
        &mut |i| {
            let ui = page(&panels, width(i));
            let mut tree = RenderTree::from_element(&ui);
            tree.set_stylesheet(sheet.clone());
            tree.apply_stylesheet_layout_overrides();
            tree.apply_stylesheet_base_styles();
            tree.compute_layout(width(i), 900.0);
        },
    );
    let c = rate(
        "build + layout() of the test rig (also parses the style sheet: an upper bound)",
        100,
        &mut |i| {
            let _ = common::layout(page(&panels, width(i)), width(i), 900.0);
        },
    );
    println!("B  summary: build {a:.0}/s, tree+layout {b:.0}/s, window-like (sheet parsed once) {d:.0}/s, sheet parsed each time {c:.0}/s (profile: {})", if cfg!(debug_assertions) { "debug" } else { "release" });
    assert!(c > 0.5, "a page of 12 panels takes seconds to build: hang?");
}

fn row(widths: &[f32], shrink0: bool, container: f32) -> RenderTree {
    let mut r = div().w(container).h(30.0).flex_row();
    for (i, w) in widths.iter().enumerate() {
        let d = div().id(format!("c{i}")).w(*w).h(20.0);
        r = r.child(if shrink0 { d.flex_shrink_0() } else { d });
    }
    common::layout(r, container + 100.0, 100.0)
}

fn widths_of(tree: &RenderTree, n: usize) -> Vec<(f32, f32)> {
    (0..n)
        .map(|i| {
            let node = tree.query_by_id(&format!("c{i}")).expect("element");
            let b = tree.get_absolute_bounds(node).expect("bounds");
            (b.x, b.width)
        })
        .collect()
}

#[test]
fn c_a_row_of_fixed_widths_keeps_them_to_one_pixel() {
    let _g = common::serial();
    let want = [300.0, 250.5, 200.0, 123.25];
    let mut worst = 0.0f32;
    for container in [1000.0f32, 600.0, 320.0] {
        let got = widths_of(&row(&want, true, container), want.len());
        for (g, w) in got.iter().zip(want) {
            worst = worst.max((g.1 - w).abs());
            assert!(
                (g.1 - w).abs() <= 1.0,
                "container {container}: width {} instead of {w}",
                g.1
            );
        }
        for k in 1..got.len() {
            let gap = got[k].0 - (got[k - 1].0 + got[k - 1].1);
            assert!(
                gap.abs() <= 1.0,
                "container {container}: the elements are not side by side (gap {gap})"
            );
        }
    }
    println!("C  worst deviation of a width with flex_shrink_0: {worst:.3} px (limit 1)");
}

#[test]
fn c_mutant_without_flex_shrink_0_does_shrink() {
    let _g = common::serial();
    let want = [300.0, 250.0, 200.0, 123.0];
    let got = widths_of(&row(&want, false, 600.0), want.len());
    println!("C  mutant (no flex_shrink_0, 873 into 600): {got:?}");
    assert!(
        got.iter().zip(want).any(|(g, w)| (g.1 - w).abs() > 1.0),
        "the row did not shrink: the measurement would not see a missing flex_shrink_0"
    );
}

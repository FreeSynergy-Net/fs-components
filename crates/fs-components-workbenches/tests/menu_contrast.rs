//! Finding b3c of the hand test of fs-texts 0.0.13: the entry the block menu marks hardly stood out from the menu (the marked
//! row was 1.12:1 against the ground). The pairs the style sheet of the menu uses are read out of the sheet itself (which
//! theme variable it names for which property), resolved in every style and in the platform default, in both appearances,
//! and measured: text 4.5:1 (WCAG 1.4.3), the edge of the marked entry 3:1 against the ground of the menu (WCAG 1.4.11), and
//! the ground of the menu opaque, so nothing under it shines through.
//!
//! Tests: arithmetic on the tokens (no window, no graphics).

#![allow(clippy::cast_possible_truncation)] // colour arithmetic in a test: 0..1 values, f64 to f32 and back

use blinc_core::Color;
use blinc_theme::{ColorScheme, ColorTokens, Theme, ThemeState};
use fs_components_render::StyleTheme;
use fs_components_styles::{StyleId, style};

/// The value of `property` in the rule of `selector` of the style sheet, as the name of its `var(--…)`.
fn var_of(selector: &str, property: &str) -> String {
    let sheet = fs_components_workbenches::styles();
    let rule = sheet
        .lines()
        .find(|l| l.starts_with(&format!("{selector} {{")))
        .unwrap_or_else(|| panic!("no rule for {selector}"));
    let body = rule.split_once('{').unwrap().1;
    let value = body
        .split(';')
        .filter_map(|d| d.split_once(':'))
        .find(|(p, _)| p.trim() == property)
        .unwrap_or_else(|| panic!("{selector} has no {property}"))
        .1;
    value
        .trim()
        .strip_prefix("var(--")
        .and_then(|v| v.split_once(')'))
        .unwrap_or_else(|| panic!("{selector} {property} is no theme variable: {value}"))
        .0
        .to_owned()
}

fn token(c: &ColorTokens, var: &str) -> Color {
    match var {
        "surface" => c.surface,
        "accent-subtle" => c.accent_subtle,
        "selection" => c.selection,
        "accent" => c.accent,
        "text-primary" => c.text_primary,
        "text-secondary" => c.text_secondary,
        "secondary-active" => c.secondary_active,
        other => panic!("the test does not know --{other}"),
    }
}

/// `c` over the opaque `ground`, as three numbers 0..1.
fn over(c: Color, ground: Color) -> [f64; 3] {
    let a = f64::from(c.a);
    let mix = |t: f32, g: f32| f64::from(t) * a + f64::from(g) * (1.0 - a);
    [mix(c.r, ground.r), mix(c.g, ground.g), mix(c.b, ground.b)]
}

fn luminance(c: [f64; 3]) -> f64 {
    let lin = |v: f64| {
        if v <= 0.039_28 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Every look the menu can have: the platform default and each style, light and dark.
fn looks() -> Vec<(String, ColorTokens)> {
    ThemeState::init_default();
    let mut all = Vec::new();
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        ThemeState::get().set_scheme(scheme);
        all.push((format!("default {scheme:?}"), ThemeState::get().colors()));
    }
    for id in StyleId::ALL {
        for dark in [false, true] {
            let theme = StyleTheme::new(style(id), dark);
            all.push((format!("{} dark={dark}", id.name()), theme.colors().clone()));
        }
    }
    all
}

#[test]
fn the_marked_entry_of_the_block_menu_stands_out_and_stays_readable() {
    let (ground_var, fill_var, hover_var) = (
        var_of(".fs-wk-slash", "background"),
        var_of(".fs-wk-slash-on", "background"),
        var_of(".fs-wk-slash-item:hover", "background"),
    );
    let edge_var = var_of(".fs-wk-slash-on", "border-left-color");
    let text_var = var_of(".fs-wk-slash-item", "color");
    let mark_var = var_of(".fs-wk-slash-mark", "color");
    let mark_on_var = var_of(
        ".fs-wk-slash-on .fs-wk-slash-mark, .fs-wk-slash-item:hover .fs-wk-slash-mark",
        "color",
    );
    for (name, c) in looks() {
        let ground = over(token(&c, &ground_var), Color::WHITE);
        let on = |v: &str, under: [f64; 3]| {
            over(
                token(&c, v),
                Color::rgba(under[0] as f32, under[1] as f32, under[2] as f32, 1.0),
            )
        };
        let (fill, hover) = (on(&fill_var, ground), on(&hover_var, ground));
        let ink = |v: &str| over(token(&c, v), Color::BLACK);
        // Text 4.5:1 (WCAG 1.4.3) on every ground an entry can have; the edge of the marked entry 3:1 against the menu (1.4.11).
        let text_pairs = [
            ("words", &text_var, ground),
            ("words, marked", &text_var, fill),
            ("words, hovered", &text_var, hover),
            ("sign", &mark_var, ground),
            ("sign, hovered", &mark_on_var, hover),
            ("sign, marked", &mark_on_var, fill),
        ];
        for (what, var, under) in text_pairs {
            let ratio = contrast(ink(var), under);
            assert!(
                ratio >= 4.5,
                "{name}: the {what} of a menu entry {ratio:.2}:1"
            );
        }
        let edge = contrast(ink(&edge_var), ground);
        assert!(
            edge >= 3.0,
            "{name}: the edge of the marked entry {edge:.2}:1"
        );
        let tint = contrast(fill, ground);
        assert!(
            tint >= 1.3,
            "{name}: the marked ground differs from the menu by {tint:.2}:1"
        );
    }
}

#[test]
fn the_ground_of_the_block_menu_is_opaque_in_every_look() {
    let ground = var_of(".fs-wk-slash", "background");
    for (name, c) in looks() {
        let a = token(&c, &ground).a;
        assert!(
            (a - 1.0).abs() < f32::EPSILON,
            "{name}: --{ground} has alpha {a}"
        );
    }
}

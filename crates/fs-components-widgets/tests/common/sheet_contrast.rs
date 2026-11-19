//! Test support: contrast of the colour pairs a widget's own style sheet names, in every style, light and dark.
//!
//! Tests: a helper over `fs_components_styles::Rgba::contrast` and the theme of `styled.rs`. It READS the sheet (`STYLES` of the
//! widget) for the token a rule gives a property (`color: var(--text-primary)`), resolves that token in the theme of each style, and
//! measures; so a changed sheet is measured, not an assumption (lesson of the tooltip, G2). A test file includes it with
//! `#[path = "common/sheet_contrast.rs"] mod sheet_contrast;` next to `mod common;` and `mod styled;`.
#![allow(dead_code)]

use blinc_layout::div::div;
use blinc_theme::{ColorToken, ThemeState};
use fs_components_styles::{Rgba, StyleId};

use crate::styled;

/// Contrast of text (WCAG 2.x AA, normal size).
pub const TEXT: f32 = 4.5;
/// Contrast of an edge, a mark or a ring that must be seen (WCAG 1.4.11).
pub const MARK: f32 = 3.0;

/// The colour token behind a `var(--name)` of a sheet.
pub fn token(name: &str) -> ColorToken {
    match name {
        "text-primary" => ColorToken::TextPrimary,
        "text-secondary" => ColorToken::TextSecondary,
        "text-tertiary" => ColorToken::TextTertiary,
        "text-inverse" => ColorToken::TextInverse,
        "text-link" => ColorToken::TextLink,
        "background" => ColorToken::Background,
        "surface" => ColorToken::Surface,
        "surface-elevated" => ColorToken::SurfaceElevated,
        "accent" => ColorToken::Accent,
        "accent-subtle" => ColorToken::AccentSubtle,
        "primary" => ColorToken::Primary,
        "border" => ColorToken::Border,
        "border-focus" => ColorToken::BorderFocus,
        "selection" => ColorToken::Selection,
        "error" => ColorToken::Error,
        "success" => ColorToken::Success,
        "warning" => ColorToken::Warning,
        "info" => ColorToken::Info,
        other => panic!("a token the contrast test does not know: {other}"),
    }
}

/// The colour of a token in the theme that is set now, with its alpha (a hover ground is a tint of the text colour, not a solid).
pub fn rgba(token: ColorToken) -> Rgba {
    let c = ThemeState::get().color(token);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgba {
        r: byte(c.r),
        g: byte(c.g),
        b: byte(c.b),
        a: byte(c.a),
    }
}

/// `top` laid over the solid `ground`: what the eye sees of a translucent tint.
pub fn over(top: Rgba, ground: Rgba) -> Rgba {
    let a = f32::from(top.a) / 255.0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mix = |t: u8, g: u8| (f32::from(t) * a + f32::from(g) * (1.0 - a)).round() as u8;
    Rgba {
        r: mix(top.r, ground.r),
        g: mix(top.g, ground.g),
        b: mix(top.b, ground.b),
        a: 255,
    }
}

/// The token the rule that starts with `selector` (exactly, followed by ` {`) gives `property` (`color`, `background`,
/// `border-color`, `outline-color`). Panics when there is no such rule or property: a renamed rule must turn the test red.
pub fn token_of(sheet: &str, selector: &str, property: &str) -> ColorToken {
    let line = sheet
        .lines()
        .find(|l| l.starts_with(&format!("{selector} {{")))
        .unwrap_or_else(|| panic!("no rule {selector}"));
    let key = format!("{property}: var(--");
    let at = line
        .match_indices(&key)
        .map(|(i, _)| i)
        .find(|&i| i == 0 || !line[..i].ends_with('-'))
        .unwrap_or_else(|| panic!("rule {selector} has no {property}"));
    let after = &line[at + key.len()..];
    token(&after[..after.find(')').expect("a closing bracket")])
}

/// What a pair stands on: a fixed token, or what another rule of the sheet gives as `background`.
#[derive(Clone, Copy)]
pub enum Ground {
    /// The token of the window or the panel.
    Token(&'static str),
    /// The `background` of this rule, laid over the token of the window.
    Rule(&'static str),
    /// The `background` of this rule, laid over this other token (a hover tint on a panel).
    RuleOver(&'static str, &'static str),
}

/// One pair to measure: `(selector, property)` of the foreground, the ground, the least ratio.
pub type Pair = ((&'static str, &'static str), Ground, f32);

/// Run `f` once for every style of the set, light and dark, with the theme of that style set.
pub fn every_style(mut f: impl FnMut(StyleId, bool)) {
    let css = fs_components_widgets::styles();
    for id in StyleId::ALL {
        for dark in [false, true] {
            let _tree = styled::layout_styled(div, 10.0, 10.0, id, dark, &css);
            f(id, dark);
        }
    }
}

/// Measure every pair of `pairs`, read from `sheet`, in every style; the failures come back as lines (empty = all good).
pub fn measure(sheet: &str, pairs: &[Pair]) -> Vec<String> {
    let mut bad = Vec::new();
    every_style(|id, dark| {
        for &((rule, prop), ground, least) in pairs {
            let fg = rgba(token_of(sheet, rule, prop));
            let bg = match ground {
                Ground::Token(name) => over(rgba(token(name)), rgba(ColorToken::Background)),
                Ground::Rule(other) => over(
                    rgba(token_of(sheet, other, "background")),
                    rgba(ColorToken::Background),
                ),
                Ground::RuleOver(other, base) => over(
                    rgba(token_of(sheet, other, "background")),
                    over(rgba(token(base)), rgba(ColorToken::Background)),
                ),
            };
            let fg = over(fg, bg);
            let c = fg.contrast(bg);
            if c < least {
                bad.push(format!(
                    "{} dark={dark}: {rule} {prop} on its ground has {c:.2}, needs {least}",
                    id.name()
                ));
            }
        }
    });
    bad
}

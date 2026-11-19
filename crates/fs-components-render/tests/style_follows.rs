//! The style of a running window follows the settings: `set_style` swaps the Blinc theme and the style in use, `StyleFollows` reads
//! `fs.look.style` and `fs.look.scheme`, and a style that is not installed changes nothing.
//!
//! Tests: one test body, because the theme and the style in use are process wide; the Blinc theme state is read back.

use blinc_theme::{ColorScheme, ColorToken, ThemeState};
use fs_components_render::{StyleFollows, chosen_style, set_style};
use fs_components_styles::{StyleId, current_style};
use fs_settings::Value;

fn primary() -> [u8; 3] {
    let c = ThemeState::get().color(ColorToken::Primary);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(c.r), byte(c.g), byte(c.b)]
}

fn choice(s: &str) -> Value {
    Value::Choice(s.to_owned())
}

#[test]
fn a_running_window_changes_style_and_appearance_by_setting_and_ignores_what_is_not_ours() {
    set_style(StyleId::Admin, false, "");
    let admin = primary();
    assert_eq!(chosen_style(), Some(StyleId::Admin));
    assert_eq!(ThemeState::get().scheme(), ColorScheme::Light);

    set_style(StyleId::Apple, false, "");
    assert_eq!(
        current_style(),
        StyleId::Apple,
        "the widgets read the style in use"
    );
    assert_ne!(
        primary(),
        admin,
        "the theme of Blinc was replaced, not only the value"
    );

    let follows = StyleFollows::new("", false);
    assert!(follows.apply("fs.look.style", &choice("windows")));
    assert_eq!(current_style(), StyleId::Windows);
    let windows = primary();
    assert!(
        !follows.apply("fs.look.style", &choice("no-such-style")),
        "not installed: nothing changes"
    );
    assert_eq!(current_style(), StyleId::Windows);
    assert_eq!(primary(), windows);
    assert!(
        !follows.apply("fs.look.style", &Value::Whole(3)),
        "not a choice"
    );
    assert!(
        !follows.apply("fs.shell.language", &choice("de")),
        "not a key of ours"
    );

    assert!(
        !follows.apply("fs.look.scheme", &choice("darkk")),
        "a typo is no appearance"
    );
    assert_eq!(
        ThemeState::get().scheme(),
        ColorScheme::Light,
        "and changes nothing"
    );
    assert!(follows.apply("fs.look.scheme", &choice("dark")));
    assert_eq!(ThemeState::get().scheme(), ColorScheme::Dark);
    assert_eq!(
        current_style(),
        StyleId::Windows,
        "the appearance keeps the style"
    );
    assert!(follows.apply("fs.look.scheme", &choice("light")));
    assert_eq!(ThemeState::get().scheme(), ColorScheme::Light);

    // Craft is dark by nature: a light appearance does not turn it light
    set_style(StyleId::Craft, false, "");
    assert_eq!(ThemeState::get().scheme(), ColorScheme::Dark);

    set_style(StyleId::Admin, false, "");
    assert_eq!(current_style(), StyleId::Admin);
}

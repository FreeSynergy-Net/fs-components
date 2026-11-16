//! Theme and fonts, the same for a window and a picture.
//!
//! Pattern: Facade over Blinc's theme and font set-up — one function builds the theme bundle (the component library's sheet plus
//! the caller's sheet, which uses theme variables only).

use std::sync::Once;

use blinc_theme::ThemeBundle;

/// The theme bundle: the component library's sheet and `css`, on the platform's theme.
///
/// When the program chose a style with [`crate::use_style`], the bundle is that style's (window and picture alike).
pub fn bundle(css: &str) -> ThemeBundle {
    match crate::chosen_style() {
        Some(id) => crate::style_bundle(id, css),
        None => blinc_cn::cn_bundle()
            .with_css(fs_components_styles::styles::control_tokens::TOKENS.to_owned())
            .with_css(css.to_owned())
            .with_css(fs_components_styles::styles::block_tokens::sheet(
                fs_components_styles::style(fs_components_styles::StyleId::Admin),
            )),
    }
}

/// Fonts for symbols the system may lack: Blinc's bundled emoji subset (the upstream exception of ADR 0089). Once per
/// process: each registration would add the font again.
///
/// No other font is registered: this crate carries no font in its binary (ADR 0089). Craft's mono face is the first installed
/// of `styles::craft::MONO` (`JetBrains Mono` when installed through fs-software, then our own list of mono faces, then the
/// generic `monospace`). With Craft chosen, a missing mono face is reported once per process (here and in [`crate::use_style`]).
pub fn register_fonts() {
    static ONCE: Once = Once::new();
    ONCE.call_once(blinc_noto_emoji::register);
    if crate::chosen_style() == Some(fs_components_styles::StyleId::Craft) {
        crate::mono::warn_when_missing();
    }
}

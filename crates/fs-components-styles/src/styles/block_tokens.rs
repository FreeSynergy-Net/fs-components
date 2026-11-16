//! The tokens of the building blocks of contract version 0.2 (dialogs, toast, alert, skeleton, switch, segmented): one entry per
//! style, written out, none left to a default.
//!
//! Pattern: Registry by exhaustive `match` — [`look`] names every [`StyleId`], so a new style does not compile until it has its
//! entry (the compiler is the completeness check; `tests/block_tokens.rs` also reads the contract and demands every token of
//! every block of 0.2 for every style, light and dark). The tokens are `:root` variables over the theme's own tokens, so colours
//! follow the light and the dark scheme; the numbers are the style's.

use super::block_marks;
use crate::color::Rgba;
use crate::style::{Style, StyleId};

/// What one style says about the blocks of 0.2, beyond the colours of its palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockLook {
    /// Alpha (0 to 255) of the black scrim behind a dialog.
    pub scrim: u8,
    /// Widest a dialog grows.
    pub dialog_width: f32,
    /// Widest a toast grows.
    pub toast_width: f32,
    /// Width of the coloured edge of a toast or an alert (0: none, the glyph and the tint tell the kind).
    pub edge: f32,
    /// Corner of a dialog.
    pub dialog_radius: f32,
    /// Padding inside a dialog.
    pub dialog_pad: f32,
    /// Corner of a message (toast, alert, partial notice).
    pub message_radius: f32,
    /// Vertical padding inside a message.
    pub message_pad: f32,
    /// Percent of the colour of the step (info, ok, warning, error) that tints the ground of a message.
    pub message_tint: u8,
    /// Share of the text colour in the placeholder bars (percent over the surface).
    pub skeleton_percent: u8,
    /// Corner of a placeholder bar.
    pub skeleton_radius: f32,
    /// Height of a bar of text.
    pub skeleton_height: f32,
    /// Length of the track of a switch.
    pub switch_width: f32,
    /// Height of the track of a switch (the thumb is 4 smaller).
    pub switch_height: f32,
}

/// The entry of `id`. Every style has its own; there is no default.
pub const fn look(id: StyleId) -> BlockLook {
    match id {
        // SAP Fiori (Horizon): dense, a clear edge on messages, a 44 x 22 switch.
        StyleId::Admin => BlockLook {
            dialog_radius: 8.0, // OpenUI5 sap_horizon[_dark] base.less:sapPopover_BorderCornerRadius = 0.5rem (8px)
            dialog_pad: 16.0,
            message_radius: 8.0,
            message_pad: 8.0,
            message_tint: 10,
            scrim: 0x66,
            dialog_width: 440.0,
            toast_width: 360.0,
            edge: 4.0,
            skeleton_percent: 14,
            skeleton_radius: 4.0,
            skeleton_height: 12.0,
            switch_width: 44.0,
            switch_height: 22.0,
        },
        // Few large things: big dialogs, a thick edge, a 56 x 30 switch.
        StyleId::Simple => BlockLook {
            dialog_radius: 16.0,
            dialog_pad: 24.0,
            message_radius: 14.0,
            message_pad: 14.0,
            message_tint: 20,
            scrim: 0x8C,
            dialog_width: 520.0,
            toast_width: 420.0,
            edge: 6.0,
            skeleton_percent: 17,
            skeleton_radius: 8.0,
            skeleton_height: 18.0,
            switch_width: 56.0,
            switch_height: 30.0,
        },
        // Apple: a light scrim, no coloured edge (glyph and tint), a 51 x 31 switch.
        StyleId::Apple => BlockLook {
            dialog_radius: 14.0,
            dialog_pad: 20.0,
            message_radius: 12.0,
            message_pad: 10.0,
            message_tint: 6,
            scrim: 0x4D,
            dialog_width: 400.0,
            toast_width: 360.0,
            edge: 0.0,
            skeleton_percent: 12,
            skeleton_radius: 6.0,
            skeleton_height: 14.0,
            switch_width: 51.0,
            switch_height: 31.0,
        },
        // Fluent 2: a light scrim, a hairline edge, a 40 x 20 switch.
        StyleId::Windows => BlockLook {
            dialog_radius: 8.0,
            dialog_pad: 24.0,
            message_radius: 4.0,
            message_pad: 12.0,
            message_tint: 5,
            scrim: 0x4D,
            dialog_width: 440.0,
            toast_width: 360.0,
            edge: 1.0,
            skeleton_percent: 13,
            skeleton_radius: 4.0,
            skeleton_height: 8.0,
            switch_width: 40.0,
            switch_height: 20.0,
        },
        // Craft: a heavy scrim, square, a 36 x 18 switch.
        StyleId::Craft => BlockLook {
            dialog_radius: 0.0,
            dialog_pad: 20.0,
            message_radius: 0.0,
            message_pad: 8.0,
            message_tint: 0,
            scrim: 0xB3,
            dialog_width: 460.0,
            toast_width: 380.0,
            edge: 3.0,
            skeleton_percent: 20,
            skeleton_radius: 0.0,
            skeleton_height: 10.0,
            switch_width: 36.0,
            switch_height: 18.0,
        },
    }
}

/// The names of the tokens every style gives (without the leading `--`). The contract names the blocks that need them
/// (`BLOCK_TOKENS`).
pub const NAMES: [&str; 33] = [
    "scrim",
    "dialog-width",
    "dialog-shadow",
    "dialog-radius",
    "dialog-pad",
    "message-radius",
    "message-pad",
    "message-tint",
    "toast-width",
    "toast-edge",
    "skeleton-base",
    "skeleton-radius",
    "skeleton-height",
    "switch-width",
    "switch-height",
    "segment-height",
    "segment-pad",
    "segment-on-bg",
    "segment-on-line",
    "segment-on-line-width",
    "segment-on-text",
    "switch-on-track",
    "switch-on-thumb",
    "switch-off-track",
    "switch-off-thumb",
    "switch-disabled-opacity",
    "alert-info-bg",
    "alert-ok-bg",
    "alert-warning-bg",
    "alert-error-bg",
    "dialog-danger-edge",
    "dialog-head-line",
    "dialog-head-gap",
];

pub use super::block_needs::BLOCK_TOKENS;

fn shadow(style: &Style) -> String {
    match style.effects.overlay_shadow {
        None => "none".to_owned(),
        Some(s) => format!(
            "{}px {}px {}px {}px {}",
            s.dx,
            s.dy,
            s.blur,
            s.spread,
            s.color.css()
        ),
    }
}

/// The tokens of `style`: `(name without the dashes, value)`, in the order of [`NAMES`].
pub fn tokens(style: &Style) -> Vec<(&'static str, String)> {
    let l = look(style.id);
    let mut values = vec![
        Rgba::hex(0).alpha(l.scrim).css(),
        format!("{}px", l.dialog_width),
        shadow(style),
        format!("{}px", l.dialog_radius),
        format!("{}px", l.dialog_pad),
        format!("{}px", l.message_radius),
        format!("{}px", l.message_pad),
        format!("{}%", l.message_tint),
        format!("{}px", l.toast_width),
        format!("{}px", l.edge),
        format!(
            "color-mix(in srgb, var(--text-primary) {}%, var(--surface))",
            l.skeleton_percent
        ),
        format!("{}px", l.skeleton_radius),
        format!("{}px", l.skeleton_height),
        format!("{}px", l.switch_width),
        format!("{}px", l.switch_height),
        format!("{}px", style.heights.control),
    ];
    values.extend(block_marks::values(style.id).map(str::to_owned));
    NAMES.into_iter().zip(values).collect()
}

/// The tokens of `style` as one `:root` rule.
pub fn sheet(style: &Style) -> String {
    use std::fmt::Write;
    let mut css = String::from(":root {");
    for (name, value) in tokens(style) {
        let _ = write!(css, " --{name}: {value};");
    }
    css.push_str(" }\n");
    css
}

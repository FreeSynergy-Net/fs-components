//! The colours a style puts into the theme variables that the blocks of contract 0.2 read (`--text-link`, `--info-bg`,
//! `--success-bg`, `--warning-bg`, `--error-bg`, `--border-error`): data of the style, read by the adapter that makes a Blinc theme
//! of a style. A style without an entry leaves the variables to the adapter's defaults.
//!
//! Pattern: none (one pure function over constants). Values of Admin: `OpenUI5` `sap_horizon` and `sap_horizon_dark`
//! (commit 4eafc4e193e5bdf485d8168dc74fcf1ffb9fc4bc), Apache-2.0, values only; "berechnet" = computed from the Less of the theme.

use crate::color::Rgba;
use crate::style::StyleId;

/// The colours of one appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockColors {
    /// A link, and the selected state's edge and words (`--text-link`).
    pub link: Rgba,
    /// Ground of a message of the step info (`--info-bg`).
    pub info_bg: Rgba,
    /// Ground of a message of the step ok (`--success-bg`).
    pub ok_bg: Rgba,
    /// Ground of a message of the step warning (`--warning-bg`).
    pub warning_bg: Rgba,
    /// Ground of a message of the step error (`--error-bg`).
    pub error_bg: Rgba,
    /// The line of the error (`--border-error`).
    pub error_line: Rgba,
}

/// The colours of `id` for the light or the dark appearance.
pub const fn of(id: StyleId, dark: bool) -> Option<BlockColors> {
    match id {
        StyleId::Admin => Some(if dark {
            BlockColors {
                // sap_horizon_dark base.less:sapHighlightColor = #4db1ff. DEVIATION: sapLinkColor dark = darken(#4db1ff,15) = #008fff
                // (base.less L94); `--text-link` is shared with the selected state (sapSelectedColor = #4db1ff), so dark links are #4db1ff
                link: Rgba::hex(0x4DB1FF),
                // sap_horizon_dark base.less:sapInformationBackground = #00144a
                info_bg: Rgba::hex(0x00144A),
                // sapSuccessBackground = #11331a
                ok_bg: Rgba::hex(0x11331A),
                // sapWarningBackground = #382700
                warning_bg: Rgba::hex(0x382700),
                // sapErrorBackground = #350000
                error_bg: Rgba::hex(0x350000),
                // sapErrorBorderColor = #fa6161
                error_line: Rgba::hex(0xFA6161),
            }
        } else {
            BlockColors {
                // sap_horizon base.less:sapHighlightColor = #0064d8 (berechnet); light sapLinkColor = sapHighlightColor (L94)
                link: Rgba::hex(0x0064D8),
                // sapInformationBackground = #e1f4ff
                info_bg: Rgba::hex(0xE1F4FF),
                // sapSuccessBackground = #f5fae5
                ok_bg: Rgba::hex(0xF5FAE5),
                // sapWarningBackground = #fff8d6
                warning_bg: Rgba::hex(0xFFF8D6),
                // sapErrorBackground = #ffeaf4
                error_bg: Rgba::hex(0xFFEAF4),
                // sapErrorBorderColor = lighten(#aa0808,13) = #e90b0b (berechnet)
                error_line: Rgba::hex(0xE90B0B),
            }
        }),
        StyleId::Simple | StyleId::Apple | StyleId::Windows | StyleId::Craft => None,
    }
}

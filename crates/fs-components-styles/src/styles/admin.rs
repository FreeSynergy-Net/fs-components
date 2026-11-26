//! Admin: SAP Fiori (Horizon) measures for dense business data.
//!
//! Pattern: none (plain data). From `docs/en/components/styles/fiori.md` (SAP Horizon tokens of
//! `SAP/fundamental-styles`): font sizes 12/14/16/24, control 36 (compact 26), line/row 44 (compact
//! 32), spacing 8/16/32/48, focus `#0032a5` 2 px; corners and colours are Horizon's (below), the sheet's "element radius 12" holds for cards only.
//!
//! Colours and corners follow SAP Horizon (owner's decision 2026-11-16), as values only (`OpenUI5`,
//! Apache-2.0, commit 4eafc4e193e5bdf485d8168dc74fcf1ffb9fc4bc; "SAP" and "Horizon" only describe the
//! model): every token carries its source as `// OpenUI5 sap_horizon[_dark] <file>:<parameter> = <value>`;
//! a value the theme computes (Less `lighten`, `darken`, `mix`) says "berechnet". **Not in the source,
//! kept from before (open):** line height of text (20), the light field border (source `#556b81`, berechnet; kept `#758ca4`),
//! the field grounds (source `#eff1f2` light and `#242e38` dark, both berechnet; kept = panel), shadows (the dialog's shadow is not
//! tied to a parameter in
//! `Dialog.less`), toast colours. Cozy is the default here; the sheet proposes compact for the generated
//! admin frontend (owner's decision open): the dense heights are the compact ones. Font: "72" is not
//! free; Inter (OFL), Noto Sans.

use crate::color::Rgba;
use crate::effects::{Effects, FocusRing, Shadow, Surface};
use crate::icons::{Cap, IconSet, Join};
use crate::style::{Provenance, Style, StyleId};
use crate::tokens::{Heights, Palette, Radii, Spacing, Type};

const fn c(rgb: u32) -> Rgba {
    Rgba::hex(rgb)
}

/// The Admin style.
pub fn build() -> Style {
    Style {
        id: StyleId::Admin,
        provenance: Provenance::Partial,
        fonts: &["Inter", "Noto Sans", "DejaVu Sans"],
        spacing: Spacing {
            xs: 4.0,
            s: 8.0,
            m: 16.0,
            l: 32.0,
            xl: 48.0,
        },
        heights: Heights {
            control: 36.0,
            control_dense: 26.0,
            row: 44.0,
            row_dense: 32.0,
            head: 44.0,
        },
        text: Type {
            caption: 12.0,
            body: 14.0,
            body_line: 20.0,
            title: 16.0,
            headline: 24.0,
            label_weight: 400,
        },
        // OpenUI5 sap_horizon base.less:sapButton_BorderCornerRadius = 0.5rem (8px), also sap_horizon_dark
        // OpenUI5 sap_horizon base.less:sapElement_BorderCornerRadius = 0.75rem (12px) (cards)
        // OpenUI5 sap_horizon base.less:sapPopover_BorderCornerRadius = 0.5rem (8px), also sap_horizon_dark: the corner of a
        // dialog, set by `dialog_radius` in `block_tokens.rs` (`overlay` stays 12: sheets and the like, and `control <= card <=
        // overlay` is a rule of the style tests). The corner of a field is not read from the source: it follows `control`.
        radii: Radii {
            control: 8.0,
            card: 12.0,
            overlay: 12.0,
        },
        light: Palette {
            // OpenUI5 sap_horizon base.less:sapBackgroundColor = #f5f6f7
            ground: c(0xF5F6F7),
            // OpenUI5 sap_horizon base.less:sapBaseColor = #ffffff
            panel: c(0xFFFFFF),
            // sapField_Background is #eff1f2 (berechnet, mix); kept white: the field stands on the panel
            field: c(0xFFFFFF),
            // OpenUI5 sap_horizon base.less:sapTextColor = #131e29
            text: c(0x131E29),
            // OpenUI5 sap_horizon base.less:sapContent_LabelColor = #556b82
            text_weak: c(0x556B82),
            // OpenUI5 sap_horizon base.less:sapPrimary5 = #758ca4 (the role as control border is ours)
            border: c(0x758CA4),
            // OpenUI5 sap_horizon base.less:sapButton_Emphasized_Background = sapBrandColor = #0070f2 (the fill)
            accent: c(0x0070F2),
            // OpenUI5 sap_horizon base.less:sapHighlightColor = darken(#0070f2,5) = #0064d8 (berechnet): accent as text and edge
            accent_text: c(0x0064D8),
            // OpenUI5 sap_horizon base.less:sapButton_Emphasized_TextColor = #ffffff (berechnet, contrast)
            on_accent: c(0xFFFFFF),
            // OpenUI5 sap_horizon base.less:sapContent_FocusColor = #0032a5 (2 px, solid)
            focus: c(0x0032A5),
            cursor: c(0x0032A5),
            // OpenUI5 sap_horizon base.less:sapHoverColor = darken(#f5f6f7,4) = #eaecee (berechnet)
            hover: c(0xEAECEE),
            // OpenUI5 sap_horizon base.less:sapList_SelectionBackgroundColor = sapPrimary1 = #ebf8ff
            selected: c(0xEBF8FF),
            // OpenUI5 sap_horizon base.less:sapNegativeColor = #aa0808
            error: c(0xAA0808),
            // OpenUI5 sap_horizon base.less:sapPositiveColor = #256f3a
            success: c(0x256F3A),
            content_tint: 250,
        },
        dark: Palette {
            // OpenUI5 sap_horizon_dark base.less:sapBackgroundColor = #12171c
            ground: c(0x12171C),
            // OpenUI5 sap_horizon_dark base.less:sapBaseColor = #1d232a
            panel: c(0x1D232A),
            // sapField_Background dark is #242e38 (berechnet, mix); kept = the panel, like light. Border: see below
            field: c(0x1D232A),
            // OpenUI5 sap_horizon_dark base.less:sapTextColor = #f5f6f7
            text: c(0xF5F6F7),
            // OpenUI5 sap_horizon_dark base.less:sapContent_LabelColor = #8396a8
            text_weak: c(0x8396A8),
            // OpenUI5 sap_horizon_dark base.less:sapField_BorderColor = sapPrimary5 = #a9b4be
            border: c(0xA9B4BE),
            // OpenUI5 sap_horizon_dark base.less:sapButton_Emphasized_Background = sapBrandColor = #0070f2 (the fill)
            accent: c(0x0070F2),
            // OpenUI5 sap_horizon_dark base.less:sapHighlightColor = #4db1ff: accent as text and edge
            accent_text: c(0x4DB1FF),
            // OpenUI5 sap_horizon_dark base.less:sapButton_Emphasized_TextColor = contrast(#0070f2, ...) = #ffffff (berechnet)
            on_accent: c(0xFFFFFF),
            // OpenUI5 sap_horizon_dark base.less:sapContent_FocusColor = #9ad3ff
            focus: c(0x9AD3FF),
            cursor: c(0x9AD3FF),
            // OpenUI5 sap_horizon_dark base.less:sapHoverColor = lighten(#12171c,8) = #222b35 (berechnet)
            hover: c(0x222B35),
            // OpenUI5 sap_horizon_dark base.less:sapList_SelectionBackgroundColor = sapPrimary1 = #1d2d3e
            selected: c(0x1D2D3E),
            // OpenUI5 sap_horizon_dark base.less:sapNegativeColor = #fa6161
            error: c(0xFA6161),
            // OpenUI5 sap_horizon_dark base.less:sapPositiveColor = #97dd40
            success: c(0x97DD40),
            content_tint: 250,
        },
        effects: Effects {
            chrome: Surface::Solid,
            overlay: Surface::Solid,
            card_shadow: Some(Shadow {
                dx: 0.0,
                dy: 0.0,
                blur: 2.0,
                spread: 0.0,
                color: c(0x0D1F33).alpha(0x40),
            }),
            overlay_shadow: Some(Shadow {
                dx: 0.0,
                dy: 8.0,
                blur: 32.0,
                spread: 0.0,
                color: c(0x0D1F33).alpha(0x40),
            }),
            focus: FocusRing {
                width: 2.0,
                offset: 2.0,
            },
        },
        effects_dark: Effects {
            chrome: Surface::Solid,
            overlay: Surface::Solid,
            card_shadow: Some(Shadow {
                dx: 0.0,
                dy: 0.0,
                blur: 2.0,
                spread: 0.0,
                color: c(0x0D1F33).alpha(0x40),
            }),
            overlay_shadow: Some(Shadow {
                dx: 0.0,
                dy: 8.0,
                blur: 32.0,
                spread: 0.0,
                color: c(0x0D1F33).alpha(0x40),
            }),
            focus: FocusRing {
                width: 2.0,
                offset: 2.0,
            },
        },
        icons: IconSet {
            stroke: 1.2,
            cap: Cap::Butt,
            join: Join::Miter,
            size: 20.0,
        },
    }
}

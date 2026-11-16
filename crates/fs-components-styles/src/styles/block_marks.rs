//! The look of the selected, on and error states of the blocks of contract 0.2, as theme-variable expressions: one entry per
//! style, written out. A style that has no look of its own for a state gets [`STANDARD`] (the accent fills the chosen segment and
//! the track of a switch that is on, white text on it).
//!
//! Pattern: Registry by exhaustive `match` ([`marks`] names every [`StyleId`]) over plain data ([`Marks`]). Values are CSS text
//! over the theme variables, so they follow the light and the dark scheme; what a variable carries in Admin (`--text-link`,
//! `--info-bg` ..., `--border-error`) is in `block_colors.rs`.

use crate::style::StyleId;

/// The expressions of one style (a CSS value each).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Marks {
    /// Padding left and right inside a segment.
    pub segment_pad: &'static str,
    /// Ground of the chosen segment.
    pub segment_on_bg: &'static str,
    /// Edge of the chosen segment.
    pub segment_on_line: &'static str,
    /// Width of that edge (0: none).
    pub segment_on_line_width: &'static str,
    /// Words of the chosen segment.
    pub segment_on_text: &'static str,
    /// Track of a switch that is on (the edge too).
    pub switch_on_track: &'static str,
    /// Thumb of a switch that is on.
    pub switch_on_thumb: &'static str,
    /// Track of a switch that is off.
    pub switch_off_track: &'static str,
    /// Thumb of a switch that is off.
    pub switch_off_thumb: &'static str,
    /// Opacity of a switch that cannot be changed.
    pub switch_disabled_opacity: &'static str,
    /// Ground of an alert of the step info.
    pub alert_info_bg: &'static str,
    /// Ground of an alert of the step ok.
    pub alert_ok_bg: &'static str,
    /// Ground of an alert (and of the partial notice) of the step warning.
    pub alert_warning_bg: &'static str,
    /// Ground of an alert of the step error.
    pub alert_error_bg: &'static str,
    /// Edge of a dialog in the state `danger`.
    pub dialog_danger_edge: &'static str,
    /// Width of the line under the head of a dialog in the state `danger` (0: none).
    pub dialog_head_line: &'static str,
    /// Room between the words of the head and that line.
    pub dialog_head_gap: &'static str,
}

/// What every style but Admin does (the look before Horizon).
pub const STANDARD: Marks = Marks {
    segment_pad: "var(--space-3)",
    segment_on_bg: "var(--accent)",
    segment_on_line: "var(--accent)",
    segment_on_line_width: "0px",
    segment_on_text: "var(--text-inverse)",
    switch_on_track: "var(--accent)",
    switch_on_thumb: "var(--text-inverse)",
    switch_off_track: "var(--input-bg)",
    switch_off_thumb: "var(--text-secondary)",
    switch_disabled_opacity: "0.5",
    alert_info_bg: "color-mix(in srgb, var(--info) var(--message-tint), var(--surface))",
    alert_ok_bg: "color-mix(in srgb, var(--success) var(--message-tint), var(--surface))",
    alert_warning_bg: "color-mix(in srgb, var(--warning) var(--message-tint), var(--surface))",
    alert_error_bg: "color-mix(in srgb, var(--error) var(--message-tint), var(--surface))",
    dialog_danger_edge: "var(--error)",
    dialog_head_line: "0px",
    dialog_head_gap: "0px",
};

/// The entry of `id`. Every style has its own; there is no default.
pub const fn marks(id: StyleId) -> Marks {
    match id {
        // SAP Horizon (values: OpenUI5 sap_horizon[_dark] SegmentedButton.less, Switch.less, Dialog.less, base.less). The ground of a
        // chosen segment and the thumb of a switch that is on are the accent in 8 % over the surface: the source says #ecf5ff
        // (light) and #002643 (dark), both berechnet; one expression cannot reach both, so the dark one is near, not equal.
        StyleId::Admin => Marks {
            // SegmentedButton.less: padding 0.5625rem = 9 px
            segment_pad: "9px",
            // sapButton_Selected_Background (berechnet) #ecf5ff / #002643; here approximated
            segment_on_bg: "color-mix(in srgb, var(--accent) 8%, var(--surface))",
            // sapButton_Selected_BorderColor = #0064d8 / #4db1ff (= `--text-link`, sapLinkColor = sapHighlightColor)
            segment_on_line: "var(--text-link)",
            segment_on_line_width: "1px",
            // sapButton_Selected_TextColor, light #0064d8 (berechnet); dark: see CHANGELOG (the source's #1d232a is too dark)
            segment_on_text: "var(--text-link)",
            // Switch.less .sapMSwtOn: sapButton_Track_Selected_Background #0064d8 / #4db1ff
            switch_on_track: "var(--text-link)",
            // Switch.less .sapMSwtHandle: #ecf5ff / #002643 (berechnet), approximated as the segment
            switch_on_thumb: "color-mix(in srgb, var(--accent) 8%, var(--surface))",
            // Switch.less .sapMSwtOff: sapNeutralColor #788fa6 / #a9b4be; here the control border (#758ca4 light, #8396a8 dark)
            switch_off_track: "var(--border)",
            // Switch.less: #ffffff / #1d232a = the surface
            switch_off_thumb: "var(--surface)",
            // Switch.less .sapMSwtDisabled: sapContent_DisabledOpacity 0.4 (base.less L451)
            switch_disabled_opacity: "0.4",
            // MessageStrip: sapInformationBackground, sapSuccessBackground, sapWarningBackground, sapErrorBackground
            alert_info_bg: "var(--info-bg)",
            alert_ok_bg: "var(--success-bg)",
            alert_warning_bg: "var(--warning-bg)",
            alert_error_bg: "var(--error-bg)",
            // Dialog.less has no edge rule: the dialog keeps its edge in the danger state ...
            dialog_danger_edge: "var(--border)",
            // ... and gets one line under the head (--_sap_m_Dialog_ErrorHeaderShadow: inset 0 -0.0625rem sapErrorBorderColor)
            dialog_head_line: "1px",
            // Dialog.less: the head has padding; 8 px (ours, the source gives it in rem of the head, not read)
            dialog_head_gap: "8px",
        },
        StyleId::Simple | StyleId::Apple | StyleId::Windows | StyleId::Craft => STANDARD,
    }
}

/// The values of `id`, in the order of `block_tokens::NAMES` after `segment-height`.
pub const fn values(id: StyleId) -> [&'static str; 17] {
    let m = marks(id);
    [
        m.segment_pad,
        m.segment_on_bg,
        m.segment_on_line,
        m.segment_on_line_width,
        m.segment_on_text,
        m.switch_on_track,
        m.switch_on_thumb,
        m.switch_off_track,
        m.switch_off_thumb,
        m.switch_disabled_opacity,
        m.alert_info_bg,
        m.alert_ok_bg,
        m.alert_warning_bg,
        m.alert_error_bg,
        m.dialog_danger_edge,
        m.dialog_head_line,
        m.dialog_head_gap,
    ]
}

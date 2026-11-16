//! The scenes of the pictures of the blocks of contract version 0.2: every block with all its states side by side.
//!
//! Tests: support for `blocks_styles.rs` (pictures, theme guard, "two styles") and `blocks_contrast.rs` (WCAG contrast).
#![allow(dead_code)]

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_fields::controls::{STYLES as CONTROL_STYLES, Segmented, Switch, SwitchState};
use fs_components_styles::StyleId;
use fs_components_styles::styles::block_tokens;
use fs_components_widgets::dialogs::{
    ChoiceDialog, ChoiceOption, ConfirmDialog, Labels, LicenseDialog, modal,
};
use fs_components_widgets::icon::Glyph;
use fs_components_widgets::states::{
    Alert, EmptyState, Labels as StateLabels, PartialNotice, Severity, Skeleton,
};
use fs_components_widgets::toast::{Labels as ToastLabels, Toast, Toasts};

/// The themes: every style light, every style but Craft (dark by nature) dark.
pub const THEMES: [(StyleId, bool); 9] = [
    (StyleId::Admin, false),
    (StyleId::Simple, false),
    (StyleId::Apple, false),
    (StyleId::Windows, false),
    (StyleId::Craft, true),
    (StyleId::Admin, true),
    (StyleId::Simple, true),
    (StyleId::Apple, true),
    (StyleId::Windows, true),
];

/// The sheets of the blocks: the widgets, then the controls (the tokens of the style come with the bundle).
pub fn css() -> String {
    format!("{}\n{CONTROL_STYLES}", fs_components_widgets::styles())
}

/// A scene: name, width, height, builder.
pub type Scene = (&'static str, u32, u32, fn() -> Div);

/// Every scene.
pub const SCENES: [Scene; 9] = [
    ("dialog-modal", 616, 480, modal_scene),
    ("confirm", 536, 700, confirm_scene),
    ("choice", 536, 560, choice_scene),
    ("license", 536, 700, license_scene),
    ("toast", 476, 480, toast_scene),
    ("switch", 416, 300, switch_scene),
    ("segmented", 476, 300, segmented_scene),
    ("skeleton", 496, 400, skeleton_scene),
    ("states", 556, 900, states_scene),
];

fn column() -> Div {
    div().flex_col().gap(3.0).w_full()
}

fn confirm(id: &str, danger: bool) -> ConfirmDialog {
    ConfirmDialog::new(
        &Labels::default(),
        "Remove Tasks?",
        "The program is removed, its data stays.",
    )
    .id(id)
    .confirm_label("Remove")
    .danger(danger)
}

fn modal_scene() -> Div {
    let page = div()
        .flex_col()
        .gap(2.0)
        .child(text("Installed programs").size(18.0))
        .child(text("Tasks   Calendar   Wiki   Files").size(14.0))
        .child(text("Last update: today").size(14.0));
    div()
        .w_full()
        .h_full()
        .child(modal(page, confirm("md", true).build()))
}

fn confirm_scene() -> Div {
    let busy = confirm("cb", false);
    busy.set_busy(true);
    column()
        .child(confirm("c1", false).build())
        .child(confirm("c2", true).build())
        .child(busy.build())
}

fn choice_scene() -> Div {
    column().child(
        ChoiceDialog::new(&Labels::default(), "Which font?")
            .id("ch")
            .message("The theme Craft needs a monospace font.")
            .choose_label("Install")
            .cancel_label("Not now")
            .option(ChoiceOption::new("mono", "JetBrains Mono").hint("OFL, 270 KB"))
            .option(
                ChoiceOption::new("old", "Old Mono")
                    .hint("not free")
                    .disabled(),
            )
            .option(ChoiceOption::new("dejavu", "DejaVu Sans Mono"))
            .build(),
    )
}

fn license_scene() -> Div {
    let text = "SIL OPEN FONT LICENSE Version 1.1\n\nPREAMBLE\nThe goals of the Open Font License (OFL) are to stimulate worldwide development of collaborative font projects.\n\nPERMISSION & CONDITIONS\nPermission is hereby granted, free of charge, to any person obtaining a copy of the Font Software.\n\nTERMINATION\nThis license becomes null and void if any of the above conditions are not met.\n\nDISCLAIMER\nTHE FONT SOFTWARE IS PROVIDED \"AS IS\".";
    column().child(
        LicenseDialog::new(
            &Labels::default(),
            "Licence of JetBrains Mono",
            "SIL OFL 1.1",
            text,
        )
        .id("ld")
        .on_copy(|_| {})
        .build(),
    )
}

fn toast_scene() -> Div {
    let t = Toasts::new(&ToastLabels::default()).id("ts");
    t.push_at(Toast::new(Severity::Info, "Catalog updated."), 0);
    t.push_at(
        Toast::new(Severity::Ok, "Tasks installed.").action("Open", || {}),
        0,
    );
    t.push_at(
        Toast::new(
            Severity::Warn,
            "fc-cache missing: the font shows after login.",
        ),
        0,
    );
    t.push_at(
        Toast::new(Severity::Error, "Download failed: no connection."),
        0,
    );
    div().w_full().child(t.build())
}

fn switch_scene() -> Div {
    let sw = |id: &str, label: &str, state: SwitchState, off: bool| {
        Switch::new(label)
            .id(id)
            .state_is(state)
            .disabled(off)
            .build()
    };
    column()
        .child(sw("s1", "Daily timer: off", SwitchState::Off, false))
        .child(sw("s2", "Daily timer: on", SwitchState::On, false))
        .child(sw("s3", "Sources: some on", SwitchState::Mixed, false))
        .child(sw("s4", "Locked: off", SwitchState::Off, true))
        .child(sw("s5", "Locked: on", SwitchState::On, true))
}

fn segmented_scene() -> Div {
    let three = Segmented::new()
        .id("g2")
        .segment("list", "List", false)
        .segment("grid", "Grid", false)
        .segment("map", "Map", true);
    column()
        .child(
            Segmented::new()
                .id("g1")
                .segment("user", "User", false)
                .segment("system", "System", false)
                .build(),
        )
        .child(three.build())
        .child(
            Segmented::new()
                .id("g3")
                .segment("a", "Day", false)
                .segment("b", "Week", false)
                .disabled(true)
                .build(),
        )
}

fn skeleton_scene() -> Div {
    column()
        .child(Skeleton::new("Loading").id("sk1").lines(3).build())
        .child(Skeleton::new("Loading").id("sk2").rows(2).build())
        .child(
            Skeleton::new("Loading")
                .id("sk3")
                .block(160.0, 48.0)
                .build(),
        )
}

fn states_scene() -> Div {
    let l = StateLabels::default();
    let missing = vec!["Work".to_owned(), "Holidays".to_owned()];
    column()
        .child(
            EmptyState::new("No appointments in this period")
                .id("em")
                .glyph(Glyph::Calendar)
                .text("Create one or pick another period.")
                .action("Create appointment", || {})
                .build(),
        )
        .child(
            Alert::failed(&l, "The server did not answer in time.", || {})
                .id("a-failed")
                .build(),
        )
        .child(
            PartialNotice::new(&l, &missing, 9)
                .id("pn")
                .retry(&l, || {})
                .build(),
        )
        .child(
            Alert::new(Severity::Info, "The catalog has 120 entries.")
                .id("a-info")
                .build(),
        )
        .child(
            Alert::new(Severity::Ok, "Everything is up to date.")
                .id("a-ok")
                .build(),
        )
        .child(
            Alert::new(Severity::Warn, "The system is offline.")
                .id("a-warn")
                .title("Offline")
                .build(),
        )
}

/// The sheet of the Admin tokens for a harness that has no bundle of a style.
pub fn admin_tokens() -> String {
    block_tokens::sheet(fs_components_styles::style(StyleId::Admin))
}

//! Pictures of the bundle G2 (alertdialog, popover, tooltip, banner, toast) in a FreeSynergy style, light and dark, with the theme
//! guard of `styled.rs` ([THEME-WAECHTER]); kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws every scene in the style Admin, light and dark, and asserts that something was drawn and that the dark picture
//! differs from the light one; the pictures are for a person to look at (rule: pictures before release).

mod common;
#[path = "common/styled.rs"]
mod styled;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::banner::{Banner, Labels as BannerLabels};
use fs_components_widgets::dialogs::{AlertDialog, Labels};
use fs_components_widgets::popover::{Labels as PopoverLabels, Popover};
use fs_components_widgets::states::Severity;
use fs_components_widgets::toast::{Labels as ToastLabels, Toast, Toasts};
use fs_components_widgets::tooltip::Tooltip;
use styled::{STRIP, SWATCH, draw_styled, ground_of};

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 6] = [
    ("alertdialog", 640, 260, alertdialog),
    ("alertdialog-danger", 640, 260, alertdialog_danger),
    ("popover", 440, 260, popover),
    ("tooltip", 440, 200, tooltip),
    ("banner", 760, 460, banners),
    ("toast", 560, 330, toasts),
];

fn trigger(label: &str) -> Div {
    div().w(90.0).h(32.0).child(text(label.to_owned()))
}

fn alertdialog() -> Div {
    let d = AlertDialog::new(
        &Labels::default(),
        "Connection lost",
        "Your changes are kept on this computer.",
    )
    .id("g2-ad1");
    div().w(520.0).child(d.build())
}

fn alertdialog_danger() -> Div {
    let d = AlertDialog::new(
        &Labels::default(),
        "Delete 3 tasks?",
        "This cannot be taken back.",
    )
    .id("g2-ad2")
    .with_cancel(None)
    .ok_label("Delete")
    .danger(true);
    div().w(520.0).child(d.build())
}

fn popover() -> Div {
    let pv = Popover::new(&PopoverLabels::default(), "Filters").id("g2-pv");
    pv.open();
    div().child(pv.build(|| trigger("Filters"), || div().child(text("Only my tasks"))))
}

fn tooltip() -> Div {
    let tip = Tooltip::new("Save the page").id("g2-tip").delay(0);
    tip.focus(true);
    div().child(tip.attach(trigger("Save")))
}

fn banners() -> Div {
    let mut column = div().w(660.0).flex_col().gap(8.0);
    for (i, step) in Severity::ALL.into_iter().enumerate() {
        let b = Banner::new(
            &BannerLabels::default(),
            step,
            "You are offline. Changes are kept.",
        )
        .id(&format!("g2-bn{i}"))
        .action("Reconnect", || {});
        column = column.child(b.build());
    }
    column
}

fn toasts() -> Div {
    let t = Toasts::new(&ToastLabels::default()).id("g2-t");
    t.push_at(Toast::new(Severity::Ok, "Saved"), 0);
    t.push_at(
        Toast::new(Severity::Warn, "Low disk space").action("Details", || {}),
        0,
    );
    t.push_at(Toast::new(Severity::Error, "Could not send"), 0);
    div().w(460.0).child(t.build())
}

#[test]
fn every_scene_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let light = draw_styled(
            build,
            w,
            h,
            StyleId::Admin,
            false,
            &css,
            &format!("g2-{name}.png"),
        );
        let dark = draw_styled(
            build,
            w,
            h,
            StyleId::Admin,
            true,
            &css,
            &format!("g2-{name}.png"),
        );
        for (img, mode) in [(&light, "light"), (&dark, "dark")] {
            let ground = ground_of(img);
            let ink = img
                .enumerate_pixels()
                .filter(|(x, y, _)| *x >= STRIP + 4 || *y >= SWATCH + 4)
                .filter(|(_, _, p)| {
                    (0..3)
                        .map(|i| i32::from(p.0[i].abs_diff(ground[i])))
                        .sum::<i32>()
                        > 40
                })
                .count();
            assert!(
                ink > 600,
                "{name} {mode}: nearly empty ({ink} ink pixels)"
            );
        }
        assert_ne!(
            ground_of(&light),
            ground_of(&dark),
            "{name}: dark is not dark"
        );
    }
}

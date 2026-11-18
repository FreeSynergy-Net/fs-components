//! Pictures of the bundle G2 (alertdialog, popover, tooltip, banner, toast), kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws each block offscreen with the theme and the default look of Blinc's buttons and asserts only that something was
//! drawn; the pictures are for a person to look at (rule: pictures before release).

mod common;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use common::{background, draw_window, ink, serial};
use fs_components_widgets::banner::{Banner, Labels as BannerLabels};
use fs_components_widgets::dialogs::{AlertDialog, Labels};
use fs_components_widgets::popover::{Labels as PopoverLabels, Popover};
use fs_components_widgets::states::Severity;
use fs_components_widgets::toast::{Labels as ToastLabels, Toast, Toasts};
use fs_components_widgets::tooltip::Tooltip;

fn drawn(img: &image::RgbaImage) {
    let (w, h) = img.dimensions();
    assert!(
        ink(img, (0, 0, w, h), background(img)) > 200,
        "nothing drawn"
    );
}

fn trigger(label: &str) -> Div {
    div().w(90.0).h(32.0).child(text(label.to_owned()))
}

#[test]
fn alertdialog_in_light_and_with_a_cancel_button() {
    let _g = serial();
    let one = AlertDialog::new(
        &Labels::default(),
        "Connection lost",
        "Your changes are kept on this computer.",
    )
    .id("g2-ad1");
    drawn(&draw_window(one.build(), 480, 220, "g2-alertdialog.png"));
    let two = AlertDialog::new(
        &Labels::default(),
        "Delete 3 tasks?",
        "This cannot be taken back.",
    )
    .id("g2-ad2")
    .with_cancel(None)
    .ok_label("Delete")
    .danger(true);
    drawn(&draw_window(
        two.build(),
        480,
        220,
        "g2-alertdialog-danger.png",
    ));
}

#[test]
fn popover_open_and_tooltip_shown() {
    let _g = serial();
    let pv = Popover::new(&PopoverLabels::default(), "Filters").id("g2-pv");
    pv.open();
    let ui = pv.build(|| trigger("Filters"), || div().child(text("Only my tasks")));
    drawn(&draw_window(ui, 360, 200, "g2-popover.png"));
    let tip = Tooltip::new("Save the page").id("g2-tip").delay(0);
    tip.focus(true);
    drawn(&draw_window(
        tip.attach(trigger("Save")),
        360,
        120,
        "g2-tooltip.png",
    ));
}

#[test]
fn banner_in_four_steps_and_the_toast_stack() {
    let _g = serial();
    let mut column = div().w(560.0).flex_col().gap(8.0);
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
    drawn(&draw_window(column, 600, 360, "g2-banner.png"));
    let t = Toasts::new(&ToastLabels::default()).id("g2-t");
    t.push_at(Toast::new(Severity::Ok, "Saved"), 0);
    t.push_at(
        Toast::new(Severity::Warn, "Low disk space").action("Details", || {}),
        0,
    );
    t.push_at(Toast::new(Severity::Error, "Could not send"), 0);
    drawn(&draw_window(t.build(), 440, 220, "g2-toast.png"));
}

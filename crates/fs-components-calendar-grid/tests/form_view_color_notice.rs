//! The notice above the grid about colours (KG7-b): a colour that cannot be read, and a colour too weak for the style, which keeps
//! its value and gets an outline (ADR 0074 section 9.4: not overwritten, reported).
//!
//! Pattern: Test double (as `form_view_color`) – fixed rows with one colour that cannot be read or one that is faint.
#![allow(clippy::pedantic)]

#[path = "support/colour.rs"]
mod colour;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use colour::{Shot, draw, look, rgb, rows, stripe};
use fs_components_calendar_grid::colour_notice::lines;
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use ui::serial;

#[test]
fn a_colour_that_cannot_be_read_is_reported_and_the_row_stays() {
    let _g = serial();
    for dark in [false, true] {
        let shot = Shot {
            style: StyleId::Admin,
            dark,
            agenda: false,
        };
        let name = format!("kg7_unreadable_{}.png", if dark { "dark" } else { "light" });
        let Some(d) = draw(rows(Some("lane-a"), Some("#d6336c")), shot, &name) else {
            continue;
        };
        assert!(
            d.tree.query_by_id("calendar-colour-notice-0").is_some(),
            "the notice names it"
        );
        assert!(
            d.tree.query_by_id("calendar-colour-notice-1").is_none(),
            "only one line"
        );
        assert!(
            d.tree.query_by_id("cal-item-a-2").is_some(),
            "the row stays"
        );
        let weak = rgb(look(shot).palette().text_weak);
        assert_eq!(
            stripe(&d, "cal-item-a-2"),
            weak,
            "it shows as without colour"
        );
    }
    ui::set_dark(false);
}

#[test]
fn a_colour_too_weak_for_the_style_keeps_its_value_gets_an_outline_and_a_notice() {
    let _g = serial();
    // White vanishes on the light ground, black on the dark one.
    for (dark, weak_colour) in [(false, "#ffffff"), (true, "#000000")] {
        let shot = Shot {
            style: StyleId::Admin,
            dark,
            agenda: false,
        };
        let name = format!("kg7_faint_{}.png", if dark { "dark" } else { "light" });
        let Some(d) = draw(rows(Some(weak_colour), Some("#d6336c")), shot, &name) else {
            continue;
        };
        let tag = if dark { "dark" } else { "light" };
        assert!(
            d.tree.query_by_id("calendar-colour-notice-0").is_some(),
            "{tag}: reported"
        );
        assert!(
            d.tree.query_by_id("calendar-colour-notice-1").is_none(),
            "{tag}: one colour, one line"
        );
        let c = colour::rgb(fs_components_styles::Rgba::from_css(weak_colour).unwrap());
        assert_eq!(stripe(&d, "cal-item-a-2"), c, "{tag}: the value is kept");
        let node = d.tree.query_by_id("cal-item-a-2").unwrap();
        let b = d.tree.get_absolute_bounds(node).unwrap();
        let edge = d.img.get_pixel(b.x as u32 + 10, b.y as u32).0;
        assert_eq!(
            edge,
            rgb(look(shot).palette().text_weak),
            "{tag}: the outline is the weak ink of the style"
        );
        let sound = draw(
            rows(Some("#2a7de1"), Some("#d6336c")),
            shot,
            &format!("kg7_sound_{tag}.png"),
        )
        .unwrap();
        assert!(
            sound.tree.query_by_id("calendar-colour-notice").is_none(),
            "{tag}: nothing to report"
        );
    }
    ui::set_dark(false);
}

#[test]
fn the_notice_words_are_counted_in_both_languages() {
    let english = lines(&Locale::english(), 2, 1);
    assert_eq!(english.len(), 2);
    assert!(
        english[0].contains('2') && english[1].contains('1'),
        "{english:?}"
    );
    assert!(lines(&Locale::english(), 0, 0).is_empty());
    let german = lines(&Locale::parse("de").unwrap(), 1, 0);
    assert!(german[0].contains("Farbe"), "{german:?}");
}

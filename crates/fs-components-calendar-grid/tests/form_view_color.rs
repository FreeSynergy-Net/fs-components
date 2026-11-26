//! Two calendars in two colours: the channel `color` of `bars-in-time@1.1.0` drawn as given, light and dark, in two styles,
//! on the month and on the agenda. KG7: the colour is the content's, the style only decides how it appears.
//!
//! Pattern: Test double (a `NoWrite` host, a `say` that returns the key) over fixed rows of November 2026, once with, once without
//! `color`, once swapped. They need a graphics adapter and fonts (software Vulkan on the runner); `FS_COMP_SHOTS` keeps the pictures.
#![allow(clippy::pedantic)]

#[path = "support/colour.rs"]
mod colour;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use colour::{Shot, centre, draw, ground, look, rgb, rows, stripe};
use fs_components_styles::{Rgba, StyleId};
use ui::serial;

const HOME: &str = "#2a7de1";
const WORK: &str = "#d6336c";

/// The ids of the first item of `home` (row `a`) and of `work` (row `b`): the month grid and the agenda.
fn ids(agenda: bool) -> (&'static str, &'static str) {
    if agenda {
        ("cal-agenda-mark-c-11-12", "cal-agenda-mark-d-11-12")
    } else {
        ("cal-item-a-2", "cal-item-b-2")
    }
}

fn mark(d: &colour::Drawn, id: &str, agenda: bool) -> [u8; 4] {
    if agenda { centre(d, id) } else { stripe(d, id) }
}

fn name(what: &str, shot: Shot) -> String {
    format!(
        "kg7_{:?}_{}_{}_{what}.png",
        shot.style,
        if shot.dark { "dark" } else { "light" },
        if shot.agenda { "agenda" } else { "month" }
    )
}

#[test]
fn the_colour_is_drawn_as_given_and_follows_the_calendar_light_and_dark() {
    let _g = serial();
    let (home, work) = (Rgba::from_css(HOME).unwrap(), Rgba::from_css(WORK).unwrap());
    for style in [StyleId::Admin, StyleId::Craft] {
        for dark in [false, true] {
            for agenda in [false, true] {
                let shot = Shot {
                    style,
                    dark,
                    agenda,
                };
                let tag = name("", shot);
                let (a, b) = ids(agenda);
                let colored = draw(rows(Some(HOME), Some(WORK)), shot, &name("color", shot));
                let swapped = draw(rows(Some(WORK), Some(HOME)), shot, &name("swapped", shot));
                let (Some(colored), Some(swapped)) = (colored, swapped) else {
                    continue;
                };
                assert_eq!(
                    mark(&colored, a, agenda),
                    rgb(home),
                    "{tag}: exactly the colour given"
                );
                assert_eq!(
                    mark(&colored, b, agenda),
                    rgb(work),
                    "{tag}: exactly the colour given"
                );
                assert_eq!(
                    mark(&swapped, a, agenda),
                    rgb(work),
                    "{tag}: follows the calendar"
                );
                assert_eq!(
                    mark(&swapped, b, agenda),
                    rgb(home),
                    "{tag}: follows the calendar"
                );
                if !agenda {
                    assert_ne!(
                        ground(&colored, a),
                        ground(&colored, b),
                        "{tag}: tinted grounds differ"
                    );
                    assert_eq!(
                        ground(&swapped, a),
                        ground(&colored, b),
                        "{tag}: ground follows too"
                    );
                }
            }
        }
    }
    ui::set_dark(false);
}

#[test]
fn without_colour_the_style_shows_its_quiet_look_that_no_colour_shares() {
    let _g = serial();
    for style in [StyleId::Admin, StyleId::Craft] {
        for dark in [false, true] {
            for agenda in [false, true] {
                let shot = Shot {
                    style,
                    dark,
                    agenda,
                };
                let tag = name("", shot);
                let (a, b) = ids(agenda);
                let plain = draw(rows(None, None), shot, &name("plain", shot));
                let colored = draw(rows(Some(HOME), Some(WORK)), shot, &name("color", shot));
                let (Some(plain), Some(colored)) = (plain, colored) else {
                    continue;
                };
                let weak = rgb(look(shot).palette().text_weak);
                assert_eq!(mark(&plain, a, agenda), weak, "{tag}: quiet mark");
                assert_eq!(mark(&plain, b, agenda), weak, "{tag}: one quiet look");
                assert_ne!(mark(&plain, a, agenda), mark(&colored, a, agenda), "{tag}");
                assert_ne!(mark(&plain, a, agenda), mark(&colored, b, agenda), "{tag}");
                assert!(
                    plain.tree.query_by_id("calendar-colour-notice").is_none(),
                    "{tag}: nothing to report"
                );
            }
        }
    }
    ui::set_dark(false);
}

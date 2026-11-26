//! Pattern: Dependency Injection (test) — the timeline in German and American English, day, week and month zoom, light and
//! dark: the month names on the axis are the CLDR names of the date package, the buttons are the ids of the calendar catalog;
//! nothing on the picture is a key. They need a graphics adapter. The pictures go to `FS_COMP_SHOTS` (local only).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-calendar-grid/tests/support/dates.rs"]
mod dates;
#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-calendar-grid/tests/support/mod.rs"]
mod support;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_core::form_view::PlaceInfo;
use fs_components_timeline::*;
use fs_components_words::Width;
use host::{bars_timeline, seed_records};
use ui::{draw_tree, layout, serial};

const W: f32 = 900.0;
const H: f32 = 520.0;

fn zoom_name(z: Zoom) -> &'static str {
    z.name()
}

/// The timeline for `tag`, drawn at `zoom`; the picture is written, the texts on it come back (buttons from the tree, axis words).
fn shot(tag: &str, zoom: Zoom, dark: bool) -> Vec<String> {
    let _g = serial();
    ui::set_dark(dark);
    let ctx = dates::host_ctx(tag, "auto", PlaceInfo::default(), dark, Vec::new());
    let t = bars_timeline(fs_components_styles::StyleId::Admin, seed_records())
        .localised(&ctx.locale, ctx.dates.clone());
    t.set_zoom(zoom);
    let tree = layout(t.build(W, H), W, H);
    if ui::has_fonts() {
        let name = format!(
            "kw-b6-{tag}-{}-{}.png",
            zoom_name(zoom),
            if dark { "dark" } else { "light" }
        );
        let img = draw_tree(&tree, W as u32, H as u32, &name);
        assert!(
            ui::ink(&img, (0, 0, W as u32, H as u32), ui::background(&img)) > 500,
            "{name}"
        );
    }
    ui::set_dark(false);
    let mut words = t.axis_texts(W);
    words.extend(support::texts_under(&tree, "tl-root"));
    words
}

#[test]
fn every_zoom_is_drawn_in_both_languages_light_and_dark_with_no_key_on_it() {
    for tag in ["de", "en-US"] {
        for zoom in Zoom::ALL {
            for dark in [false, true] {
                let words = shot(tag, zoom, dark);
                assert!(!words.is_empty(), "{tag} {zoom:?}");
                for w in &words {
                    assert!(
                        !w.contains("calendar-") && !w.contains("timeline."),
                        "{tag} {zoom:?}: a key on the picture: {w:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_axis_names_the_months_the_language_way() {
    let de = shot("de", Zoom::Month, false);
    assert!(de.iter().any(|w| w.starts_with("November 2026")), "{de:?}");
    assert!(de.iter().any(|w| w.starts_with("Dezember")), "{de:?}");
    let en = shot("en-US", Zoom::Month, false);
    assert!(en.iter().any(|w| w.starts_with("November 2026")), "{en:?}");
    assert!(en.iter().any(|w| w.starts_with("December")), "{en:?}");
    // The week zoom has narrower month ticks: they carry the medium month title of the language (the first tick is October).
    for tag in ["de", "en-US"] {
        let ctx = dates::host_ctx(tag, "auto", PlaceInfo::default(), false, Vec::new());
        let medium = ctx.dates.month_title(2026, 10, Width::Medium);
        let words = shot(tag, Zoom::Week, false);
        assert!(
            words.contains(&medium),
            "{tag}: {medium:?} not in {words:?}"
        );
    }
}

#[test]
fn the_buttons_and_today_come_from_the_calendar_catalog() {
    let de = shot("de", Zoom::Week, false);
    for w in ["Tag", "Woche", "Monat"] {
        assert!(de.contains(&w.to_owned()), "{w}: {de:?}");
    }
    let en = shot("en-US", Zoom::Week, false);
    for w in ["Day", "Week", "Month"] {
        assert!(en.contains(&w.to_owned()), "{w}: {en:?}");
    }
}

//! Pattern: none (test). The calendar grid in four views, light and dark, in German and in American English: the names come
//! from the CLDR package, the buttons from the catalog; nothing on the picture is a key. They need a graphics adapter.
//! The pictures go to `FS_COMP_SHOTS` (local only).
#![allow(clippy::pedantic)]

#[path = "support/dates.rs"]
mod dates;
#[path = "support/mod.rs"]
mod support;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use dates::{host_ctx, input};
use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::form_view::View;
use fs_components_core::form_read::FormRow;
use fs_components_core::form_view::PlaceInfo;
use ui::{draw_tree, layout, serial};

const W: f32 = 900.0;
const H: f32 = 640.0;

fn row(key: &str, label: &str, start: &str, end: &str, all_day: &str) -> FormRow {
    FormRow::of(
        key,
        &[
            ("label", label),
            ("start", start),
            ("end", end),
            ("all-day", all_day),
        ],
    )
}

fn rows() -> Vec<FormRow> {
    vec![
        row("offsite", "Offsite", "2026-11-03", "2026-11-04", "true"),
        row("trip", "Trip", "2026-11-09", "2026-11-12", "true"),
        row(
            "sync",
            "Sync",
            "2026-11-10T10:00:00+00:00",
            "2026-11-10T11:00:00+00:00",
            "false",
        ),
        row(
            "call",
            "Call",
            "2026-11-10T10:30:00+00:00",
            "2026-11-10T11:30:00+00:00",
            "false",
        ),
        row(
            "review",
            "Review",
            "2026-11-12T13:00:00+00:00",
            "2026-11-12T14:00:00+00:00",
            "false",
        ),
    ]
}

const VIEWS: [&str; 4] = ["month", "week", "day", "agenda"];

/// Mount `view` for `tag`, draw it, and give the picture and the texts on it.
fn shot(tag: &str, view: &str, dark: bool) -> Vec<String> {
    let _g = serial();
    ui::set_dark(dark);
    let place = PlaceInfo::default().with_view(view);
    let ctx = host_ctx(tag, "auto", place, dark, rows());
    let today = Day::new(2026, 11, 10).unwrap();
    let mounted = View.mount_on(&input(rows()), ctx, today);
    let tree = layout(mounted.build(W, H), W, H);
    let name = format!(
        "kw-b5-{tag}-{view}-{}.png",
        if dark { "dark" } else { "light" }
    );
    if ui::has_fonts() {
        let img = draw_tree(&tree, W as u32, H as u32, &name);
        assert!(
            ui::ink(&img, (0, 0, W as u32, H as u32), ui::background(&img)) > 500,
            "{name}"
        );
    }
    ui::set_dark(false);
    support::texts_under(&tree, "cal-root")
}

#[test]
fn every_view_is_drawn_in_both_languages_light_and_dark_with_no_key_on_it() {
    for tag in ["de", "en-US"] {
        for view in VIEWS {
            for dark in [false, true] {
                let texts = shot(tag, view, dark);
                assert!(!texts.is_empty(), "{tag} {view}");
                for t in &texts {
                    assert!(!t.contains("calendar."), "{tag} {view}: {t:?}");
                    assert!(
                        !t.contains("calendar-"),
                        "{tag} {view}: a key on the picture: {t:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn german_names_the_month_and_the_days_the_german_way_and_starts_on_monday() {
    let month = shot("de", "month", false);
    assert!(month.contains(&"November 2026".to_owned()), "{month:?}");
    assert!(month.contains(&"Heute".to_owned()) && month.contains(&"Zurück".to_owned()));
    let at = month.iter().position(|t| t == "Mo").expect("a head");
    let heads = &month[at..at + 7];
    assert_eq!(heads, ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]);
    let agenda = shot("de", "agenda", false);
    assert!(
        agenda
            .iter()
            .any(|t| t.starts_with("Dienstag, 10. November")),
        "{agenda:?}"
    );
    assert!(agenda.contains(&"ganztägig".to_owned()), "{agenda:?}");
    let day = shot("de", "day", false);
    assert!(day.iter().any(|t| t.starts_with("Dienstag")), "{day:?}");
}

#[test]
fn american_english_starts_on_sunday_and_says_the_month_its_way() {
    let month = shot("en-US", "month", false);
    assert!(month.contains(&"November 2026".to_owned()), "{month:?}");
    assert!(month.contains(&"Today".to_owned()) && month.contains(&"Previous".to_owned()));
    assert!(month.iter().any(|t| t == "Sun"), "{month:?}");
    let first = month.iter().position(|t| t == "Sun").unwrap();
    let mon = month.iter().position(|t| t == "Mon").unwrap();
    assert!(first < mon, "Sunday before Monday: {month:?}");
    let agenda = shot("en-US", "agenda", false);
    assert!(
        agenda.iter().any(|t| t.starts_with("Tuesday, November 10")),
        "{agenda:?}"
    );
    assert!(agenda.contains(&"All day".to_owned()), "{agenda:?}");
}

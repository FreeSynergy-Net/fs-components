//! Two calendars in two colours on the month grid: the channel `color` of `bars-in-time@1.1.0` drawn, light and dark.
//!
//! Pattern: Test double (a `NoWrite` host, a `say` that returns the key) over fixed rows of November 2026, once with and once without
//! `color`. They need a graphics adapter and fonts (software Vulkan on the runner); `FS_COMP_SHOTS` keeps the pictures.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_components_calendar_grid::form_view::View;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::DrawCtx;
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::calendar_view::color_slot;
use fs_components_words::Locale;
use ui::{draw_tree, layout, serial};

const W: f32 = 900.0;
const H: f32 = 640.0;

/// Two calendars; their colours as the module would write them (JSCalendar `color`).
fn rows(home: Option<&str>, work: Option<&str>) -> Vec<FormRow> {
    let make = |key: &str, label: &str, day: &str, lane: &str, color: Option<&str>| {
        let mut pairs = vec![
            ("label", label),
            ("start", day),
            ("end", day),
            ("lane", lane),
            ("all-day", "true"),
        ];
        if let Some(c) = color {
            pairs.push(("color", c));
        }
        FormRow::of(key, &pairs)
    };
    vec![
        make("a", "Dentist", "2026-11-09", "home", home),
        make("b", "Planning", "2026-11-10", "work", work),
        make("c", "Birthday", "2026-11-12", "home", home),
        make("d", "Release", "2026-11-12", "work", work),
        make("e", "Market", "2026-11-17", "home", home),
        make("f", "Review", "2026-11-18", "work", work),
    ]
}

fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "bars-in-time@1.1.0".to_owned(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(dark: bool) -> DrawCtx {
    DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, dark),
        Arc::new(|k| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(|| input(rows(None, None))),
    )
}

fn picture(rows: Vec<FormRow>, dark: bool, name: &str) -> Option<image::RgbaImage> {
    ui::set_dark(dark);
    let mounted = View.mount_on(
        &input(rows),
        ctx(dark),
        fs_components_calendar_grid::Day::new(2026, 11, 10).unwrap(),
    );
    let tree = layout(mounted.build(W, H), W, H);
    ui::has_fonts().then(|| draw_tree(&tree, W as u32, H as u32, name))
}

/// The pixel at the empty right part of a chip of the calendar `home` (`500, 105`) or `work` (`500, 45`): its fill, no text.
fn fill(img: &image::RgbaImage, at: (u32, u32)) -> [u8; 4] {
    img.get_pixel(at.0, at.1).0
}

const HOME: (u32, u32) = (110, 304);
const WORK: (u32, u32) = (240, 304);

#[test]
fn two_calendars_in_two_colours_are_drawn_in_two_colours_light_and_dark() {
    let _g = serial();
    let (home, work) = ("#2a7de1", "#d6336c");
    assert_ne!(
        color_slot(home),
        color_slot(work),
        "the fixture needs two slots"
    );
    for dark in [false, true] {
        let tag = if dark { "dark" } else { "light" };
        let colored = picture(
            rows(Some(home), Some(work)),
            dark,
            &format!("kg7grid_color_{tag}.png"),
        );
        let plain = picture(rows(None, None), dark, &format!("kg7grid_plain_{tag}.png"));
        let swapped = picture(
            rows(Some(work), Some(home)),
            dark,
            &format!("kg7grid_swapped_{tag}.png"),
        );
        let (Some(colored), Some(plain), Some(swapped)) = (colored, plain, swapped) else {
            continue;
        };
        assert_ne!(
            fill(&colored, HOME),
            fill(&colored, WORK),
            "{tag}: two calendars, two colours"
        );
        assert_eq!(
            fill(&plain, HOME),
            fill(&plain, WORK),
            "{tag}: no colour, one look as before"
        );
        assert_eq!(
            fill(&swapped, HOME),
            fill(&colored, WORK),
            "{tag}: the colour follows the calendar"
        );
        assert_eq!(
            fill(&swapped, WORK),
            fill(&colored, HOME),
            "{tag}: the colour follows the calendar"
        );
    }
    ui::set_dark(false);
}

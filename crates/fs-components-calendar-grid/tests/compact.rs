//! The variant `compact`: the small month of a side bar, with marks instead of bars, and its week-number column.
//!
//! Pattern: Table test — which variant a place asks for, then what the mounted month holds with and without it (ids and texts of the
//! laid-out tree), then the small month drawn light and dark. The drawing needs a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

mod support;

use std::sync::Arc;

use fs_components_calendar_grid::form_view::View;
use fs_components_calendar_grid::place::variant_of;
use fs_components_calendar_grid::{Day, Variant};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use support::texts_under;
use ui::{draw_tree, layout, serial};

fn day(y: i32, m: u8, d: u8) -> Day {
    Day::new(y, m, d).unwrap()
}

#[test]
fn only_the_word_compact_makes_the_small_month() {
    let table = [
        (None, Variant::Full),
        (Some("compact"), Variant::Compact),
        (Some("color-2"), Variant::Full),
        (Some("Compact"), Variant::Full),
        (Some(""), Variant::Full),
    ];
    for (word, want) in table {
        let place = PlaceInfo {
            variant: word.map(str::to_owned),
            ..PlaceInfo::default()
        };
        assert_eq!(variant_of(&place), want, "{word:?}");
    }
}

fn input() -> FormInput {
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: vec![
            FormRow::of(
                "offsite",
                &[
                    ("label", "Offsite"),
                    ("start", "2026-11-09"),
                    ("end", "2026-11-11"),
                    ("all-day", "true"),
                ],
            ),
            FormRow::of(
                "call",
                &[
                    ("label", "Call"),
                    ("start", "2026-11-20T10:00:00+00:00"),
                    ("end", "2026-11-20T11:00:00+00:00"),
                ],
            ),
        ],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(place: PlaceInfo, dark: bool) -> DrawCtx {
    DrawCtx::new(
        place,
        (StyleId::Admin, dark),
        Arc::new(|k| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
}

fn mini(numbers: bool) -> PlaceInfo {
    let place = PlaceInfo::default().with_variant("compact");
    place.with_option("view.week-numbers", if numbers { "true" } else { "false" })
}

fn tree_of(place: PlaceInfo, w: f32, h: f32) -> blinc_layout::prelude::RenderTree {
    let mounted = View.mount_on(&input(), ctx(place, false), day(2026, 11, 10));
    layout(mounted.build(w, h), w, h)
}

/// Indexes (of the 42 cells) that carry an element with the id prefix.
fn indexes(tree: &blinc_layout::prelude::RenderTree, prefix: &str) -> Vec<usize> {
    (0..42)
        .filter(|i| tree.query_by_id(&format!("{prefix}-{i}")).is_some())
        .collect()
}

#[test]
fn the_small_month_marks_the_days_with_items_and_has_no_bars_and_no_toolbar() {
    let _g = serial();
    let tree = tree_of(mini(false), 300.0, 300.0);
    // Monday 2026-10-26 is cell 0: Nov 9..=11 are 14..=16, Nov 20 is 25.
    assert_eq!(indexes(&tree, "cal-mark"), vec![14, 15, 16, 25]);
    assert_eq!(indexes(&tree, "cal-cell").len(), 42);
    assert!(tree.query_by_id("cal-item-offsite-2").is_none());
    assert!(tree.query_by_id("cal-toolbar").is_none());
    assert!(tree.query_by_id("cal-prev").is_none());
}

#[test]
fn the_full_month_has_bars_and_a_toolbar_and_no_marks() {
    let _g = serial();
    let tree = tree_of(PlaceInfo::default(), 900.0, 640.0);
    assert!(indexes(&tree, "cal-mark").is_empty());
    assert!(tree.query_by_id("cal-toolbar").is_some());
    assert!(tree.query_by_id("cal-item-offsite-2").is_some());
}

#[test]
fn compact_has_no_small_form_of_the_week_so_it_draws_as_usual() {
    let _g = serial();
    let tree = tree_of(mini(false).with_view("week"), 900.0, 640.0);
    assert!(tree.query_by_id("cal-toolbar").is_some());
    assert!(indexes(&tree, "cal-mark").is_empty());
}

#[test]
fn the_week_numbers_of_the_small_month_are_narrower_and_have_no_head() {
    let _g = serial();
    let full = tree_of(
        PlaceInfo::default().with_option("view.week-numbers", "true"),
        900.0,
        640.0,
    );
    let small = tree_of(mini(true), 300.0, 300.0);
    assert_eq!(texts_under(&small, "cal-week-0"), ["44"]);
    assert_eq!(texts_under(&small, "cal-week-5"), ["49"]);
    let heads = texts_under(&small, "cal-week-numbers");
    assert_eq!(heads, ["44", "45", "46", "47", "48", "49"], "no head text");
    assert!(texts_under(&full, "cal-week-numbers").contains(&"Wk".to_owned()));
    let (small_w, _) = ui::size_of(&small, "cal-week-numbers");
    let (full_w, _) = ui::size_of(&full, "cal-week-numbers");
    assert!(small_w < full_w, "{small_w} < {full_w}");
}

#[test]
fn the_small_month_is_drawn_with_its_marks_light_and_dark() {
    let _g = serial();
    let (w, h) = (300.0_f32, 300.0_f32);
    for dark in [false, true] {
        ui::set_dark(dark);
        let name = format!(
            "calendar_compact_{}.png",
            if dark { "dark" } else { "light" }
        );
        let mounted = View.mount_on(&input(), ctx(mini(true), dark), day(2026, 11, 10));
        let tree = layout(mounted.build(w, h), w, h);
        let node = tree.query_by_id("cal-mark-15").expect("a mark");
        let b = tree.get_absolute_bounds(node).expect("bounds");
        let img = draw_tree(&tree, w as u32, h as u32, &name);
        let rect = (
            b.x as u32,
            b.y as u32,
            b.width as u32 + 1,
            b.height as u32 + 1,
        );
        assert!(
            ui::ink(&img, rect, ui::background(&img)) > 0,
            "{name}: the mark is drawn"
        );
    }
    ui::set_dark(false);
}

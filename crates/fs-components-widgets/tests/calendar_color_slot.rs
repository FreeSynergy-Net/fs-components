//! The channel `color` of `bars-in-time@1.1.0` as a colour slot of the calendar components.
//!
//! Tests: Table tests over `color_slot` (same colour, same slot; unreadable = none) and over `read_bars` (read, absent, unreadable counted).

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_widgets::calendar_view::{SLOTS, color_slot, read_bars};

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

#[test]
fn a_colour_has_one_slot_however_it_is_written() {
    let red = color_slot("red").expect("a name");
    assert!(red < SLOTS);
    for same in [
        "#f00",
        "#FF0000",
        "rgb(255 0 0)",
        "rgb(255, 0, 0)",
        " Red ",
        "hsl(0, 100%, 50%)",
    ] {
        assert_eq!(color_slot(same), Some(red), "{same}");
    }
    assert_eq!(color_slot("red"), color_slot("red"), "stable");
}

#[test]
fn every_slot_is_reached_by_some_colour_and_none_is_out_of_range() {
    let mut seen = [false; 6];
    for r in (0..=255).step_by(15) {
        for g in (0..=255).step_by(51) {
            let slot = color_slot(&format!("rgb({r} {g} 90)")).unwrap();
            seen[usize::from(slot)] = true;
        }
    }
    assert!(seen.iter().all(|s| *s), "{seen:?}");
}

#[test]
fn what_is_no_colour_gives_no_slot() {
    for bad in [
        "",
        "  ",
        "lane-a",
        "#12",
        "rgb(1 2)",
        "300",
        "transparent",
        "rgba(0 0 0 / 0)",
    ] {
        assert_eq!(color_slot(bad), None, "{bad:?}");
    }
}

#[test]
fn read_bars_takes_the_slot_of_the_colour_and_leaves_rows_without_one_as_they_were() {
    let base = [("start", "2026-11-25"), ("end", "2026-11-25")];
    let mut with: Vec<(&str, &str)> = base.to_vec();
    with.push(("color", "#2a7de1"));
    let mut bad: Vec<(&str, &str)> = base.to_vec();
    bad.push(("color", "not a colour"));
    let mut lane: Vec<(&str, &str)> = base.to_vec();
    lane.push(("lane", "team"));
    let read = read_bars(&input(vec![
        FormRow::of("a", &with),
        FormRow::of("b", &base),
        FormRow::of("c", &bad),
        FormRow::of("d", &lane),
    ]));
    assert!(read.invalid.is_empty(), "no row is lost for its colour");
    assert_eq!(read.entries.len(), 4);
    assert_eq!(read.entries[0].color, color_slot("#2a7de1"));
    assert!(read.entries[0].color.is_some());
    assert_eq!(read.entries[1].color, None);
    assert_eq!(read.entries[2].color, None, "unreadable = like none");
    assert_eq!(read.entries[3].color, None, "the lane gives no colour");
    assert_eq!(
        read.invalid_color,
        ["c"],
        "the unreadable colour is counted"
    );
    assert_eq!(read.entries[1].to_item().category, 0, "as before");
    assert_eq!(read.entries[2].to_item().category, 0);
    assert_eq!(
        Some(read.entries[0].to_item().category),
        read.entries[0].color
    );
}

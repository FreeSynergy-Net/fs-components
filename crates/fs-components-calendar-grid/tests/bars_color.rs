//! The channel `color` of `bars-in-time@1.1.0` read as the colour the module gave.
//!
//! Tests: Table tests over `read_bars` (the colour as given in every CSS writing, absent, unreadable counted, the row stays).

use fs_components_calendar_grid::calendar_core::read_bars;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_styles::Rgba;

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

const BASE: [(&str, &str); 2] = [("start", "2026-11-25"), ("end", "2026-11-25")];

fn with(extra: (&'static str, &'static str)) -> Vec<(&'static str, &'static str)> {
    let mut pairs = BASE.to_vec();
    pairs.push(extra);
    pairs
}

#[test]
fn a_colour_is_the_same_however_it_is_written() {
    let red = Rgba::from_css("red").expect("a name");
    for same in [
        "#f00",
        "#ff0000",
        "rgb(255 0 0)",
        "hsl(0 100% 50%)",
        " RED ",
    ] {
        assert_eq!(Rgba::from_css(same), Some(red), "{same}");
    }
    let blue = Rgba::from_css("#2a7de1").unwrap();
    assert_eq!((blue.r, blue.g, blue.b), (0x2a, 0x7d, 0xe1), "as given");
}

#[test]
fn read_bars_keeps_the_colour_as_given_and_leaves_rows_without_one_as_they_were() {
    let read = read_bars(&input(vec![
        FormRow::of("a", &with(("color", "#2a7de1"))),
        FormRow::of("b", &BASE),
        FormRow::of("c", &with(("color", "not a colour"))),
        FormRow::of("d", &with(("lane", "team"))),
    ]));
    assert!(read.invalid.is_empty(), "no row is lost for its colour");
    assert_eq!(read.entries.len(), 4);
    assert_eq!(read.entries[0].color, Rgba::from_css("#2a7de1"));
    assert!(read.entries[0].color.is_some());
    assert_eq!(read.entries[1].color, None);
    assert_eq!(read.entries[2].color, None, "unreadable = like none");
    assert_eq!(read.entries[3].color, None, "the lane gives no colour");
    assert_eq!(read.invalid_color, ["c"], "the unreadable one is counted");
}

#[test]
fn what_is_no_colour_is_counted_not_guessed() {
    for bad in ["lane-a", "#12", "300", "transparent", "abc123"] {
        let read = read_bars(&input(vec![FormRow::of("x", &with(("color", bad)))]));
        assert_eq!(read.entries[0].color, None, "{bad:?}");
        assert_eq!(read.invalid_color.len(), 1, "{bad:?}");
    }
}

//! Pictures of `tile-grid` (people C): four people read from `tiles@1.1.0` (the facts `organizations`, `titles`, `emails`, `phones`, each
//! the `pref` value of the module), at two widths (three tiles in a row, then two: the rest wraps), light and dark. Amara has two phones
//! (both under ONE label) and no organization (nothing slides into its place); one tile has no picture (initials). Pictures are kept
//! when `FS_COMP_SHOTS` names a directory (`tile-grid-<wide|narrow>-<light|dark>.png`); they are never part of the repository.
//!
//! Tests: pixel probes on the mounted `View` (no window; they need a graphics adapter).
//!
//! Mutants (each must turn a test red):
//! - the second phone is dropped                                -> `two_values_of_a_fact_draw_more_and_the_dark_page_is_dark`
//! - the empty fact is drawn as a line                          -> `a_missing_fact_leaves_no_line`
//! - the grid does not wrap (always three in a row)             -> `the_narrow_picture_wraps_to_more_rows`
//!
//! Pattern: Test double (a `NoWrite` host and a `say` with the words of the facts) over rows built by `FormRow::from_values`.

#![allow(clippy::pedantic)]

mod common;

use std::sync::Arc;

use common::{background, draw, ink, serial, set_dark};
use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::tile_grid::form_view::View;
use fs_components_words::Locale;

const FACTS: [(&str, &str); 4] = [
    ("organizations", "Organization"),
    ("titles", "Title"),
    ("emails", "E-mail"),
    ("phones", "Phone"),
];

fn person(key: &str, name: &str, values: &[(&str, &str)], number: &str) -> FormRow {
    let map = vec![(
        "facts".to_owned(),
        FACTS.iter().map(|(n, _)| (*n).to_owned()).collect(),
    )];
    FormRow::from_values(key, Vec::new(), &map, values.iter().copied())
        .with_values("label", &[("label", name)])
        .with_values("order", &[("order", number)])
}

fn input(two_phones: bool) -> FormInput {
    let mut amara = vec![("titles", "Engineer"), ("emails", "amara@zolenko.example")];
    amara.push(("phones", "+49 30 1234"));
    if two_phones {
        amara.push(("phones", "+49 171 555 0142"));
    }
    let rows = vec![
        person("amara", "Amara Zolenko", &amara, "01"),
        person(
            "tariq",
            "Tariq Lindqvist",
            &[
                ("organizations", "Helfa"),
                ("titles", "Chair"),
                ("emails", "tariq@lindqvist.example"),
                ("phones", "+46 8 555"),
            ],
            "02",
        ),
        // no picture on file: the face shows initials; the next one has only a name
        person("li", "Li Wei", &[("emails", "li@wei.example")], "03"),
        person("noor", "Noor", &[], "04"),
    ];
    FormInput {
        form: "tiles@1.1.0".to_owned(),
        rows,
        channels: vec![
            ChannelInfo {
                name: "facts".to_owned(),
                fields: FACTS
                    .iter()
                    .map(|(n, _)| FieldInfo {
                        name: (*n).to_owned(),
                        block: "field-text".to_owned(),
                        label: format!("person.{n}"),
                    })
                    .collect(),
                choices: Vec::new(),
            },
            ChannelInfo {
                name: "preview".to_owned(),
                fields: vec![FieldInfo {
                    name: "media".to_owned(),
                    block: "person-face".to_owned(),
                    label: String::new(),
                }],
                choices: Vec::new(),
            },
        ],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(dark: bool) -> DrawCtx {
    DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, dark),
        Arc::new(|k| {
            FACTS
                .iter()
                .find(|(n, _)| k == format!("person.{n}"))
                .map_or_else(|| k.to_owned(), |(_, w)| (*w).to_owned())
        }),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(|| input(false)),
    )
}

fn shot(dark: bool, two_phones: bool, w: u32, h: u32, name: &str) -> image::RgbaImage {
    set_dark(dark);
    let mounted = View.mount(input(two_phones), ctx(dark));
    let img = draw(mounted.build(w as f32, h as f32), w, h, name);
    set_dark(false);
    img
}

#[test]
fn two_values_of_a_fact_draw_more_and_the_dark_page_is_dark() {
    let _g = serial();
    let light = shot(false, true, 900, 420, "tile-grid-wide-light.png");
    let dark = shot(true, true, 900, 420, "tile-grid-wide-dark.png");
    let one = shot(false, false, 900, 420, "tile-grid-wide-one-phone-light.png");
    let bg = background(&light);
    assert!(
        ink(&light, (0, 0, 900, 420), bg) > 4000,
        "the tiles are drawn"
    );
    // Amara's tile is the first of three in the first row: a second phone is a line of ink more
    assert!(
        ink(&light, (0, 0, 300, 210), bg) > ink(&one, (0, 0, 300, 210), bg) + 100,
        "the second phone is drawn"
    );
    assert_ne!(light, dark, "light and dark differ");
    assert!(
        background(&dark)[0] + 80 < background(&light)[0],
        "dark is not dark"
    );
}

#[test]
fn the_narrow_picture_wraps_to_more_rows() {
    let _g = serial();
    // 900 px: three tiles in the first row, the fourth below; 500 px: two and two. The bottom of the picture is empty ground at 900
    // (one row of tiles is short of the second) and drawn at 500, where the second row holds two tiles.
    let wide = shot(false, true, 900, 520, "tile-grid-wide-tall-light.png");
    let narrow = shot(false, true, 500, 520, "tile-grid-narrow-light.png");
    let bg = background(&wide);
    // the second tile of the second row (x 250..500) exists only when the grid wrapped to two in a row
    let second_row = |img: &image::RgbaImage| ink(img, (250, 215, 250, 150), bg);
    assert!(
        second_row(&narrow) > second_row(&wide) + 100,
        "{} against {}",
        second_row(&narrow),
        second_row(&wide)
    );
    shot(true, true, 500, 520, "tile-grid-narrow-dark.png");
}

#[test]
fn a_missing_fact_leaves_no_line() {
    let _g = serial();
    // Noor has no fact at all: her tile (4th, second row of the wide picture, first column) is a name and a face only, so it has less ink
    // than Tariq's with four facts (2nd of the first row)
    let img = shot(false, true, 900, 520, "tile-grid-wide-light-facts.png");
    let bg = background(&img);
    let noor = ink(&img, (0, 215, 300, 150), bg);
    let tariq = ink(&img, (300, 0, 300, 210), bg);
    assert!(noor < tariq, "noor {noor} against tariq {tariq}");
}

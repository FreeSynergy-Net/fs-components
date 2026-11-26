//! `tile-grid`: the tiles read from `tiles@1.0.0` and `tiles@1.1.0` (facts by path, the face by the block of the preview), and the grid
//! of them (as many in a row as fit, the rest wraps).
//!
//! Tests: pure reading tests over `FormInput`; the element is looked at by its ids (`<name>-tile-<key>`, `<name>-row-r<n>`).
//!
//! Mutants (each must turn a test red):
//! - a fact is taken by position, a missing field shifts the rest      -> `a_fact_stands_under_its_own_word_and_a_missing_one_shifts_nothing`
//! - the second value of a fact is dropped                             -> `a_fact_with_two_values_keeps_both_under_one_label`
//! - the preview of another block is drawn as a face                   -> `the_face_follows_the_block_of_the_preview`
//! - the number is always the place, `order` is ignored                -> `the_number_is_order_else_the_place`
//! - the grid keeps one column however narrow or wide it is           -> `as_many_tiles_in_a_row_as_fit_and_the_rest_wraps`

mod common;

use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_widgets::tile_grid::form_view::{FACTS, tiles};
use fs_components_widgets::tile_grid::{Tile, TileGrid};

fn field(name: &str, block: &str) -> FieldInfo {
    FieldInfo {
        name: name.to_owned(),
        block: block.to_owned(),
        label: format!("person.{name}"),
    }
}

fn say(key: &str) -> String {
    key.strip_prefix("person.").map_or_else(
        || key.to_owned(),
        |n| {
            let mut c = n.chars();
            c.next().map_or(String::new(), |f| {
                f.to_uppercase().collect::<String>() + c.as_str()
            })
        },
    )
}

fn input(form: &str, preview_block: &str, rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: form.to_owned(),
        rows,
        channels: vec![
            ChannelInfo {
                name: FACTS.to_owned(),
                fields: vec![
                    field("organizations", "field-text"),
                    field("titles", "field-text"),
                    field("emails", "field-email"),
                    field("phones", "field-phone"),
                ],
                choices: Vec::new(),
            },
            ChannelInfo {
                name: "preview".to_owned(),
                fields: vec![field("media", preview_block)],
                choices: Vec::new(),
            },
        ],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn row(key: &str, values: &[(&str, &str)]) -> FormRow {
    let map = vec![(
        FACTS.to_owned(),
        ["organizations", "titles", "emails", "phones"]
            .map(str::to_owned)
            .to_vec(),
    )];
    FormRow::from_values(key, Vec::new(), &map, values.iter().copied())
        .with_values("label", &[("label", key)])
}

fn labels(tile: &Tile) -> Vec<&str> {
    tile.facts.iter().map(|f| f.label.as_str()).collect()
}

#[test]
fn a_fact_stands_under_its_own_word_and_a_missing_one_shifts_nothing() {
    // `organizations` has no value: the e-mail must not slide under "Organizations"
    let i = input(
        "tiles@1.1.0",
        "person-face",
        vec![row(
            "Amara",
            &[("emails", "amara@example.org"), ("phones", "+234 1 555")],
        )],
    );
    let t = tiles(&i, &say);
    assert_eq!(t.len(), 1);
    assert_eq!(labels(&t[0]), ["Emails", "Phones"]);
    assert_eq!(t[0].facts[1].values, ["+234 1 555"]);
}

#[test]
fn a_fact_with_two_values_keeps_both_under_one_label() {
    let i = input(
        "tiles@1.1.0",
        "person-face",
        vec![row(
            "Amara",
            &[("emails", "a@x.org"), ("emails", "b@x.org")],
        )],
    );
    let t = tiles(&i, &say);
    assert_eq!(labels(&t[0]), ["Emails"]);
    assert_eq!(t[0].facts[0].values, ["a@x.org", "b@x.org"]);
}

#[test]
fn the_old_minor_reads_kind_and_time_as_plain_lines() {
    let map = vec![
        ("kind".to_owned(), vec!["kind".to_owned()]),
        ("time".to_owned(), vec!["time".to_owned()]),
    ];
    let r = FormRow::from_values(
        "shot",
        Vec::new(),
        &map,
        [("kind", "screenshot"), ("time", "2026-11-26")],
    )
    .with_values("label", &[("label", "Capture")]);
    let mut i = input("tiles@1.0.0", "", vec![r]);
    i.channels.clear();
    let t = tiles(&i, &say);
    assert_eq!(labels(&t[0]), ["", ""]);
    assert_eq!(t[0].facts[0].values, ["screenshot"]);
    assert_eq!(t[0].facts[1].values, ["2026-11-26"]);
}

#[test]
fn the_number_is_order_else_the_place() {
    let with_order = row("b", &[]).with_values("order", &[("order", "7")]);
    let i = input("tiles@1.1.0", "", vec![row("a", &[]), with_order]);
    let t = tiles(&i, &say);
    assert_eq!(t[0].number, "01");
    assert_eq!(t[1].number, "7");
}

#[test]
fn the_face_follows_the_block_of_the_preview() {
    let with_image = |block: &str| {
        let r = row("a", &[]).with_values("preview", &[("media", "data:image/png;base64,AAAA")]);
        tiles(&input("tiles@1.1.0", block, vec![r]), &say).remove(0)
    };
    let face = with_image("person-face");
    assert!(face.face && face.image.is_some());
    // no block said: the face is the safe choice (initials, never a broken picture)
    assert!(with_image("").face);
    // another block draws its own preview; this element does not pretend to
    let other = with_image("file-preview");
    assert!(!other.face && other.image.is_none());
}

#[test]
fn a_tile_without_a_preview_is_a_face_without_a_picture() {
    let t = tiles(
        &input("tiles@1.1.0", "person-face", vec![row("Amara", &[])]),
        &say,
    );
    assert!(t[0].face && t[0].image.is_none(), "initials stand in");
}

fn many(n: usize) -> Vec<Tile> {
    (0..n)
        .map(|i| Tile::new(format!("k{i}"), format!("Tile {i}")).numbered(format!("{:02}", i + 1)))
        .collect()
}

#[test]
fn as_many_tiles_in_a_row_as_fit_and_the_rest_wraps() {
    let _g = common::serial();
    let grid = TileGrid::new(many(5));
    assert_eq!(grid.columns(800.0).0, 3);
    assert_eq!(grid.columns(500.0).0, 2);
    assert_eq!(grid.columns(100.0).0, 1, "never fewer than one");
    for (room, rows) in [(800.0, 2usize), (500.0, 3), (100.0, 5)] {
        let tree = common::layout(grid.build(room), room, 2000.0);
        let has = |id: &str| tree.element_registry().get(id).is_some();
        assert!(has(&format!("tiles-row-r{}", rows - 1)), "room {room}");
        assert!(!has(&format!("tiles-row-r{rows}")), "room {room}");
        for i in 0..5 {
            assert!(has(&format!("tiles-tile-k{i}")), "room {room}, tile {i}");
        }
    }
}

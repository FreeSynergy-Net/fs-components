//! `meter`: the zone rule (after HTML's `meter`), role and ARIA values, the value text with the zone in words, the zone class, the
//! fill, the words in two languages, no tab stop.
//!
//! Tests: model tests over `zone`; attribute reads on the built element.

mod common;

use blinc_layout::prelude::RenderTree;
use common::{attr, layout_with, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::meter::{Labels, Meter, STYLES, TEXTS, Zone, zone};
use fs_components_widgets::slider::Scale;
use fs_components_words::{Locale, Localized};

fn show(m: &Meter) -> RenderTree {
    layout_with(m.build(200.0), 400.0, 60.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the meter");
    tree.element_registry().has_class(node, class)
}

#[test]
fn the_zone_is_good_in_the_region_of_the_optimum_fair_next_to_it_and_poor_far_from_it() {
    // low 30, high 70, optimum in the low region (low is good, e.g. a temperature or a disk fill)
    assert_eq!(zone(10.0, 30.0, 70.0, 5.0), Zone::Good);
    assert_eq!(zone(50.0, 30.0, 70.0, 5.0), Zone::Fair);
    assert_eq!(zone(90.0, 30.0, 70.0, 5.0), Zone::Poor);
    // optimum in the high region (high is good, e.g. a score)
    assert_eq!(zone(90.0, 30.0, 70.0, 95.0), Zone::Good);
    assert_eq!(zone(50.0, 30.0, 70.0, 95.0), Zone::Fair);
    assert_eq!(zone(10.0, 30.0, 70.0, 95.0), Zone::Poor);
    // optimum in the middle: both ends are fair
    assert_eq!(zone(50.0, 30.0, 70.0, 50.0), Zone::Good);
    assert_eq!(zone(10.0, 30.0, 70.0, 50.0), Zone::Fair);
    assert_eq!(zone(90.0, 30.0, 70.0, 50.0), Zone::Fair);
    // the edges belong to the middle region
    assert_eq!(zone(30.0, 30.0, 70.0, 50.0), Zone::Good);
    assert_eq!(zone(70.0, 30.0, 70.0, 50.0), Zone::Good);
    // cuts given the wrong way round are put right
    assert_eq!(zone(10.0, 70.0, 30.0, 5.0), Zone::Good);
}

#[test]
fn every_word_is_in_german_and_english_and_names_the_zone() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(
        (de.good.as_str(), de.fair.as_str(), de.poor.as_str()),
        ("gut", "mittel", "schlecht")
    );
    let en = Labels::default();
    assert_eq!(en.of(Zone::Good), "good");
    assert_eq!(en.of(Zone::Fair), "fair");
    assert_eq!(en.of(Zone::Poor), "poor");
}

#[test]
fn the_meter_is_a_named_meter_with_values_and_the_zone_in_words() {
    let _g = serial();
    let m = Meter::new(Scale::new(0.0, 100.0, 1.0), "Disk space", 55.0)
        .id("mt")
        .zones(30.0, 70.0, 5.0);
    let tree = show(&m);
    assert_eq!(attr(&tree, "mt", "role").as_deref(), Some("meter"));
    assert_eq!(
        attr(&tree, "mt", "aria-label").as_deref(),
        Some("Disk space")
    );
    assert_eq!(attr(&tree, "mt", "aria-valuenow").as_deref(), Some("55"));
    assert_eq!(attr(&tree, "mt", "aria-valuemin").as_deref(), Some("0"));
    assert_eq!(attr(&tree, "mt", "aria-valuemax").as_deref(), Some("100"));
    assert_eq!(
        attr(&tree, "mt", "aria-valuetext").as_deref(),
        Some("55, fair")
    );
    assert_eq!(
        attr(&tree, "mt", "tabindex"),
        None,
        "a meter is read, not set"
    );
    let de = Meter::new(Scale::new(0.0, 100.0, 1.0), "Platte", 90.0)
        .id("md")
        .zones(30.0, 70.0, 5.0)
        .labels(&Labels::for_locale(&Locale::parse("de").unwrap()));
    assert_eq!(
        attr(&show(&de), "md", "aria-valuetext").as_deref(),
        Some("90, schlecht")
    );
}

#[test]
fn the_zone_class_follows_the_value_and_without_cuts_it_is_always_good() {
    let _g = serial();
    let at = |v: f64| {
        let m = Meter::new(Scale::new(0.0, 100.0, 1.0), "x", v)
            .id("mz")
            .zones(30.0, 70.0, 5.0);
        let tree = show(&m);
        (
            has(&tree, "mz", "fs-meter-good"),
            has(&tree, "mz", "fs-meter-fair"),
            has(&tree, "mz", "fs-meter-poor"),
            m.zone(),
        )
    };
    assert_eq!(at(10.0), (true, false, false, Zone::Good));
    assert_eq!(at(50.0), (false, true, false, Zone::Fair));
    assert_eq!(at(90.0), (false, false, true, Zone::Poor));
    let plain = Meter::new(Scale::new(0.0, 100.0, 1.0), "x", 99.0).id("mp");
    assert_eq!(plain.zone(), Zone::Good);
    assert!(has(&show(&plain), "mp", "fs-meter-good"));
}

#[test]
fn the_fill_is_the_fraction_of_the_width_and_a_value_off_the_scale_is_held_to_it() {
    let _g = serial();
    let width_of = |v: f64| {
        let m = Meter::new(Scale::new(0.0, 200.0, 1.0), "x", v).id("mf");
        let tree = show(&m);
        let node = tree.query_by_id("mf-fill").expect("the fill");
        tree.get_absolute_bounds(node).expect("bounds").width
    };
    assert!((width_of(100.0) - 100.0).abs() < 1.0, "half of 200 pixels");
    assert!((width_of(500.0) - 200.0).abs() < 1.0, "held to the end");
    assert!(width_of(-5.0).abs() < 1.0, "held to the start");
}

#[test]
fn cuts_given_the_wrong_way_round_and_a_value_off_the_scale_are_put_right() {
    assert_eq!(
        zone(50.0, 70.0, 30.0, 5.0),
        Zone::Fair,
        "the cuts are ordered before the regions are found"
    );
    let _g = serial();
    let m = Meter::new(Scale::new(0.0, 200.0, 1.0), "x", 500.0).id("mc");
    assert_eq!(
        attr(&show(&m), "mc", "aria-valuenow").as_deref(),
        Some("200"),
        "held to the end"
    );
    let m = Meter::new(Scale::new(0.0, 200.0, 1.0), "x", -4.0).id("md");
    assert_eq!(attr(&show(&m), "md", "aria-valuenow").as_deref(), Some("0"));
}

//! The person picker as pictures: with hits, nobody, nothing found, loading, failed, "assign to me" without a card; German and
//! English, light and dark. Pictures are kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: pixel tests over every surface; ids of the surfaces in the laid-out tree.

mod common;

use common::{background, draw, ink, layout, serial, set_dark};
use fs_components_widgets::person_picker::{
    Candidate, FakeSource, Labels, PersonPicker, PersonRef, Script,
};
use fs_components_words::{Locale, Localized};
use std::sync::Arc;

fn r(id: &str) -> PersonRef {
    PersonRef::new(id)
}

fn people() -> Vec<Candidate> {
    vec![
        Candidate::new(r("p-ada"))
            .named("Ada Lovelace")
            .detailed("ada@example.org"),
        Candidate::new(r("p-bob")).named("Bob Marley"),
        Candidate::new(r("p-hid")),
    ]
}

fn picker(fake: &FakeSource, id: &str, lang: &str) -> PersonPicker {
    PersonPicker::new(Arc::new(fake.clone()))
        .id(id)
        .labels(Labels::for_locale(&Locale::parse(lang).unwrap()))
        .choose(vec![r("p-bob")])
}

#[test]
fn every_surface_draws_and_the_ids_are_there() {
    let _g = serial();
    let full = FakeSource::new(people());
    let p = picker(&full, "pa", "en");
    let tree = layout(p.build(), 520.0, 420.0);
    for id in [
        "pa-row-p-ada",
        "pa-row-p-bob",
        "pa-row-p-hid",
        "pa-me",
        "pa-confirm",
        "pa-cancel",
    ] {
        assert!(tree.query_by_id(id).is_some(), "{id}");
    }
    let empty = FakeSource::new(vec![]);
    let tree = layout(picker(&empty, "pb", "en").build(), 520.0, 420.0);
    assert!(
        tree.query_by_id("pb-nothing").is_some(),
        "nobody at all is said"
    );
    let q = picker(&full, "pc", "en");
    q.set_query("zzz");
    let tree = layout(q.build(), 520.0, 420.0);
    assert!(
        tree.query_by_id("pc-none").is_some(),
        "nobody found is said"
    );
    let missing = FakeSource::missing("person_directory_v1");
    let tree = layout(picker(&missing, "pd", "en").build(), 520.0, 420.0);
    assert!(
        tree.query_by_id("pd-error").is_some(),
        "a missing source is shown, not an empty list"
    );
    assert!(tree.query_by_id("pd-error-text").is_some());
    let pending = FakeSource::new(people());
    pending.set_script(Script::Pending);
    let tree = layout(picker(&pending, "pe", "en").build(), 520.0, 420.0);
    assert!(tree.query_by_id("pe-loading").is_some());
    let me = picker(&full, "pf", "en");
    me.choose_me();
    let tree = layout(me.build(), 520.0, 420.0);
    assert!(
        tree.query_by_id("pf-notice").is_some(),
        "no own card is said"
    );
}

#[test]
fn the_pictures_in_both_languages_and_schemes() {
    let _g = serial();
    let full = FakeSource::new(people());
    let missing = FakeSource::missing("person_directory_v1");
    for dark in [false, true] {
        set_dark(dark);
        for lang in ["de", "en"] {
            let scheme = if dark { "dark" } else { "light" };
            let hits = draw(
                picker(&full, "ph", lang).build(),
                520,
                360,
                &format!("picker-hits-{scheme}-{lang}.png"),
            );
            assert!(ink(&hits, (0, 0, 520, 360), background(&hits)) > 6000);
            let q = picker(&full, "pn", lang);
            q.set_query("zzz");
            draw(
                q.build(),
                520,
                360,
                &format!("picker-none-{scheme}-{lang}.png"),
            );
            let bad = draw(
                picker(&missing, "pm", lang).build(),
                520,
                360,
                &format!("picker-error-{scheme}-{lang}.png"),
            );
            assert!(ink(&bad, (0, 0, 520, 360), background(&bad)) > 3000);
            assert_ne!(hits, bad);
        }
    }
    set_dark(false);
}

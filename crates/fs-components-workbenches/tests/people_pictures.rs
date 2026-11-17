//! Pictures of the contacts look: the list with round faces, the centred person with cards, choosing, no hit, nobody.
//!
//! Tests: pixel tests, drawn offscreen (skipped with a message without a graphics adapter). With `FS_COMP_SHOTS=<dir>` the pictures
//! are kept (`people-contacts-<light|dark>-<state>.png`); `FS_RENDER_SCHEME=dark` draws the dark scheme. They are for the eye of
//! the person who decides; the tests only check that each state looks different from the plain detail.
//!
//! Mutants (each must turn a test red):
//! - the mode of choosing draws nothing (no check circles)  -> `pictures_of_the_contacts_look_differ_by_state`
//! - an empty list draws the same as a list with people     -> `pictures_of_the_contacts_look_differ_by_state`

#[path = "people_common/mod.rs"]
mod people_common;

use std::path::PathBuf;
use std::sync::Arc;

use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::Book;

fn widget(book: Book) -> (Arc<Book>, PeopleWidget) {
    let book = Arc::new(book);
    (
        book.clone(),
        PeopleWidget::new(book, PeopleLabels::default()),
    )
}

fn draw(widget: &PeopleWidget, name: &str) -> Option<image::RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-people-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let canvas = Canvas::new(900, 600, widget.styles());
    match draw_to_png(&canvas, |w, h| widget.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
            Some(picture)
        }
    }
}

fn scheme() -> &'static str {
    if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        "dark"
    } else {
        "light"
    }
}

#[test]
fn pictures_of_the_contacts_look_differ_by_state() {
    let _one = people_common::harness::serial();
    let (_, w) = widget(Book::with(&["ada", "bob", "clara"]));
    let shot =
        |w: &PeopleWidget, what: &str| draw(w, &format!("people-contacts-{}-{what}.png", scheme()));
    let Some(detail) = shot(&w, "detail") else {
        return;
    };
    w.with_state(|s| {
        s.toggle_marking();
        s.toggle_mark("bob");
    });
    let marking = shot(&w, "marking").unwrap();
    w.with_state(|s| {
        s.toggle_marking();
        s.search("zzz");
    });
    let nobody = shot(&w, "search-empty").unwrap();
    let (_, empty) = widget(Book::with(&[]));
    let none = shot(&empty, "empty").unwrap();
    for (a, b, what) in [
        (&detail, &marking, "choosing"),
        (&detail, &nobody, "a search without a hit"),
        (&detail, &none, "no people"),
        (&nobody, &none, "two kinds of empty"),
    ] {
        assert_ne!(a, b, "{what} looks like the plain detail");
    }
}

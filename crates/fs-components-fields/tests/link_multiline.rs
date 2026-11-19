//! The fields `link` and `multilineText`: the displayed link (role, underline, Enter and click once each, no
//! scheme that runs something), the many-line input (a line break stays, the value is the typed text), and the
//! contrast of the link ink in every style, light and dark.
//!
//! Tests: raw key events and the pointer through the window's own router; the ink is measured in a drawn picture.
//! Mutants (definitions `mutanten/fs-components/definitionen/mut-f1a.py`): the scheme list, the `href` of a bad link, the
//! first-line rule of the compact line, `role` link, Enter doubling with a click, the ink token of the link.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, draw, has_fonts, layout, serial};
use fs_components_fields::{
    FieldInput, Follow, KeysAsText, Kind, LINK_CONTRAST, Look, Say, compact_with, display_with,
    link_ink,
};
use fs_components_styles::StyleId;

type Seen = Arc<Mutex<Vec<String>>>;

fn follow(seen: &Seen) -> Follow {
    let s = Arc::clone(seen);
    Some(Arc::new(move |href: &str| {
        s.lock().unwrap().push(href.to_owned())
    }))
}

fn parts(text: &str) -> Vec<String> {
    vec![text.to_owned()]
}

#[test]
fn a_good_link_is_a_link_with_role_and_opens_once_by_enter_and_once_by_click() {
    let _g = serial();
    let seen = Seen::default();
    let look = Look::new(StyleId::Admin, false);
    let ui = display_with(
        Kind::Link,
        &parts("https://example.org/a"),
        &look,
        &KeysAsText,
        &follow(&seen),
    );
    let mut tree = layout(ui, 420.0, 120.0);
    assert_eq!(
        attr(&tree, "field-link-open", "role").as_deref(),
        Some("link")
    );
    assert_eq!(
        attr(&tree, "field-link-open", "href").as_deref(),
        Some("https://example.org/a")
    );
    let mut input = Input::new();
    assert!(
        input.tab_to(&mut tree, "field-link-open"),
        "the link is a tab stop"
    );
    input.key(&mut tree, 13);
    assert_eq!(
        *seen.lock().unwrap(),
        ["https://example.org/a"],
        "Enter follows once, not twice"
    );
    input.key(&mut tree, 32);
    assert_eq!(
        seen.lock().unwrap().len(),
        2,
        "known limit: Blinc turns Space into a click on the focused element too, so Space follows once (APG: Enter only)"
    );
    input.click(&mut tree, "field-link-open");
    assert_eq!(seen.lock().unwrap().len(), 3, "a click follows once");
}

#[test]
fn a_scheme_that_runs_something_is_shown_as_text_and_opens_nothing() {
    let _g = serial();
    let seen = Seen::default();
    let look = Look::new(StyleId::Admin, false);
    for bad in [
        "javascript:alert(1)",
        "data:text/html,x",
        "example.org",
        "https://",
    ] {
        let ui = display_with(Kind::Link, &parts(bad), &look, &KeysAsText, &follow(&seen));
        let tree = layout(ui, 420.0, 120.0);
        assert_eq!(
            attr(&tree, "field-link-open", "role"),
            None,
            "{bad}: no link"
        );
        let compact = compact_with(Kind::Link, &parts(bad), &look, &follow(&seen));
        let tree2 = layout(compact, 420.0, 60.0);
        assert_eq!(
            attr(&tree2, "field-link-open-compact", "role"),
            None,
            "{bad}"
        );
    }
    assert!(seen.lock().unwrap().is_empty(), "nothing was opened");
}

#[test]
fn the_compact_link_is_a_link_too() {
    let _g = serial();
    let seen = Seen::default();
    let look = Look::new(StyleId::Admin, false);
    let ui = compact_with(
        Kind::Link,
        &parts("mailto:ada@example.org"),
        &look,
        &follow(&seen),
    );
    let mut tree = layout(ui, 420.0, 60.0);
    assert_eq!(
        attr(&tree, "field-link-open-compact", "role").as_deref(),
        Some("link")
    );
    Input::new().click(&mut tree, "field-link-open-compact");
    assert_eq!(*seen.lock().unwrap(), ["mailto:ada@example.org"]);
}

#[test]
fn the_many_line_input_keeps_its_line_breaks() {
    let _g = serial();
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let look = Look::new(StyleId::Admin, false);
    let changes = Seen::default();
    let seen = Arc::clone(&changes);
    let input = FieldInput::new(Kind::Multiline, look, say)
        .name("notes")
        .value("one\r\ntwo\nthree")
        .on_change(move |t| seen.lock().unwrap().push(t.to_owned()));
    assert_eq!(
        input.parts(),
        ["one\ntwo\nthree"],
        "a pasted CRLF is a line break"
    );
    assert_eq!(input.text(), "one\ntwo\nthree");
    assert_eq!(input.problem(), None);
    let _tree = layout(input.build(), 420.0, 240.0);
    assert!(
        fs_components_fields::Kind::Multiline.field().multiline(),
        "the type asks for a text area"
    );
}

/// The link ink on the ground of a display, in every style, light and dark: at least 4.5:1, the ink drawn is the ink of the
/// token (counter: the plain text colour differs from the link colour in the picture).
#[test]
fn the_link_ink_reads_in_every_style_light_and_dark() {
    let _g = serial();
    if !has_fonts() {
        return;
    }
    for id in StyleId::ALL {
        for dark in [false, true] {
            let look = Look::new(id, dark);
            let pal = look.palette();
            let ink = link_ink(pal);
            for (what, ground) in [("ground", pal.ground), ("panel", pal.panel)] {
                assert!(
                    ink.contrast(ground) >= LINK_CONTRAST,
                    "{} dark={dark}: link ink on the {what}",
                    id.name()
                );
            }
            // counter: the bare token is the reason for `link_ink` (Apple's blue reads at 3.5:1 on its ground)
            if id == StyleId::Apple && !dark {
                assert!(pal.accent_text.contrast(pal.ground) < LINK_CONTRAST);
                assert_ne!(ink, pal.accent_text);
            }
            let name = format!(
                "link-{}-{}.png",
                id.name(),
                if dark { "dark" } else { "light" }
            );
            let ui = display_with(
                Kind::Link,
                &parts("https://example.org/page"),
                &look,
                &KeysAsText,
                &None,
            );
            let page = blinc_layout::div::div()
                .w(420.0)
                .h(90.0)
                .bg(Look::color(pal.ground))
                .child(ui);
            let img = draw(page, 420, 90, &name);
            let want = [ink.r, ink.g, ink.b];
            // underline: the link is told by more than its colour: a run of ink as wide as a word lies in one row
            // (the letters alone make runs of about 12 pixels)
            let ground = [pal.ground.r, pal.ground.g, pal.ground.b];
            let far = |p: &image::Rgba<u8>| {
                (0..3)
                    .map(|i| i32::from(p[i]).abs_diff(i32::from(ground[i])))
                    .sum::<u32>()
                    > 60
            };
            let longest = (0..img.height())
                .map(|y| {
                    let (mut run, mut best) = (0u32, 0u32);
                    for x in 0..img.width() {
                        run = if far(img.get_pixel(x, y)) { run + 1 } else { 0 };
                        best = best.max(run);
                    }
                    best
                })
                .max()
                .unwrap_or(0);
            assert!(
                longest >= 80,
                "{name}: no underline (longest run {longest})"
            );
            let found = img.pixels().any(|p| {
                let d = |a: u8, b: u8| i32::from(a).abs_diff(i32::from(b));
                d(p[0], want[0]) + d(p[1], want[1]) + d(p[2], want[2]) < 24
            });
            assert!(found, "{name}: no pixel of the link ink");
        }
    }
}

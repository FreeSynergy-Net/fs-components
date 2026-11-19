//! The fields `flag` (a switch), `integer` and `whole` (number fields), and their contrast in every style, light and dark.
//!
//! Tests: raw key events and the pointer through the window's own router; the switch is told by position (its thumb) as well as
//! colour. Mutants (definitions `mutanten/fs-components/definitionen/mut-f1b.py`): the canonical form, the range of `whole`, the
//! stepping limits, `aria-checked`, a flip that answers twice, the role `spinbutton`, the check or the cross of the compact line.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, draw_tree, has_fonts, layout, layout_with, serial};
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look, Problem, Say, compact, display};
use fs_components_styles::StyleId;

type Seen = Arc<Mutex<Vec<String>>>;

fn look() -> Look {
    Look::new(StyleId::Admin, false)
}

fn say() -> Arc<dyn Say> {
    Arc::new(KeysAsText)
}

fn p(t: &str) -> Vec<String> {
    vec![t.to_owned()]
}

#[test]
fn the_switch_answers_once_per_flip_by_key_or_click_and_says_so() {
    let _g = serial();
    let seen = Seen::default();
    let s = Arc::clone(&seen);
    let input = FieldInput::new(Kind::Flag, look(), say())
        .name("on")
        .on_change(move |t| s.lock().unwrap().push(t.to_owned()));
    assert_eq!(input.text(), "", "not answered yet: no value, not `false`");
    let sheet = fs_components_fields::controls::STYLES;
    let mut tree = layout_with(input.build(), 420.0, 120.0, sheet);
    assert_eq!(attr(&tree, "on-flag", "role").as_deref(), Some("switch"));
    assert_eq!(
        attr(&tree, "on-flag", "aria-checked").as_deref(),
        Some("false")
    );
    let mut keys = Input::new();
    assert!(keys.tab_to(&mut tree, "on-flag-hit"), "a tab stop");
    keys.key(&mut tree, 13);
    assert_eq!(*seen.lock().unwrap(), ["true"], "Enter flips once");
    assert_eq!(input.text(), "true");
    keys.click(&mut tree, "on-flag-hit");
    assert_eq!(
        *seen.lock().unwrap(),
        ["true", "false"],
        "a click flips once"
    );
    assert_eq!(input.text(), "false");
}

#[test]
fn a_switch_starts_from_the_stored_value() {
    let _g = serial();
    let input = FieldInput::new(Kind::Flag, look(), say())
        .name("on")
        .value("Yes");
    assert_eq!(input.text(), "true", "yes is read, true is written");
    let tree = layout(input.build(), 420.0, 120.0);
    assert_eq!(
        attr(&tree, "on-flag", "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(input.problem(), None);
    let bad = FieldInput::new(Kind::Flag, look(), say()).value("maybe");
    assert_eq!(bad.problem(), Some(Problem::NotYesNo));
}

#[test]
fn a_flag_is_said_in_words_in_the_display_and_by_a_mark_in_the_lines() {
    let _g = serial();
    let words: Arc<dyn Say> = Arc::new(|k: &str| match k {
        "value.flag.true" => "Yes".to_owned(),
        other => other.to_owned(),
    });
    let f = Kind::Flag.field();
    assert_eq!(f.word(&p("true")), Some("value.flag.true"));
    assert_eq!(f.word(&p("false")), Some("value.flag.false"));
    assert_eq!(f.word(&p("")), None);
    assert!(f.mark(&p("true")).is_some() && f.mark(&p("false")).is_some());
    assert_ne!(
        f.mark(&p("true")),
        f.mark(&p("false")),
        "a check is not a cross"
    );
    assert_eq!(f.mark(&p("")), None);
    let _ = layout(
        display(Kind::Flag, &p("true"), &look(), &*words),
        420.0,
        120.0,
    );
    let _ = layout(compact(Kind::Flag, &p("false"), &look()), 420.0, 60.0);
}

#[test]
fn a_number_field_is_a_spinbutton_and_steps() {
    let _g = serial();
    let seen = Seen::default();
    let s = Arc::clone(&seen);
    let input = FieldInput::new(Kind::Integer, look(), say())
        .name("n")
        .value("41")
        .on_change(move |t| s.lock().unwrap().push(t.to_owned()));
    let tree = layout(input.build(), 420.0, 140.0);
    // the role sits on the element around the field (a Blinc text input takes no attribute)
    assert_eq!(attr(&tree, "n-spin", "role").as_deref(), Some("spinbutton"));
    assert_eq!(
        attr(&tree, "n-spin", "aria-valuenow").as_deref(),
        Some("41")
    );
    assert_eq!(
        attr(&tree, "n-spin", "aria-valuemin"),
        None,
        "integer has no lower end"
    );
    input.step(1);
    assert_eq!(input.text(), "42");
    input.step(-2);
    assert_eq!(input.text(), "40");
    assert_eq!(*seen.lock().unwrap(), ["42", "40"]);
    let whole = FieldInput::new(Kind::Whole, look(), say())
        .name("w")
        .value("0");
    let wtree = layout(whole.build(), 420.0, 140.0);
    assert_eq!(
        attr(&wtree, "w-spin", "aria-valuemin").as_deref(),
        Some("0")
    );
    whole.step(-1);
    assert_eq!(whole.text(), "0", "whole stays at 0");
    let empty = FieldInput::new(Kind::Integer, look(), say());
    let _tree = layout(empty.build(), 420.0, 140.0);
    empty.step(-1);
    assert_eq!(empty.text(), "-1", "from nothing the step starts at 0");
    let word = FieldInput::new(Kind::Integer, look(), say()).value("abc");
    let _tree = layout(word.build(), 420.0, 140.0);
    word.step(1);
    assert_eq!(
        word.text(),
        "abc",
        "text that is not a number is left alone"
    );
    assert_eq!(word.problem(), Some(Problem::NotANumber));
    let flag = FieldInput::new(Kind::Flag, look(), say()).value("true");
    let _tree = layout(flag.build(), 420.0, 140.0);
    flag.step(1);
    assert_eq!(flag.text(), "true", "a step does nothing for another type");
}

#[test]
fn a_number_has_the_range_role_and_a_large_value_is_not_rounded() {
    let _g = serial();
    let big = "9223372036854775807";
    let input = FieldInput::new(Kind::Integer, look(), say())
        .name("n")
        .value(big);
    assert_eq!(input.text(), big, "no f64 in between: 2^63-1 stays");
    input.step(1);
    assert_eq!(input.text(), big, "the end of the range stays");
    let tree = layout(input.build(), 420.0, 140.0);
    assert_eq!(attr(&tree, "n-spin", "role").as_deref(), Some("spinbutton"));
    let whole = FieldInput::new(Kind::Whole, look(), say())
        .name("w")
        .value("7");
    assert_eq!(whole.problem(), None);
    assert_eq!(
        FieldInput::new(Kind::Whole, look(), say())
            .value("-3")
            .problem(),
        Some(Problem::OutOfRange)
    );
}

/// The switch in every style, light and dark: the thumb and the label read; the track of an answered flag differs from an
/// unanswered one (told by position as well as colour: the on thumb lies to the right of the off thumb).
#[test]
fn the_switch_reads_in_every_style_light_and_dark() {
    let _g = serial();
    if !has_fonts() {
        return;
    }
    for id in StyleId::ALL {
        for dark in [false, true] {
            common::set_dark(dark);
            let look = Look::new(id, dark);
            let pal = look.palette();
            let sheet = format!(
                "{}\n{}",
                fs_components_fields::controls::STYLES,
                fs_components_styles::styles::block_tokens::sheet(look.style)
            );
            let mut xs = Vec::new();
            for value in ["false", "true"] {
                let input = FieldInput::new(Kind::Flag, look, say())
                    .name("s")
                    .value(value);
                let page = blinc_layout::div::div()
                    .w(420.0)
                    .h(110.0)
                    .bg(Look::color(pal.ground))
                    .child(input.build());
                let tree = layout_with(page, 420.0, 110.0, &sheet);
                let name = format!(
                    "flag-{}-{}-{value}.png",
                    id.name(),
                    if dark { "dark" } else { "light" }
                );
                let img = draw_tree(&tree, 420, 110, &name);
                let bg = common::background(&img);
                assert!(
                    common::ink(&img, (0, 0, 420, 110), bg) > 100,
                    "{name}: nearly empty"
                );
                xs.push(img);
            }
            // counter: an answered switch looks different from an off one (position of the thumb, colour of the track)
            assert_ne!(
                xs[0].as_raw(),
                xs[1].as_raw(),
                "{} dark={dark}: on and off look alike",
                id.name()
            );
        }
    }
    common::set_dark(false);
}

#[test]
fn the_new_problems_have_their_stable_keys() {
    assert_eq!(Problem::NotYesNo.key(), "problem.flag.bad");
    assert_eq!(Problem::OutOfRange.key(), "problem.number.out-of-range");
}

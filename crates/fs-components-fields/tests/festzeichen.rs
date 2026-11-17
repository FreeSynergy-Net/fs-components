//! P1 probe (no release): can Blinc carry ONE input `local @ server . tld` whose `@` and last `.` are fixed signs drawn in the field,
//! with one focus and keyboard model? Keys go in without a screen (the window's own router and key broadcast). Each K case is a check
//! function: it must pass on the real field and fail on its mutant. The clipboard of the desktop is never touched: Ctrl+C/Ctrl+V do not
//! go through the tree (Blinc writes the system clipboard there); the field's own copy and paste logic is called directly.
//!
//! Tests: K1..K9 and their mutants M1..M8, headless. A picture (empty, filled, error) when `FS_COMP_SHOTS` is set and an adapter exists.
#![allow(clippy::pedantic, dead_code)]

#[path = "festzeichen/address.rs"]
mod address;
#[path = "festzeichen/cases.rs"]
mod cases;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;
#[path = "festzeichen/ghost.rs"]
mod ghost;
#[path = "festzeichen/keys.rs"]
mod keys;
#[path = "festzeichen/split.rs"]
mod split;
#[path = "festzeichen/typist.rs"]
mod typist;
#[path = "festzeichen/view.rs"]
mod view;

use address::{AddressField, Mutation, Mutation::*};
use std::sync::Arc;
use typist::Typist;

const TLDS: &[&str] = &["org", "com", "de", "example"];

use cases::{
    k1_at_jumps, k2_typed_addresses, k3_server_with_dots, k4_mid_dot, k5_backspace,
    k6_backspace_at_start, k7_arrows, k8_paste, k9_copy, k10_only_text, k11_second_at,
};
use ghost::{
    k12_typing_overwrites_ghost, k13_accept_ghost_sign, k14_accept_ending, k15_copy_without_ghost,
    k16_end_enter_tab,
};

type Check = fn(Mutation) -> Result<(), String>;

fn ok(cond: bool, what: impl Into<String>) -> Result<(), String> {
    if cond { Ok(()) } else { Err(what.into()) }
}

fn parts(f: &AddressField) -> String {
    format!(
        "[{}|{}|{}] focus {:?}",
        f.value(0),
        f.value(1),
        f.value(2),
        f.focus()
    )
}

const CASES: [(&str, Check, Mutation); 16] = [
    ("k1_at_jumps", k1_at_jumps, AtIsText),
    ("k2_typed_addresses", k2_typed_addresses, JoinWithoutDot),
    ("k3_server_with_dots", k3_server_with_dots, TldDotSwallowed),
    ("k4_mid_dot", k4_mid_dot, MidDotJumps),
    ("k5_backspace", k5_backspace, NoBackJump),
    (
        "k6_backspace_at_start",
        k6_backspace_at_start,
        NoBackspaceJump,
    ),
    ("k7_arrows", k7_arrows, NoArrowJump),
    ("k8_paste", k8_paste, FirstDot),
    ("k9_copy", k9_copy, CopyPartOnly),
    ("k10_only_text", k10_only_text, BlincEmailType),
    ("k11_second_at", k11_second_at, AtIsText),
    (
        "k12_typing_overwrites_ghost",
        k12_typing_overwrites_ghost,
        AtIsText,
    ),
    ("k13_accept_ghost_sign", k13_accept_ghost_sign, NoArrowJump),
    ("k14_accept_ending", k14_accept_ending, AcceptDouble),
    (
        "k15_copy_without_ghost",
        k15_copy_without_ghost,
        GhostInCopy,
    ),
    ("k16_end_enter_tab", k16_end_enter_tab, AcceptDouble),
];

#[test]
fn every_case_passes_on_the_real_field() {
    let _g = common::serial();
    let bad: Vec<String> = CASES
        .iter()
        .filter_map(|(n, f, _)| f(Intact).err().map(|e| format!("{n}: {e}")))
        .collect();
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn every_mutant_turns_its_case_red() {
    let _g = common::serial();
    let alive: Vec<String> = CASES
        .iter()
        .filter(|(_, f, m)| f(*m).is_ok())
        .map(|(n, _, m)| format!("{n} survived {m:?}"))
        .collect();
    assert!(alive.is_empty(), "{alive:#?}");
}

#[test]
fn a_click_on_a_part_gives_it_the_keyboard() {
    let _g = common::serial();
    let mut t = Typist::new(Intact);
    t.click("addr-server");
    assert_eq!(
        t.field.focus(),
        Some(1),
        "one click, one part has the keyboard"
    );
    t.text("x");
    assert_eq!(t.field.value(1), "x");
}

#[test]
fn the_field_in_four_states_as_a_picture() {
    let _g = common::serial();
    let mut states = Vec::new();
    for (typed, accept, error) in [
        ("", false, false),
        ("ada@mail.o", false, false),
        ("ada@mail.o", true, false),
        ("ada@localhost", false, true),
    ] {
        let mut t = Typist::new(Intact);
        t.text(typed);
        if accept {
            t.press(13, false);
        }
        states.push((t.field, error));
    }
    let mut ui = blinc_layout::div::div().flex_col().gap(16.0).p(20.0);
    for (f, error) in &states {
        ui = ui.child(Arc::clone(f).view(*error));
    }
    let img = common::draw(ui, 560, 460, "felder-festzeichen.png");
    assert_eq!((img.width(), img.height()), (560, 460));
}

#[test]
fn the_list_of_endings_is_required() {
    assert!(AddressField::new(Intact, &[]).is_err(), "no list, no field");
}

#[test]
fn what_the_window_does_with_tab_and_what_a_screen_reader_gets() {
    let _g = common::serial();
    let mut t = Typist::with(Intact);
    t.text("a@b.o");
    t.window_key(9);
    let seen = t.field.st.lock().unwrap().seen.clone();
    let nodes = t.tree.accessibility_nodes();
    for n in &nodes {
        eprintln!(
            "MEASURE a11y: role {:?} label {:?} value {:?} placeholder {:?}",
            n.role, n.label, n.value, n.placeholder
        );
        eprintln!("MEASURE a11y hint {:?} for {:?}", n.hint, n.label);
    }
    eprintln!(
        "MEASURE tab: frame saw {seen:?}, tld now {:?}",
        t.field.value(2)
    );
    let values: Vec<&str> = nodes.iter().map(|n| n.value.as_str()).collect();
    assert!(
        !values.iter().any(|v| v.contains("rg")),
        "the grey ending must not be a value: {values:?}"
    );
}

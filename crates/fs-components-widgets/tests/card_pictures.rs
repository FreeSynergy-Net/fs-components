//! Pictures of `card` (P3) and of the container's layer (P4): the states normal, hover, held, carried and locked side by side, and a board of
//! two cells with the places READ FROM THE LAID-OUT TREE (`BlincBounds`), the line between two cards and the ghost under the pointer
//! with the reason. Drawn in Admin and Simple, light and dark, each with the theme guard of `styled.rs` (the style is set through
//! `ThemeState`); kept when `FS_COMP_SHOTS` names a directory (`<Style>-<light|dark>-card-states.png`, `...-card-drag.png`).
//!
//! Tests: in every look each state differs from every other (a crop of each card), the four looks differ from each other, the place and
//! the line measured in a real layout are the ones of the pointer, something was drawn.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

mod card_support;
mod common;
#[path = "common/styled.rs"]
mod styled;

use std::sync::Arc;

use blinc_core::BlincContextState;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::{RenderTree, text};
use card_support::{at, card};
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::answer::Reason;
use fs_components_widgets::card::BlincBounds;
use fs_components_widgets::card::layer::{drop_line, ghost, within};
use fs_components_widgets::card::{
    Bounds, Card, DropTargets, MeasuredTargets, Mode, Outcome, Rect, Target,
};
use fs_components_widgets::{Key, KeyEvent};
use styled::{STRIP, SWATCH, draw_styled_tree, ground_of, layout_styled};

const LOOKS: [(StyleId, bool); 4] = [
    (StyleId::Admin, false),
    (StyleId::Admin, true),
    (StyleId::Simple, false),
    (StyleId::Simple, true),
];

fn css() -> String {
    format!(
        "{}\n.cp-line {{ color: var(--text-secondary); font-size: var(--text-xs); }}\n.cp-cell {{ border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); padding: var(--space-2); background: var(--surface); }}",
        fs_components_widgets::styles()
    )
}

fn parts(title: &str, line: &str) -> Vec<Div> {
    vec![
        div()
            .class("fs-card-header")
            .child(text(title.to_owned()).class("fs-card-title")),
        div()
            .class("fs-card-body")
            .child(text(line.to_owned()).class("cp-line")),
    ]
}

/// Any place at all, so a press can become a carry.
struct Anywhere;
impl DropTargets for Anywhere {
    fn target_at(&self, _: (f32, f32), _: &fs_components_widgets::card::CardRef) -> Option<Target> {
        Some(at("doing", None, None))
    }
}

const STATES: [&str; 5] = ["normal", "hover", "held", "carried", "locked"];

fn make(state: &str) -> Card {
    let c = Card::new(format!("cs-{state}"), card(state))
        .at(at("todo", None, None))
        .targets(Arc::new(Anywhere));
    match state {
        "held" => {
            assert_eq!(c.key_press(&KeyEvent::new(Key::Space)), Outcome::Lifted);
            c
        }
        "carried" => {
            c.press((0.0, 0.0));
            c.pointer((50.0, 50.0));
            c
        }
        "locked" => c.locked(Reason::new("agent", "An agent is working on it.")),
        _ => c,
    }
}

fn states_scene() -> Div {
    let mut col = div().id("cs-all").flex_col().gap(12.0).w(320.0);
    for s in STATES {
        let mut d = make(s).build(parts(
            &format!("Write the letter ({s})"),
            "Due on Friday, two people",
        ));
        if s == "hover" {
            d = d.class("fs-card-hover");
        }
        col = col.child(d);
    }
    col
}

fn registry(tree: &RenderTree) {
    BlincContextState::get().set_element_registry(Arc::clone(tree.element_registry()) as blinc_core::AnyElementRegistry);
}

fn not_empty(img: &image::RgbaImage, look: (StyleId, bool)) {
    let ground = ground_of(img);
    let ink = img
        .enumerate_pixels()
        .filter(|(x, y, _)| *x >= STRIP + 4 || *y >= SWATCH + 4)
        .filter(|(_, _, p)| {
            (0..3)
                .map(|i| i32::from(p.0[i].abs_diff(ground[i])))
                .sum::<i32>()
                > 40
        })
        .count();
    assert!(ink > 3000, "{look:?}: nearly empty ({ink} ink pixels)");
}

fn all_differ(seen: &[(String, Vec<u8>)]) {
    for (i, (a, pa)) in seen.iter().enumerate() {
        for (b, pb) in &seen[i + 1..] {
            assert_ne!(pa, pb, "the pictures {a} and {b} are the same");
        }
    }
}

#[test]
fn the_states_of_the_card_look_different_in_four_looks() {
    let _g = serial();
    let mut seen = Vec::new();
    for look in LOOKS {
        let tree = layout_styled(states_scene, 400.0, 760.0, look.0, look.1, &css());
        registry(&tree);
        let img = draw_styled_tree(&tree, 400, 760, look.0, look.1, "card-states.png");
        not_empty(&img, look);
        // each state is a crop of its own card; no two are the same pixels
        let mut crops: Vec<(String, Vec<u8>)> = Vec::new();
        for s in STATES {
            let r: Rect = BlincBounds.bounds(&format!("cs-{s}")).expect("laid out");
            let crop =
                image::imageops::crop_imm(&img, r.x as u32, r.y as u32, r.w as u32, r.h as u32)
                    .to_image();
            crops.push((s.to_owned(), crop.into_raw()));
        }
        all_differ(&crops);
        assert_eq!(make("locked").mode(), Mode::Locked);
        seen.push((format!("{}-{}", look.0.name(), look.1), img.into_raw()));
    }
    all_differ(&seen);
}

// ---- P4: the layer of a container, places read from the laid-out tree -----------------------------------------------------

const POINTER_IN_BOARD: (f32, f32) = (330.0, 118.0);

fn cell(id: &str, cards: &[&str]) -> Div {
    let mut d = div()
        .id(id)
        .class("cp-cell")
        .flex_col()
        .gap(10.0)
        .w(220.0)
        .h(300.0);
    for n in cards {
        let c = Card::new(format!("{id}-{n}"), card(n));
        d = d.child(c.build(parts(&format!("Task {n}"), "A line of text")));
    }
    d
}

/// The board: two cells side by side, and (second pass) the layer with the line and the ghost, in the board's coordinates.
fn board(layer: Option<(Target, Reason, Rect)>, measured: &MeasuredTargets) -> Div {
    let mut b = div()
        .id("board")
        .relative()
        .flex_row()
        .gap(20.0)
        .w(480.0)
        .h(320.0);
    b = b
        .child(cell("c-todo", &["a", "b"]))
        .child(cell("c-doing", &["c", "d"]));
    if let Some((to, why, origin)) = layer {
        let line = within(measured.line(&to).expect("line"), (origin.x, origin.y));
        b = b.child(drop_line("c-line", line, true)).child(ghost(
            "c-ghost",
            parts("Task a", "A line of text"),
            POINTER_IN_BOARD,
            (200.0, 64.0),
            Some(&why),
        ));
    }
    b
}

fn measured() -> MeasuredTargets {
    let mut t = MeasuredTargets::new(Arc::new(BlincBounds))
        .zone(None, "todo", "c-todo")
        .zone(None, "doing", "c-doing")
        .rule(Arc::new(|_, to| {
            if to.column == "doing" {
                Err(Reason::new("wip-full", "Doing is full."))
            } else {
                Ok(())
            }
        }));
    for (col, names) in [("todo", ["a", "b"]), ("doing", ["c", "d"])] {
        for n in names {
            t = t.card(card(n), None, col, &format!("c-{col}-{n}"));
        }
    }
    t
}

#[test]
fn the_place_and_the_line_measured_in_a_real_layout_are_the_ones_of_the_pointer() {
    let _g = serial();
    let mut seen = Vec::new();
    for look in LOOKS {
        let t = measured();
        // pass 1: lay the board out, read where it is, ask the place of the pointer
        let first = layout_styled(|| board(None, &t), 600.0, 400.0, look.0, look.1, &css());
        registry(&first);
        let origin = BlincBounds.bounds("board").expect("board laid out");
        let window = (origin.x + POINTER_IN_BOARD.0, origin.y + POINTER_IN_BOARD.1);
        let carried = card("a");
        let to = t
            .target_at(window, &carried)
            .expect("a place under the pointer");
        // the pointer is in the cell `doing`, between its two cards c and d
        assert_eq!(
            (to.column.as_str(), to.after.clone(), to.before.clone()),
            ("doing", Some(card("c")), Some(card("d"))),
            "{look:?}"
        );
        let why = t.can_drop(&carried, &to).unwrap_err();
        let line = t.line(&to).unwrap();
        let (c, d) = (
            BlincBounds.bounds("c-doing-c").unwrap(),
            BlincBounds.bounds("c-doing-d").unwrap(),
        );
        assert!(
            line.y > c.y + c.h - 0.01 && line.y < d.y + 0.01,
            "{look:?}: the line lies in the gap, {line:?}"
        );
        assert!((line.x - BlincBounds.bounds("c-doing").unwrap().x).abs() < 0.01);
        // pass 2: the same board with the layer
        let tree = layout_styled(
            || board(Some((to.clone(), why.clone(), origin)), &t),
            600.0,
            400.0,
            look.0,
            look.1,
            &css(),
        );
        registry(&tree);
        let ghost_at = BlincBounds.bounds("c-ghost").expect("ghost laid out");
        assert!(
            (ghost_at.x + ghost_at.w / 2.0 - window.0).abs() < 1.0,
            "{look:?}: the ghost is centred under the pointer"
        );
        assert!(
            BlincBounds.bounds("c-ghost-reason").is_some(),
            "{look:?}: the reason is on the ghost"
        );
        let img = draw_styled_tree(&tree, 600, 400, look.0, look.1, "card-drag.png");
        not_empty(&img, look);
        seen.push((format!("{}-{}", look.0.name(), look.1), img.into_raw()));
    }
    all_differ(&seen);
}

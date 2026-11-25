//! Pictures of `grid` with cells that hold elements (P2 of the board-in-grid draft): lanes as spanning parent rows of a treegrid, cards in
//! the cells, rows of different heights, an empty cell kept greifbar by `min_height`, one cell the keys are in. Drawn in the styles Admin and
//! Simple, light and dark, each with the theme guard of `styled.rs` (the style is set through `ThemeState`, never faked); kept when
//! `FS_COMP_SHOTS` names a directory (file names `<Style>-<light|dark>-gridcells-mixed.png`).
//!
//! Tests: the four pictures differ from each other (a style or a scheme that did nothing is red), the lane rows are measured to be
//! of different heights, something was drawn.
//!
//! Pattern: none (tests).

mod common;
#[path = "common/styled.rs"]
mod styled;

use std::collections::HashMap;
use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_theme::SpacingToken;
use common::{serial, size_of};
use fs_components_styles::StyleId;
use fs_components_widgets::grid::{CellAt, CellView, Column, Grid, Row};
use fs_components_widgets::{Key, KeyEvent};
use styled::{STRIP, SWATCH, draw_styled_tree, ground_of, layout_styled};

const W: u32 = 760;
const H: u32 = 1000;
const WF: f32 = 760.0;
const HF: f32 = 1000.0;
const FILE: &str = "gridcells-mixed.png";
const CARD_CSS: &str = ".pc-card { border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); background: var(--surface-elevated); padding: var(--space-2); margin-bottom: var(--space-2); }\n.pc-title { color: var(--text-primary); font-size: var(--text-sm); font-weight: 700; }\n.pc-line { color: var(--text-secondary); font-size: var(--text-xs); }\n.pc-head { color: var(--text-primary); font-size: var(--text-sm); font-weight: 700; }";

/// The cards of a cell: `row/col` -> titles.
struct Cards(HashMap<&'static str, Vec<&'static str>>);

impl CellView for Cards {
    fn cell(&self, at: &CellAt<'_>) -> Option<Div> {
        if at.col == "title" {
            return None;
        }
        let list = self.0.get(format!("{}/{}", at.row, at.col).as_str())?;
        let mut column = div().flex_col().w_full();
        for t in list {
            column = column.child(
                div()
                    .class("pc-card")
                    .flex_col()
                    .child(text(*t).class("pc-title"))
                    .child(text("Due on Friday, two people").class("pc-line")),
            );
        }
        Some(column)
    }
}

fn scene() -> Div {
    let cols = vec![
        Column::new("todo", "To do"),
        Column::new("doing", "Doing"),
        Column::new("done", "Done"),
    ];
    let cells = |r: &str| Row::new(r, vec![String::new(); 3]).min_height(SpacingToken::Space16);
    let anna = Row::spanning("anna", "Anna (4 cards)").with_children(vec![cells("anna-cells")]);
    let ben = Row::spanning("ben", "Ben (3 cards)").with_children(vec![cells("ben-cells")]);
    let none = Row::spanning("none", "Without a person").with_children(vec![cells("none-cells")]);
    let data: HashMap<&'static str, Vec<&'static str>> = HashMap::from([
        (
            "anna-cells/todo",
            vec!["Write the letter", "Buy stamps", "Water the plants"],
        ),
        ("anna-cells/doing", vec!["Call the landlord"]),
        ("ben-cells/todo", vec!["Plan the trip"]),
        ("ben-cells/doing", vec!["Book the train", "Pack the bags"]),
    ]);
    let g = Grid::treegrid(cols, &[anna.opened(), ben.opened(), none.opened()])
        .id("pc")
        .name("Lanes")
        .column_width(236.0)
        .hold_cells()
        .cells(Arc::new(Cards(data)));
    assert!(g.go_to("anna-cells", "doing"));
    assert!(
        g.press(KeyEvent::new(Key::Enter)),
        "the keys go into the cell"
    );
    div().child(g.build())
}

#[test]
fn lanes_of_different_heights_with_cards_in_four_looks() {
    let _g = serial();
    let css = format!("{}\n{CARD_CSS}", fs_components_widgets::styles());
    let mut seen: Vec<(String, Vec<u8>)> = Vec::new();
    for (id, dark) in [
        (StyleId::Admin, false),
        (StyleId::Admin, true),
        (StyleId::Simple, false),
        (StyleId::Simple, true),
    ] {
        let tree = layout_styled(scene, WF, HF, id, dark, &css);
        let (anna, ben, none) = (
            size_of(&tree, "pc-row-anna-cells").1,
            size_of(&tree, "pc-row-ben-cells").1,
            size_of(&tree, "pc-row-none-cells").1,
        );
        assert!(
            anna > ben && ben > none,
            "{id:?}: lane rows differ: {anna} {ben} {none}"
        );
        let head = size_of(&tree, "pc-cell-anna-todo");
        assert_eq!(
            head.0,
            3.0 * 236.0,
            "{id:?}: the lane head spans three columns"
        );
        let img = draw_styled_tree(&tree, W, H, id, dark, FILE);
        let ground = ground_of(&img);
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
        assert!(
            ink > 3000,
            "{id:?} dark={dark}: nearly empty ({ink} ink pixels)"
        );
        seen.push((format!("{}-{dark}", id.name()), img.into_raw()));
    }
    for (i, (a, pa)) in seen.iter().enumerate() {
        for (b, pb) in &seen[i + 1..] {
            assert_ne!(pa, pb, "the pictures {a} and {b} are the same");
        }
    }
}

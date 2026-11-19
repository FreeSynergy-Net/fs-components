//! Pictures of the bundle G3 (listbox, tree, grid, treegrid, table, feed) in a FreeSynergy style, light and dark, with the theme guard
//! of `styled.rs` ([THEME-WAECHTER]); kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: draws every scene in the style Admin, light and dark, and asserts that something was drawn and that the dark picture
//! differs from the light one; the pictures are for a person to look at (rule: pictures before release).

mod common;
#[path = "common/styled.rs"]
mod styled;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use common::serial;
use fs_components_styles::StyleId;
use fs_components_widgets::feed::{Article, Feed};
use fs_components_widgets::grid::{Column, Grid, Row};
use fs_components_widgets::listbox::{Listbox, Opt};
use fs_components_widgets::table::Table;
use fs_components_widgets::tree::Tree;
use fs_components_widgets::tree_view::TreeNode;
use styled::{STRIP, SWATCH, draw_styled, ground_of};

type Scene = (&'static str, u32, u32, fn() -> Div);

const SCENES: [Scene; 6] = [
    ("listbox", 500, 200, listbox),
    ("tree", 500, 260, tree),
    ("grid", 600, 240, grid),
    ("treegrid", 600, 260, treegrid),
    ("table", 600, 200, table),
    ("feed", 600, 280, feed),
];

fn listbox() -> Div {
    let l = Listbox::multiple(vec![
        Opt::new("a", "Alpha"),
        Opt::new("b", "Beta").disabled(),
        Opt::new("c", "Gamma"),
        Opt::new("d", "Delta"),
    ])
    .id("g3-lb")
    .name("Letters");
    l.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Space,
    ));
    l.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Down,
    ));
    div().w(300.0).child(l.build())
}

fn tree() -> Div {
    let mut docs = TreeNode::new("docs", "Documents");
    docs.children = vec![TreeNode::new("a", "Report"), TreeNode::new("b", "Notes")];
    let t = Tree::new(&[
        docs,
        TreeNode::new("pics", "Pictures"),
        TreeNode::new("music", "Music"),
    ])
    .id("g3-tr")
    .name("Folders");
    t.set_open("docs", true);
    t.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Down,
    ));
    t.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Enter,
    ));
    div().w(300.0).child(t.build())
}

fn cols() -> Vec<Column> {
    vec![
        Column::new("n", "Name"),
        Column::new("s", "Size"),
        Column::new("k", "Kind"),
    ]
}

fn row(key: &str, name: &str, size: &str) -> Row {
    Row::new(
        key,
        vec![name.to_owned(), size.to_owned(), "file".to_owned()],
    )
}

fn grid() -> Div {
    let g = Grid::new(
        cols(),
        &[
            row("a", "Report.md", "12 KB"),
            row("b", "Notes.md", "4 KB"),
            row("c", "Plan.md", "9 KB"),
        ],
    )
    .id("g3-gr");
    g.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Down,
    ));
    g.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Right,
    ));
    div().w(460.0).child(g.build())
}

fn treegrid() -> Div {
    let top = row("docs", "Documents", "21 KB").with_children(vec![
        row("a", "Report.md", "12 KB"),
        row("b", "Notes.md", "9 KB"),
    ]);
    let g = Grid::treegrid(cols(), &[top, row("pics", "Pictures", "3 MB")]).id("g3-tg");
    g.set_open("docs", true);
    g.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Down,
    ));
    div().w(460.0).child(g.build())
}

fn table() -> Div {
    let rows = vec![
        vec!["Ada".to_owned(), "12".to_owned()],
        vec!["Bob".to_owned(), "7".to_owned()],
    ];
    let t = Table::new(
        vec![Column::new("n", "Name"), Column::new("s", "Score")],
        rows,
    )
    .id("g3-tb")
    .caption("Scores")
    .row_headers()
    .column_width(200.0);
    div().w(460.0).child(t.build())
}

fn feed() -> Div {
    let art = |k: &str| {
        let body = format!("What happened in {k}.");
        Article::new(k, format!("News of {k}"), move || {
            div().child(text(body.clone()))
        })
    };
    let f = Feed::new(vec![art("May"), art("June"), art("July")])
        .id("g3-fd")
        .name("News");
    f.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::PageDown,
    ));
    div().w(460.0).flex_col().child(f.build())
}

#[test]
fn every_scene_is_drawn_in_one_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for (name, w, h, build) in SCENES {
        let file = format!("g3-{name}.png");
        let light = draw_styled(build, w, h, StyleId::Admin, false, &css, &file);
        let dark = draw_styled(build, w, h, StyleId::Admin, true, &css, &file);
        for (img, mode) in [(&light, "light"), (&dark, "dark")] {
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
            assert!(ink > 300, "{name} {mode}: nearly empty ({ink} ink pixels)");
        }
        assert_ne!(
            ground_of(&light),
            ground_of(&dark),
            "{name}: dark is not dark"
        );
    }
}

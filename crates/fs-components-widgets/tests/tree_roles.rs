//! `tree`: roles and numbers (`aria-level`, `aria-setsize`, `aria-posinset`, `aria-expanded` only on parents, `aria-selected`), the
//! `group` of an open parent, the keys (Down, Up, Right, Left, Home, End, Enter and Space as the window's click AND as raw keys),
//! the state classes, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::tree::{STYLES, Tree};
use fs_components_widgets::tree_view::TreeNode;

type Log = Arc<Mutex<Vec<String>>>;

fn nodes() -> Vec<TreeNode> {
    let mut docs = TreeNode::new("docs", "Documents");
    docs.children = vec![TreeNode::new("a", "Report"), {
        let mut y = TreeNode::new("y", "2026");
        y.children = vec![TreeNode::new("q", "Q4")];
        y
    }];
    vec![
        docs,
        TreeNode::new("pics", "Pictures"),
        TreeNode::new("music", "Music"),
    ]
}

fn tree(log: &Log) -> Tree {
    let l = Arc::clone(log);
    Tree::new(&nodes())
        .id("tr")
        .name("Folders")
        .on_pick(move |k| l.lock().unwrap().push(k.to_owned()))
}

fn show(t: &Tree) -> RenderTree {
    layout_with(t.build(), 400.0, 300.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the row");
    tree.element_registry().has_class(node, class)
}

fn current(t: &RenderTree) -> Vec<&'static str> {
    ["docs", "a", "y", "q", "pics", "music"]
        .into_iter()
        .filter(|k| {
            t.query_by_id(&format!("tr-row-{k}"))
                .is_some_and(|n| t.element_registry().has_class(n, "fs-treeitem-current"))
        })
        .collect()
}

#[test]
fn the_tree_is_a_named_tree_with_one_tab_stop_and_closed_parents_hide_their_children() {
    let _g = serial();
    let tree = show(&tree(&Log::default()));
    assert_eq!(attr(&tree, "tr", "role").as_deref(), Some("tree"));
    assert_eq!(attr(&tree, "tr", "aria-label").as_deref(), Some("Folders"));
    assert_eq!(attr(&tree, "tr", "tabindex").as_deref(), Some("0"));
    assert_eq!(
        attr(&tree, "tr-item-docs", "role").as_deref(),
        Some("treeitem")
    );
    assert_eq!(
        attr(&tree, "tr-item-docs", "aria-expanded").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&tree, "tr-item-pics", "aria-expanded"),
        None,
        "a leaf has no aria-expanded"
    );
    assert!(
        tree.query_by_id("tr-item-a").is_none(),
        "closed: no children"
    );
    assert!(tree.query_by_id("tr-group-docs").is_none());
}

#[test]
fn level_set_size_and_position_are_told_for_every_visible_item() {
    let _g = serial();
    let t = tree(&Log::default());
    t.set_open("docs", true);
    t.set_open("y", true);
    let tree = show(&t);
    let n = |id: &str, a: &str| attr(&tree, id, a);
    assert_eq!(n("tr-item-docs", "aria-level").as_deref(), Some("1"));
    assert_eq!(n("tr-item-docs", "aria-setsize").as_deref(), Some("3"));
    assert_eq!(n("tr-item-docs", "aria-posinset").as_deref(), Some("1"));
    assert_eq!(n("tr-item-music", "aria-posinset").as_deref(), Some("3"));
    assert_eq!(n("tr-item-y", "aria-level").as_deref(), Some("2"));
    assert_eq!(n("tr-item-y", "aria-setsize").as_deref(), Some("2"));
    assert_eq!(n("tr-item-y", "aria-posinset").as_deref(), Some("2"));
    assert_eq!(n("tr-item-q", "aria-level").as_deref(), Some("3"));
    assert_eq!(n("tr-item-docs", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(n("tr-group-docs", "role").as_deref(), Some("group"));
    assert_eq!(n("tr-group-y", "role").as_deref(), Some("group"));
}

#[test]
fn down_and_up_walk_the_visible_items_and_right_and_left_open_close_and_climb() {
    let _g = serial();
    let t = tree(&Log::default());
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tr"));
    assert_eq!(
        current(&tree),
        ["docs"],
        "the cursor starts on the first item"
    );
    input.key(&mut tree, 40);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(current(&tree), ["pics"], "Down skips the hidden children");
    input.key(&mut tree, 38);
    input.key(&mut tree, 39);
    frame(&mut tree, 400.0, 300.0);
    assert!(t.is_open("docs"), "Right opens a closed parent");
    assert_eq!(current(&tree), ["docs"], "and stays");
    assert_eq!(
        attr(&tree, "tr-item-docs", "aria-expanded").as_deref(),
        Some("true")
    );
    input.key(&mut tree, 39);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(
        current(&tree),
        ["a"],
        "Right on an open parent goes to its first child"
    );
    input.key(&mut tree, 39);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(current(&tree), ["a"], "Right on a leaf does nothing");
    input.key(&mut tree, 37);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(
        current(&tree),
        ["docs"],
        "Left on a leaf goes to its parent"
    );
    input.key(&mut tree, 37);
    frame(&mut tree, 400.0, 300.0);
    assert!(!t.is_open("docs"), "Left on an open parent closes it");
    assert_eq!(current(&tree), ["docs"]);
    input.key(&mut tree, 37);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(
        current(&tree),
        ["docs"],
        "Left on a closed top item does nothing"
    );
}

#[test]
fn home_and_end_jump_over_the_visible_items_only() {
    let _g = serial();
    let t = tree(&Log::default());
    t.set_open("docs", true);
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tr"));
    input.key(&mut tree, 35);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(current(&tree), ["music"]);
    input.key(&mut tree, 36);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(current(&tree), ["docs"]);
    input.key(&mut tree, 40);
    input.key(&mut tree, 40);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(
        current(&tree),
        ["y"],
        "an open parent's children are walked"
    );
    input.key(&mut tree, 38);
    input.key(&mut tree, 38);
    input.key(&mut tree, 38);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(current(&tree), ["docs"], "no wrap at the top");
}

#[test]
fn enter_and_space_pick_once_as_the_windows_click_and_as_raw_keys() {
    let _g = serial();
    let log = Log::default();
    let t = tree(&log);
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tr"));
    input.key(&mut tree, 13);
    assert_eq!(t.picked().as_deref(), Some("docs"));
    input.key(&mut tree, 40);
    input.key(&mut tree, 32);
    assert_eq!(t.picked().as_deref(), Some("pics"));
    input.key(&mut tree, 40);
    input.key_event_to(&mut tree, "tr", KEY_DOWN, 32);
    assert_eq!(t.picked().as_deref(), Some("music"), "a raw Space picks");
    input.key(&mut tree, 38);
    input.key_event_to(&mut tree, "tr", KEY_DOWN, 13);
    assert_eq!(t.picked().as_deref(), Some("pics"), "a raw Enter picks");
    input.key_event_to(&mut tree, "tr", KEY_DOWN, 65);
    assert_eq!(
        *log.lock().unwrap(),
        ["docs", "pics", "music", "pics"],
        "another key does nothing; each pick is heard once"
    );
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(
        attr(&tree, "tr-item-pics", "aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&tree, "tr-item-music", "aria-selected").as_deref(),
        Some("false")
    );
    assert!(has(&tree, "tr-row-pics", "fs-treeitem-picked"));
}

#[test]
fn a_click_picks_and_moves_the_cursor_and_closing_an_ancestor_keeps_the_cursor_on_a_visible_item() {
    let _g = serial();
    let log = Log::default();
    let t = tree(&log);
    t.set_open("docs", true);
    let mut tree = show(&t);
    let mut input = Input::new();
    input.click(&mut tree, "tr-row-a");
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(*log.lock().unwrap(), ["a"]);
    assert_eq!(current(&tree), ["a"]);
    t.set_open("docs", false);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(
        current(&tree),
        ["docs"],
        "the cursor goes to the closed ancestor"
    );
    t.set_open("pics", true);
    t.set_open("nothing", true);
    assert!(
        !t.is_open("pics") && !t.is_open("nothing"),
        "a leaf and a missing key do not open"
    );
}

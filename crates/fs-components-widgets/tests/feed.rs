//! `feed`: role, name, `aria-busy`, the article roles with title, position and size (`-1` while unknown), Page Down and Page Up (and
//! only those), a click, pushing more articles, the current class, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use blinc_layout::prelude::{RenderTree, text};
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::feed::{Article, Feed, STYLES};
use fs_components_widgets::keys::{Key, KeyEvent};

type Log = Arc<Mutex<Vec<String>>>;

fn art(key: &str) -> Article {
    let body = format!("body of {key}");
    Article::new(key, format!("Title {key}"), move || div().child(text(body.clone())))
}

fn feed(log: &Log) -> Feed {
    let l = Arc::clone(log);
    Feed::new(vec![art("a"), art("b"), art("c")]).id("fd").name("News").on_move(move |k| l.lock().unwrap().push(k.to_owned()))
}

fn show(f: &Feed) -> RenderTree {
    layout_with(f.build(), 500.0, 400.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the article");
    tree.element_registry().has_class(node, class)
}

#[test]
fn the_feed_is_a_named_feed_and_each_article_tells_title_place_and_an_unknown_size() {
    let _g = serial();
    let tree = show(&feed(&Log::default()));
    assert_eq!(attr(&tree, "fd", "role").as_deref(), Some("feed"));
    assert_eq!(attr(&tree, "fd", "aria-label").as_deref(), Some("News"));
    assert_eq!(attr(&tree, "fd", "tabindex").as_deref(), Some("0"), "one tab stop");
    assert_eq!(attr(&tree, "fd-article-b", "role").as_deref(), Some("article"));
    assert_eq!(attr(&tree, "fd-article-b", "aria-posinset").as_deref(), Some("2"));
    assert_eq!(attr(&tree, "fd-article-b", "aria-setsize").as_deref(), Some("-1"), "the total is not known: -1 as the APG asks");
    assert_eq!(attr(&tree, "fd-article-b", "aria-labelledby").as_deref(), Some("fd-title-b"));
    assert!(tree.query_by_id("fd-title-b").is_some(), "the title the article names exists");
}

#[test]
fn a_known_total_is_told_and_never_less_than_what_is_there() {
    let _g = serial();
    let f = Feed::new(vec![art("a"), art("b")]).id("ft").total(40);
    let tree = show(&f);
    assert_eq!(attr(&tree, "ft-article-a", "aria-setsize").as_deref(), Some("40"));
    let small = Feed::new(vec![art("a"), art("b"), art("c")]).id("fs").total(1);
    let tree = show(&small);
    assert_eq!(attr(&tree, "fs-article-a", "aria-setsize").as_deref(), Some("3"));
}

#[test]
fn page_down_and_page_up_walk_the_articles_and_other_keys_do_nothing() {
    let _g = serial();
    let log = Log::default();
    let f = feed(&log);
    let mut tree = show(&f);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "fd"));
    assert_eq!(f.current().as_deref(), Some("a"));
    input.key(&mut tree, 34);
    assert_eq!(f.current().as_deref(), Some("b"));
    input.key(&mut tree, 34);
    input.key(&mut tree, 34);
    assert_eq!(f.current().as_deref(), Some("c"), "no wrap at the end");
    input.key(&mut tree, 33);
    assert_eq!(f.current().as_deref(), Some("b"));
    input.key(&mut tree, 33);
    input.key(&mut tree, 33);
    assert_eq!(f.current().as_deref(), Some("a"), "no wrap at the start");
    for code in [40, 38, 39, 37, 36, 35, 13, 32] {
        input.key(&mut tree, code);
    }
    assert_eq!(f.current().as_deref(), Some("a"), "arrows, Home, End, Enter and Space are the host's");
    assert_eq!(*log.lock().unwrap(), ["b", "c", "b", "a"], "each move is heard once");
}

#[test]
fn the_current_class_follows_the_cursor_and_a_click_moves_it() {
    let _g = serial();
    let log = Log::default();
    let f = feed(&log);
    let mut tree = show(&f);
    assert!(has(&tree, "fd-article-a", "fs-article-current"));
    assert!(!has(&tree, "fd-article-b", "fs-article-current"));
    let mut input = Input::new();
    input.click(&mut tree, "fd-article-c");
    frame(&mut tree, 500.0, 400.0);
    assert!(has(&tree, "fd-article-c", "fs-article-current"));
    assert!(!has(&tree, "fd-article-a", "fs-article-current"));
    assert_eq!(*log.lock().unwrap(), ["c"]);
}

#[test]
fn busy_is_told_while_more_are_loading_and_pushed_articles_join_the_end() {
    let _g = serial();
    let f = feed(&Log::default());
    let mut tree = show(&f);
    assert_eq!(attr(&tree, "fd-articles", "aria-busy").as_deref(), Some("false"));
    f.busy(true);
    frame(&mut tree, 500.0, 400.0);
    assert_eq!(attr(&tree, "fd-articles", "aria-busy").as_deref(), Some("true"));
    f.push(vec![art("d")]);
    f.busy(false);
    frame(&mut tree, 500.0, 400.0);
    assert_eq!(attr(&tree, "fd-articles", "aria-busy").as_deref(), Some("false"));
    assert_eq!(attr(&tree, "fd-article-d", "aria-posinset").as_deref(), Some("4"));
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "fd"));
    for _ in 0..5 {
        input.key(&mut tree, 34);
    }
    assert_eq!(f.current().as_deref(), Some("d"), "the new article can be reached");
}

#[test]
fn an_empty_feed_has_no_cursor_and_the_first_push_gives_it_one() {
    let _g = serial();
    let f = Feed::new(Vec::new()).id("fe");
    let _tree = show(&f);
    assert!(f.current().is_none());
    assert!(!f.press(KeyEvent::new(Key::PageDown)), "nothing to move to");
    assert!(f.current().is_none());
    f.push(vec![art("a")]);
    assert_eq!(f.current().as_deref(), Some("a"));
}

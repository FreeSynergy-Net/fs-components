//! `checklist` (model): blanks are no item, a key is never given again, the keys on an empty list do nothing, the cursor keeps its item
//! when an earlier one goes and stays in range when the last goes.
//!
//! Tests: model tests over `State`; no element, no window.

use fs_components_widgets::checklist::{Item, Outcome, State};
use fs_components_widgets::keys::{Key, KeyEvent};

#[test]
fn the_model_refuses_blanks_and_never_reuses_a_key() {
    let mut s = State::new(vec![Item::new("n1", "x")]);
    assert_eq!(s.add("  "), None);
    let k = s.add(" y ").expect("an item");
    assert_ne!(k, "n1");
    assert_eq!(s.items()[1].text, "y", "trimmed");
    assert!(s.remove(&k));
    let k2 = s.add("z").expect("an item");
    assert_ne!(k2, k, "a removed key is not given again");
    assert_eq!(s.toggle("nothing"), None);
    assert!(!s.remove("nothing"));
    let mut empty = State::new(Vec::new());
    for key in [Key::Down, Key::Space, Key::Delete, Key::Home] {
        assert_eq!(empty.press(KeyEvent::new(key)), Outcome::Ignored);
    }
}

#[test]
fn the_cursor_keeps_its_item_when_an_earlier_item_goes_and_stays_in_range_when_the_last_goes() {
    let mut s = State::new(vec![
        Item::new("a", "A"),
        Item::new("b", "B"),
        Item::new("c", "C"),
        Item::new("d", "D"),
    ]);
    s.go("c");
    assert_eq!(s.cursor(), 2);
    assert!(s.remove("a"));
    assert_eq!(s.cursor(), 1, "the cursor is still on C, one place earlier");
    assert_eq!(s.items()[s.cursor()].key, "c");
    assert!(s.remove("d"));
    assert_eq!(s.cursor(), 1, "a later item going does not move it");
    s.go("c");
    assert!(s.remove("c"));
    assert_eq!(
        s.cursor(),
        0,
        "the last item went: the cursor holds to the last place that is left"
    );
    assert_eq!(s.items()[0].key, "b");
}

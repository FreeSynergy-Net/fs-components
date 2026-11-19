//! `checklist` (model): blanks are no item, a key is never given again, the keys on an empty list do nothing, the cursor keeps its item
//! when an earlier one goes and stays in range when the last goes.
//!
//! Tests: model tests over `State`; no element, no window.

use fs_components_widgets::checklist::{Item, Outcome, State};
use fs_components_widgets::keys::{Key, KeyEvent};

#[test]
fn the_model_refuses_blanks_and_never_reuses_a_key() {
    let (blank, k, text, removed, k2, none, gone, empty) = within(|| {
        let mut s = State::new(vec![Item::new("n1", "x")]);
        let blank = s.add("  ");
        let k = s.add(" y ").expect("an item");
        let text = s.items()[1].text.clone();
        let removed = s.remove(&k);
        let k2 = s.add("z").expect("an item");
        let none = s.toggle("nothing");
        let gone = s.remove("nothing");
        let mut empty = State::new(Vec::new());
        let empty: Vec<Outcome> = [Key::Down, Key::Space, Key::Delete, Key::Home]
            .into_iter()
            .map(|key| empty.press(KeyEvent::new(key)))
            .collect();
        (blank, k, text, removed, k2, none, gone, empty)
    });
    assert_eq!(blank, None);
    assert_ne!(k, "n1");
    assert_eq!(text, "y", "trimmed");
    assert!(removed);
    assert_ne!(k2, k, "a removed key is not given again");
    assert_eq!(none, None);
    assert!(!gone);
    assert!(empty.iter().all(|o| *o == Outcome::Ignored));
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

/// Run `f` on another thread and give it a few seconds: a loop that never ends is a failed test, not a hung run.
fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .expect("the call did not come back within 5 seconds (an endless loop)")
}

#[test]
fn many_additions_give_distinct_keys_and_trimmed_texts_and_come_back() {
    let (keys, texts) = within(|| {
        let mut s = State::new(vec![Item::new("n0", "first"), Item::new("n2", "third")]);
        for i in 0..12 {
            s.add(&format!("  item {i}\t"));
        }
        let keys: Vec<String> = s.items().iter().map(|i| i.key.clone()).collect();
        let texts: Vec<String> = s.items().iter().map(|i| i.text.clone()).collect();
        (keys, texts)
    });
    let mut unique = keys.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), keys.len(), "no key twice: {keys:?}");
    assert_eq!(keys.len(), 14);
    assert!(
        texts[2..]
            .iter()
            .all(|t| t.starts_with("item ") && t == t.trim()),
        "every new text is trimmed: {texts:?}"
    );
    assert_eq!(texts[2], "item 0");
}

#[test]
fn a_key_that_is_taken_is_skipped_and_the_call_comes_back() {
    let k = within(|| {
        // the counter would give n1 and n2 next: both are taken
        let mut s = State::new(vec![Item::new("n1", "a"), Item::new("n2", "b")]);
        s.add("c").expect("an item")
    });
    assert!(k != "n1" && k != "n2", "got {k}");
}

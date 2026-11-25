//! The keys of the card (M3/M4): one table, mode x key; the table itself, then the card through real key events, with the sorter's words
//! as the one keyboard way to move a card.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]
mod card_support;

use std::sync::{Arc, Mutex};

use card_support::{at, card};
use fs_components_widgets::answer::{Answer, Reason};
use fs_components_widgets::card::keys::{Mode, Step, step};
use fs_components_widgets::card::{Card, CardEvents, CardRef, Carry, DropTargets, Outcome, Target};
use fs_components_widgets::sortable_list::Command;
use fs_components_widgets::{Key, KeyEvent};

fn k(key: Key) -> KeyEvent {
    KeyEvent::new(key)
}
fn shift_f10() -> KeyEvent {
    let mut e = k(Key::Other(121));
    e.shift = true;
    e
}

#[test]
fn the_table_mode_by_key() {
    use Command as C;
    use Step::{Activate, Menu, Pass, Refuse, Sort};
    let arrows = [
        (Key::Up, C::Up),
        (Key::Down, C::Down),
        (Key::Left, C::Left),
        (Key::Right, C::Right),
        (Key::Home, C::First),
        (Key::End, C::Last),
        (Key::PageUp, C::PageUp),
        (Key::PageDown, C::PageDown),
    ];
    // rest
    assert_eq!(step(Mode::Rest, &k(Key::Enter)), Activate);
    assert_eq!(step(Mode::Rest, &k(Key::Space)), Sort(C::Toggle));
    assert_eq!(step(Mode::Rest, &shift_f10()), Menu);
    assert_eq!(step(Mode::Rest, &k(Key::Other(93))), Menu);
    assert_eq!(
        step(Mode::Rest, &k(Key::Other(121))),
        Pass,
        "F10 without Shift is the window's"
    );
    for key in [Key::Escape, Key::Up, Key::Delete, Key::Other(65)] {
        assert_eq!(step(Mode::Rest, &k(key)), Pass, "{key:?}");
    }
    // held: the same words as the sorter's
    for (key, c) in arrows {
        assert_eq!(step(Mode::Held, &k(key)), Sort(c), "{key:?}");
    }
    assert_eq!(
        step(Mode::Held, &k(Key::Enter)),
        Sort(C::Toggle),
        "Enter puts down, it does not open"
    );
    assert_eq!(step(Mode::Held, &k(Key::Space)), Sort(C::Toggle));
    assert_eq!(step(Mode::Held, &k(Key::Escape)), Sort(C::Cancel));
    assert_eq!(
        step(Mode::Held, &shift_f10()),
        Pass,
        "a held card is not opened"
    );
    // carried by the pointer
    assert_eq!(step(Mode::Carried, &k(Key::Escape)), Sort(C::Cancel));
    for key in [Key::Enter, Key::Space, Key::Up, Key::Other(93)] {
        assert_eq!(step(Mode::Carried, &k(key)), Pass, "{key:?}");
    }
    // locked
    assert_eq!(step(Mode::Locked, &k(Key::Enter)), Activate);
    assert_eq!(step(Mode::Locked, &k(Key::Space)), Refuse);
    assert_eq!(step(Mode::Locked, &shift_f10()), Menu);
    assert_eq!(step(Mode::Locked, &k(Key::Up)), Pass);
    // Ctrl and Alt are never the card's (undo etc. belong to the host)
    for mode in [Mode::Rest, Mode::Held, Mode::Carried, Mode::Locked] {
        for key in [Key::Enter, Key::Space, Key::Escape, Key::Up] {
            let mut e = k(key);
            e.ctrl = true;
            assert_eq!(step(mode, &e), Pass);
            e.ctrl = false;
            e.alt = true;
            assert_eq!(step(mode, &e), Pass);
        }
    }
}

#[derive(Default)]
struct Log(Mutex<Vec<String>>);
impl Log {
    fn lines(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}
impl CardEvents for Log {
    fn activate(&self, c: &CardRef) {
        self.0
            .lock()
            .unwrap()
            .push(format!("activate {}", c.iri_text()));
    }
    fn menu(&self, _: &CardRef, _: (f32, f32)) {
        self.0.lock().unwrap().push("menu".into());
    }
    fn dropped(&self, carry: &Carry, to: &Target) -> Answer {
        self.0
            .lock()
            .unwrap()
            .push(format!("drop {}->{}", carry.from.column, to.column));
        Answer::Accepted
    }
    fn changed(&self, _: &CardRef) {
        self.0.lock().unwrap().push("changed".into());
    }
}
struct OnlyDoing;
impl DropTargets for OnlyDoing {
    fn target_at(&self, _: (f32, f32), _: &CardRef) -> Option<Target> {
        None
    }
    fn can_drop(&self, _: &CardRef, to: &Target) -> Result<(), Reason> {
        if to.column == "doing" {
            Ok(())
        } else {
            Err(Reason::new("wip", "Doing is the only place."))
        }
    }
}

fn held_card(log: &Arc<Log>) -> Card {
    Card::new("c", card("a"))
        .at(at("todo", None, None))
        .events(log.clone())
        .targets(Arc::new(OnlyDoing))
}

#[test]
fn enter_activates_space_lifts_the_container_walks_enter_puts_down() {
    let log = Arc::new(Log::default());
    let c = held_card(&log);
    assert_eq!(c.key_press(&k(Key::Enter)), Outcome::Activated);
    assert_eq!(
        c.key_press(&k(Key::Up)),
        Outcome::Nothing,
        "the container's key at rest"
    );
    assert_eq!(c.key_press(&k(Key::Space)), Outcome::Lifted);
    assert_eq!(c.mode(), Mode::Held);
    assert_eq!(c.key_press(&k(Key::Right)), Outcome::Walk(Command::Right));
    c.choose(at("doing", None, None));
    assert_eq!(c.over().unwrap().0, Some(at("doing", None, None)));
    assert!(c.blocked().is_none());
    assert_eq!(
        c.key_press(&k(Key::Enter)),
        Outcome::Dropped(Answer::Accepted)
    );
    assert_eq!(c.mode(), Mode::Rest);
    assert!(log.lines().contains(&"drop todo->doing".to_owned()));
    assert_eq!(
        log.lines()
            .iter()
            .filter(|l| l.starts_with("activate"))
            .count(),
        1,
        "put down is not an activation"
    );
}

#[test]
fn a_place_that_does_not_take_the_card_is_said_at_once_and_nothing_is_sent() {
    let log = Arc::new(Log::default());
    let c = held_card(&log);
    c.key_press(&k(Key::Space));
    c.choose(at("done", None, None));
    assert_eq!(c.blocked().unwrap().said(), "Doing is the only place.");
    match c.key_press(&k(Key::Space)) {
        Outcome::Refused(r) => assert_eq!(r.code, "wip"),
        other => panic!("{other:?}"),
    }
    assert!(
        !log.lines().iter().any(|l| l.starts_with("drop")),
        "no drop reached the owner"
    );
}

#[test]
fn escape_gives_the_hold_up_and_a_locked_card_says_why() {
    let log = Arc::new(Log::default());
    let c = held_card(&log);
    c.key_press(&k(Key::Space));
    assert_eq!(c.key_press(&k(Key::Escape)), Outcome::Back);
    assert_eq!(c.mode(), Mode::Rest);
    assert_eq!(
        c.key_press(&k(Key::Escape)),
        Outcome::Nothing,
        "nothing to give up: the window's Escape"
    );
    let l = Card::new("l", card("b"))
        .events(log.clone())
        .locked(Reason::new("agent", "An agent is working on it."));
    assert_eq!(l.mode(), Mode::Locked);
    assert_eq!(
        l.key_press(&k(Key::Space)),
        Outcome::Refused(Reason::new("agent", "An agent is working on it."))
    );
    assert_eq!(l.mode(), Mode::Locked, "not lifted");
    assert_eq!(l.key_press(&k(Key::Enter)), Outcome::Activated);
    assert_eq!(l.key_press(&shift_f10()), Outcome::Menu);
    assert!(log.lines().contains(&"menu".to_owned()));
}

#[test]
fn space_without_targets_lifts_nothing() {
    let log = Arc::new(Log::default());
    let c = Card::new("n", card("n")).events(log.clone());
    assert_eq!(
        c.key_press(&k(Key::Space)),
        Outcome::Nothing,
        "passed on, not the card's"
    );
    assert_eq!(c.mode(), Mode::Rest, "not held");
    assert!(
        !log.lines().iter().any(|l| l.starts_with("drop")),
        "no drop"
    );
    assert_eq!(
        c.key_press(&k(Key::Enter)),
        Outcome::Activated,
        "Enter still activates"
    );
}

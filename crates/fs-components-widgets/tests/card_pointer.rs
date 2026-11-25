//! The card with the pointer: click, carry, drop with the arguments of the contract, the refusal at once, a locked card, a card with no
//! targets; the key of a carry is a reference (object IRI and media type), never a text.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]
mod card_support;

use std::sync::{Arc, Mutex};

use card_support::{at, card, stack, zones};
use fs_components_widgets::answer::{Answer, Reason};
use fs_components_widgets::card::gesture::Moved;
use fs_components_widgets::card::{
    Card, CardEvents, CardRef, Carry, DropTargets, Mode, Outcome, Target, target_at,
};

type Line = (String, Option<Carry>, Option<Target>);
#[derive(Default)]
struct Owner(Mutex<Vec<Line>>);
impl CardEvents for Owner {
    fn activate(&self, c: &CardRef) {
        self.0
            .lock()
            .unwrap()
            .push((format!("activate {}", c.iri_text()), None, None));
    }
    fn menu(&self, _: &CardRef, at: (f32, f32)) {
        self.0
            .lock()
            .unwrap()
            .push((format!("menu {at:?}"), None, None));
    }
    fn dropped(&self, carry: &Carry, to: &Target) -> Answer {
        self.0
            .lock()
            .unwrap()
            .push(("drop".into(), Some(carry.clone()), Some(to.clone())));
        Answer::Accepted
    }
}

/// A board of three columns (fixed rectangles), `doing` is full.
struct Board;
impl DropTargets for Board {
    fn target_at(&self, window: (f32, f32), carried: &CardRef) -> Option<Target> {
        let cards = [
            stack(None, "todo", 0.0, &["a", "b"]),
            stack(None, "doing", 120.0, &["c"]),
        ]
        .concat();
        target_at(&zones(None), &cards, window, Some(carried))
    }
    fn can_drop(&self, _: &CardRef, to: &Target) -> Result<(), Reason> {
        if to.column == "doing" {
            Err(Reason::new("wip-full", "Doing is full."))
        } else {
            Ok(())
        }
    }
}

fn card_a(owner: &Arc<Owner>) -> Card {
    Card::new("a", card("a"))
        .at(at("todo", None, Some("b")))
        .events(owner.clone())
        .targets(Arc::new(Board))
}

#[test]
fn a_click_activates_once_and_the_click_after_a_carry_does_nothing() {
    let owner = Arc::new(Owner::default());
    let c = card_a(&owner);
    c.press((10.0, 10.0));
    assert_eq!(c.pointer((11.0, 11.0)), Moved::Nothing);
    assert_eq!(c.release(), Outcome::Activated);
    assert_eq!(c.release(), Outcome::Nothing);
    assert_eq!(owner.0.lock().unwrap().len(), 1);
    // after a carry the click that follows finds nothing going on
    c.press((10.0, 10.0));
    assert_eq!(c.pointer((250.0, 30.0)), Moved::Started);
    assert_eq!(c.mode(), Mode::Carried);
    assert!(matches!(c.release(), Outcome::Dropped(Answer::Accepted)));
    assert_eq!(c.release(), Outcome::Nothing);
    assert_eq!(owner.0.lock().unwrap().len(), 2);
}

#[test]
fn a_drop_carries_the_reference_and_names_the_arguments_of_the_contract() {
    let owner = Arc::new(Owner::default());
    let c = card_a(&owner);
    c.press((10.0, 10.0));
    c.pointer((250.0, 100.0));
    c.release();
    let log = owner.0.lock().unwrap();
    let (what, carry, to) = &log[0];
    assert_eq!(what, "drop");
    let carry = carry.as_ref().unwrap();
    assert_eq!(carry.key.iri_text(), "https://example.org/tasks/a");
    assert_eq!(carry.key.media_type.0, "application/activity+json");
    assert_eq!(carry.from, at("todo", None, Some("b")));
    let to = to.as_ref().unwrap();
    assert_eq!(
        (
            to.column.as_str(),
            to.lane.clone(),
            to.after.clone(),
            to.before.clone()
        ),
        ("done", None, None, None)
    );
    assert_eq!(Target::ARGS, ["column", "lane", "after", "before"]);
}

#[test]
fn a_bad_reference_is_refused_by_the_type() {
    assert!(CardRef::parse("not an iri", "text/plain").is_err());
    assert!(CardRef::parse("https://example.org/x", "text/plain").is_ok());
}

#[test]
fn the_refusal_shows_while_over_the_place_and_nothing_is_sent() {
    let owner = Arc::new(Owner::default());
    let c = card_a(&owner);
    c.press((10.0, 10.0));
    c.pointer((130.0, 30.0));
    assert_eq!(c.blocked().unwrap().code, "wip-full");
    let (over, pointer) = c.over().unwrap();
    assert_eq!(over.unwrap().column, "doing");
    assert_eq!(pointer, (130.0, 30.0));
    c.pointer((250.0, 30.0));
    assert!(c.blocked().is_none(), "the next place takes it");
    c.pointer((130.0, 30.0));
    assert!(matches!(c.release(), Outcome::Refused(r) if r.code == "wip-full"));
    assert!(owner.0.lock().unwrap().is_empty());
    assert_eq!(c.mode(), Mode::Rest);
}

#[test]
fn over_no_place_the_card_goes_back_and_escape_gives_up() {
    let owner = Arc::new(Owner::default());
    let c = card_a(&owner);
    c.press((10.0, 10.0));
    c.pointer((110.0, 30.0)); // the gap
    assert!(c.over().unwrap().0.is_none());
    assert_eq!(c.release(), Outcome::Back);
    c.press((10.0, 10.0));
    c.pointer((250.0, 30.0));
    assert_eq!(
        c.key_press(&fs_components_widgets::KeyEvent::new(
            fs_components_widgets::Key::Escape
        )),
        Outcome::Back
    );
    assert_eq!(c.release(), Outcome::Nothing);
    assert!(owner.0.lock().unwrap().is_empty());
}

#[test]
fn a_locked_card_is_not_carried_and_says_why_and_a_card_without_targets_is_only_clickable() {
    let owner = Arc::new(Owner::default());
    let l = card_a(&owner).locked(Reason::new("agent", "An agent holds it."));
    l.press((10.0, 10.0));
    assert_eq!(l.pointer((250.0, 30.0)), Moved::Nothing);
    assert_eq!(l.blocked().unwrap().code, "agent");
    assert_eq!(l.release(), Outcome::Nothing);
    assert!(owner.0.lock().unwrap().is_empty());
    let plain = Card::new("p", card("p")).events(owner.clone());
    plain.press((10.0, 10.0));
    assert_eq!(
        plain.pointer((250.0, 30.0)),
        Moved::Nothing,
        "nothing to carry to"
    );
    assert_eq!(plain.release(), Outcome::Activated);
    assert_eq!(owner.0.lock().unwrap().len(), 1);
}

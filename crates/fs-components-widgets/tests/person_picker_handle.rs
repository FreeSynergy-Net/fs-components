//! The person picker's handle: names looked up at each draw, keys and clicks in process.
//!
//! Tests: Model tests over a counting source; an in-process mouse and keyboard.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial, size_of};
use fs_components_widgets::person_picker::{
    Bindings, Candidate, CandidatePage, CandidateSource, FakeSource, Labels, Outcome, PersonPicker,
    PersonRef, PickerError,
};

fn r(id: &str) -> PersonRef {
    PersonRef::new(id)
}

fn people() -> Vec<Candidate> {
    vec![
        Candidate::new(r("p-ada")).named("Ada Lovelace"),
        Candidate::new(r("p-bob")).named("Bob Marley"),
        Candidate::new(r("p-eve")).named("Eve"),
    ]
}

/// A source that counts how often a name is asked for: the picker looks names up at each draw and stores none.
struct Counting {
    asked: Mutex<u32>,
}

impl CandidateSource for Counting {
    fn search(&self, _q: &str, _l: usize) -> Result<Option<CandidatePage>, PickerError> {
        Ok(Some(CandidatePage::default()))
    }
    fn me(&self) -> Result<Option<PersonRef>, PickerError> {
        Ok(None)
    }
    fn display_of(&self, person: &PersonRef) -> Option<String> {
        *self.asked.lock().unwrap() += 1;
        Some(format!("Name of {}", person.as_str()))
    }
}

#[test]
fn a_chosen_name_is_looked_up_at_each_draw_and_never_copied() {
    let _g = serial();
    let src = Arc::new(Counting {
        asked: Mutex::new(0),
    });
    let picker = PersonPicker::new(Arc::clone(&src) as Arc<dyn CandidateSource>)
        .id("cp")
        .choose(vec![r("p-far")]);
    assert_eq!(*src.asked.lock().unwrap(), 0, "choosing asks for no name");
    let _tree = layout(picker.build(), 520.0, 400.0);
    let first = *src.asked.lock().unwrap();
    assert!(
        first >= 1,
        "the draw asks the source for the chosen person's name"
    );
    let _tree = layout(picker.build(), 520.0, 400.0);
    assert!(
        *src.asked.lock().unwrap() > first,
        "and asks again at the next draw"
    );
    assert_eq!(
        picker.chosen(),
        [r("p-far")],
        "what is held is the reference alone"
    );
}

#[test]
fn keys_step_the_picker_and_the_search_field_keeps_its_own_keys() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let fake = FakeSource::new(people());
    let picker = PersonPicker::new(Arc::new(fake))
        .id("kp")
        .keys(Bindings::default())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(picker.build(), 520.0, 500.0);
    let mut input = Input::new();
    input.focus(&tree, "kp");
    input.key_broadcast(&mut tree, 40); // Down
    input.key_broadcast(&mut tree, 32); // Space
    assert_eq!(picker.chosen(), [r("p-bob")]);
    input.key_broadcast(&mut tree, 27); // Escape
    assert_eq!(got.lock().unwrap().last(), Some(&Outcome::Cancelled));
    // typing into the search field must not choose rows
    let before = picker.chosen();
    input.click_at(&mut tree, 260.0, 80.0);
    input.key_broadcast(&mut tree, 32);
    assert_eq!(picker.chosen(), before);
}

#[test]
fn clicks_choose_assign_to_me_and_confirm() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let fake = FakeSource::new(people()).with_me(r("p-me"));
    let picker = PersonPicker::new(Arc::new(fake))
        .id("cl")
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(picker.build(), 520.0, 500.0);
    let mut input = Input::new();
    input.click(&mut tree, "cl-pick-p-eve");
    let mut tree = layout(picker.build(), 520.0, 500.0);
    input.click(&mut tree, "cl-me");
    assert_eq!(picker.chosen(), [r("p-eve"), r("p-me")]);
    let mut tree = layout(picker.build(), 520.0, 500.0);
    input.click(&mut tree, "cl-confirm");
    assert_eq!(
        got.lock().unwrap().last(),
        Some(&Outcome::Confirmed(vec![r("p-eve"), r("p-me")]))
    );
}

/// A person the source gives no name for is drawn with the word "name not shared", never with the reference as a name.
#[test]
fn a_row_without_a_released_name_shows_the_word_not_the_reference() {
    let _g = serial();
    let mut labels = Labels::default();
    labels.unnamed = "Person whose name is not shared with this window".into();
    let fake = FakeSource::new(vec![Candidate::new(r("p-x"))]);
    let picker = PersonPicker::new(Arc::new(fake)).id("un").labels(labels);
    let tree = layout(picker.build(), 520.0, 400.0);
    let (width, _) = size_of(&tree, "un-who-p-x");
    assert!(
        width > 150.0,
        "the row holds the long word, not the short reference (width {width})"
    );
}

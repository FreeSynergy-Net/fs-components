//! The person picker's state, without any drawing: search, choice, "assign to me", errors, references only.
//!
//! Tests: Model tests over the state and a typed fake source.

use std::sync::Arc;

use fs_components_mf2::check_pairs;
use fs_components_widgets::person_picker::{
    Candidate, ErrorCode, FakeSource, Labels, Outcome, PersonRef, Phase, PickerError, Picking,
    Refusal, Script, Selection, TEXTS,
};
use fs_components_words::{Locale, Localized};

fn r(id: &str) -> PersonRef {
    PersonRef::new(id)
}

fn people() -> Vec<Candidate> {
    vec![
        Candidate::new(r("p-ada"))
            .named("Ada Lovelace")
            .detailed("ada@example.org"),
        Candidate::new(r("p-bob")).named("Bob Marley"),
        Candidate::new(r("p-eve"))
            .named("李雷 Li Lei")
            .detailed("Org: Ärzte"),
        Candidate::new(r("p-hid")), // the source releases no name
    ]
}

fn picking(fake: &FakeSource) -> Picking {
    Picking::new(Arc::new(fake.clone()), 20)
}

fn refs(p: &Picking) -> Vec<&str> {
    p.rows().iter().map(|x| x.as_str()).collect()
}

fn chosen(p: &Picking) -> Vec<&str> {
    p.chosen().iter().map(PersonRef::as_str).collect()
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.me, "Mir zuweisen");
    assert_eq!(Labels::default().me, "Assign to me");
    assert_eq!(de.chosen(3), "3 ausgewählt");
    assert_eq!(Labels::default().chosen(3), "3 chosen");
}

#[test]
fn search_filters_by_released_text_and_keeps_the_source_rank() {
    let fake = FakeSource::new(people());
    let mut p = picking(&fake);
    assert_eq!(
        refs(&p),
        ["p-ada", "p-bob", "p-eve", "p-hid"],
        "empty query: the first ones"
    );
    p.set_query("LOVE");
    assert_eq!(refs(&p), ["p-ada"]);
    p.set_query("example.org");
    assert_eq!(refs(&p), ["p-ada"], "the released detail is searched too");
    p.set_query("zzz");
    assert!(refs(&p).is_empty());
    assert!(matches!(p.phase(), Phase::Ready(page) if page.items.is_empty()));
    assert_eq!(
        fake.asked().last(),
        Some(&("zzz".to_string(), 20)),
        "each query asks the source"
    );
}

#[test]
fn the_limit_is_asked_and_more_is_reported() {
    let fake = FakeSource::new(people());
    let p = Picking::new(Arc::new(fake.clone()), 2);
    assert_eq!(refs(&p), ["p-ada", "p-bob"]);
    assert!(matches!(p.phase(), Phase::Ready(page) if page.more));
    assert_eq!(fake.asked()[0].1, 2);
    let _ = Picking::new(Arc::new(fake.clone()), 500);
    assert_eq!(
        fake.asked().last().unwrap().1,
        50,
        "never above the source's own cap of 50"
    );
}

#[test]
fn a_person_without_a_released_name_stays_unnamed() {
    let fake = FakeSource::new(people());
    let p = picking(&fake);
    let Phase::Ready(page) = p.phase() else {
        panic!("not ready")
    };
    let hid = page
        .items
        .iter()
        .find(|c| c.reference == r("p-hid"))
        .unwrap();
    assert_eq!(hid.display, None, "the picker invents no name");
    assert_eq!(fake.display_of(&r("p-hid")), None);
}

#[test]
fn several_people_can_be_chosen_in_order_and_taken_away() {
    let fake = FakeSource::new(people());
    let mut p = picking(&fake);
    assert_eq!(p.toggle(&r("p-eve")), Outcome::Changed);
    assert_eq!(p.toggle(&r("p-ada")), Outcome::Changed);
    assert_eq!(chosen(&p), ["p-eve", "p-ada"]);
    p.toggle(&r("p-eve"));
    assert_eq!(chosen(&p), ["p-ada"]);
    assert_eq!(p.confirm(), Outcome::Confirmed(vec![r("p-ada")]));
}

#[test]
fn single_selection_replaces_the_choice() {
    let fake = FakeSource::new(people()).with_me(r("p-bob"));
    let mut p = picking(&fake);
    p.set_selection(Selection::Single);
    p.toggle(&r("p-ada"));
    p.toggle(&r("p-eve"));
    assert_eq!(chosen(&p), ["p-eve"]);
    p.choose_me();
    assert_eq!(
        chosen(&p),
        ["p-bob"],
        "assign to me replaces in single mode"
    );
    let mut q = picking(&fake);
    q.choose(vec![r("p-ada"), r("p-bob")]);
    q.set_selection(Selection::Single);
    assert_eq!(chosen(&q), ["p-ada"]);
}

#[test]
fn a_hidden_choice_is_confirmed_and_can_be_taken_away() {
    let fake = FakeSource::new(people());
    let mut p = picking(&fake);
    p.toggle(&r("p-ada"));
    p.set_query("bob");
    assert_eq!(
        chosen(&p),
        ["p-ada"],
        "the query hides the row, the choice stays"
    );
    assert_eq!(p.confirm(), Outcome::Confirmed(vec![r("p-ada")]));
    assert_eq!(
        p.toggle(&r("p-ada")),
        Outcome::Changed,
        "a hidden chosen person can be removed"
    );
    assert_eq!(
        p.toggle(&r("p-nobody")),
        Outcome::Refused(Refusal::NoSuchCandidate)
    );
    p.require_choice(true);
    assert_eq!(p.confirm(), Outcome::Refused(Refusal::NothingChosen));
}

#[test]
fn assign_to_me_chooses_the_own_reference_without_doubles() {
    let fake = FakeSource::new(people()).with_me(r("p-me"));
    let mut p = picking(&fake);
    assert_eq!(p.choose_me(), Outcome::Changed);
    assert_eq!(p.choose_me(), Outcome::Changed);
    assert_eq!(
        chosen(&p),
        ["p-me"],
        "me is not in the rows and still chosen once"
    );
    assert_eq!(p.confirm(), Outcome::Confirmed(vec![r("p-me")]));
}

#[test]
fn assign_to_me_without_an_own_card_or_with_a_failing_source_says_so() {
    let none = FakeSource::new(people());
    let mut p = picking(&none);
    assert_eq!(p.choose_me(), Outcome::Refused(Refusal::NoMe));
    assert!(p.notice().is_some(), "never silent");
    assert!(p.chosen().is_empty());
    let failing =
        FakeSource::new(people()).with_me_error(PickerError::new(ErrorCode::Denied, "ich-karte"));
    let mut q = picking(&failing);
    assert_eq!(q.choose_me(), Outcome::Refused(Refusal::SourceFailed));
    assert!(q.notice().is_some());
    q.set_query("a");
    assert!(q.notice().is_none(), "the next step clears the line");
}

#[test]
fn a_missing_or_failing_source_is_a_visible_error_with_code_and_parameter() {
    let fake = FakeSource::missing("person_directory_v1");
    let mut p = picking(&fake);
    let Phase::Failed(e) = p.phase().clone() else {
        panic!("a missing source must not look like an empty list")
    };
    assert_eq!(
        (e.code, e.param.as_str()),
        (ErrorCode::SourceMissing, "person_directory_v1")
    );
    assert_eq!(p.move_cursor(1), Outcome::Refused(Refusal::NotReady));
    let en = Labels::default().error(&e);
    assert!(
        en.contains("person_directory_v1") && en.contains("[picker.source-missing]"),
        "{en}"
    );
    let de = Labels::for_locale(&Locale::parse("de").unwrap()).error(&e);
    assert!(
        de.contains("Personenverzeichnis") && de.contains("person_directory_v1"),
        "{de}"
    );
    for (code, key) in [
        (ErrorCode::Denied, "[picker.denied]"),
        (ErrorCode::Unavailable, "[picker.unavailable]"),
        (ErrorCode::Failed, "[picker.failed]"),
    ] {
        assert!(
            Labels::default()
                .error(&PickerError::new(code, "x"))
                .ends_with(key)
        );
    }
    // it recovers when the source does
    fake.set_script(Script::Answer);
    assert_eq!(p.refresh(), Outcome::Changed);
    assert!(matches!(p.phase(), Phase::Ready(_)));
}

#[test]
fn a_pending_answer_is_the_loading_surface_until_refresh() {
    let fake = FakeSource::new(people());
    fake.set_script(Script::Pending);
    let mut p = picking(&fake);
    assert_eq!(*p.phase(), Phase::Loading);
    fake.set_script(Script::Answer);
    p.refresh();
    assert_eq!(refs(&p).len(), 4);
}

#[test]
fn the_cursor_moves_over_the_rows_and_stops_at_the_ends() {
    let fake = FakeSource::new(people());
    let mut p = picking(&fake);
    assert_eq!(p.cursor(), Some(&r("p-ada")));
    p.move_cursor(1);
    p.move_cursor(1);
    assert_eq!(p.cursor(), Some(&r("p-eve")));
    p.move_cursor(10);
    assert_eq!(p.cursor(), Some(&r("p-hid")));
    p.move_cursor(-10);
    assert_eq!(p.cursor(), Some(&r("p-ada")));
    p.set_query("zzz");
    assert_eq!(p.move_cursor(1), Outcome::Refused(Refusal::Empty));
}

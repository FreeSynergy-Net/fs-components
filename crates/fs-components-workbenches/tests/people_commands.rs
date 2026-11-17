//! The commands of the people workbench as a model: which lines a menu offers, which are locked and why, what running one does,
//! the choosing of several people, the groups, the keymap.
//!
//! Tests: state tests with a test double source (`people_common::Book`); no window.
//!
//! Mutants (each must turn a test red):
//! - a line the source cannot do is offered anyway (`capabilities` ignored)  -> `a_menu_offers_only_what_the_source_can_do`
//! - Merge is enabled with fewer than two marked                              -> `merge_is_locked_with_a_reason_until_two_are_marked`
//! - a locked command runs                                                    -> `a_locked_line_runs_nothing`
//! - Delete deletes at once instead of asking                                 -> `delete_from_the_menu_asks_first`
//! - the marks survive leaving the mode of choosing                           -> `leaving_the_mode_forgets_the_marks`
//! - a group filter is ignored (everyone is listed)                           -> `a_group_narrows_the_list_and_everyone_brings_it_back`
//! - Delete is a shortcut in a text field                                     -> `the_keymap_runs_the_shortcuts_and_leaves_delete_to_lists`
//! - a card copy hands the program a request                                  -> `copy_and_mail_go_where_they_belong`

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_commands::{
    FocusContext, KeyPress, Modifiers, Outcome, Platform, Resolved, State,
};
use fs_components_workbenches::people::{
    COPY, DELETE, EDIT, EXPORT, FieldKind, MAIL, MERGE, NEW, PeopleCapabilities, PeopleLabels,
    PeopleRequest, PeopleState, PersonField, Ran, SELECT, Variant, group_id, mark_id,
};
use people_common::Book;

fn state_with(book: Book) -> (Arc<Book>, PeopleState) {
    let book = Arc::new(book);
    let state = PeopleState::new(book.clone(), PeopleLabels::default(), Variant::Full);
    (book, state)
}

fn ids(items: &[Resolved]) -> Vec<String> {
    items
        .iter()
        .filter_map(|i| match i {
            Resolved::Command(c) => Some(c.id.as_str().to_owned()),
            _ => None,
        })
        .collect()
}

fn book_with(names: &[&str], capabilities: PeopleCapabilities) -> Book {
    let mut book = Book::with(names);
    book.capabilities = capabilities;
    book
}

#[test]
fn a_menu_offers_only_what_the_source_can_do() {
    let (_, s) = state_with(book_with(&["ada"], PeopleCapabilities::read_only()));
    assert!(ids(&s.person_menu()).is_empty(), "read only: no line");
    assert_eq!(ids(&s.empty_menu()), [SELECT], "choosing is always possible");
    let (_, s) = state_with(book_with(&["ada"], PeopleCapabilities::all()));
    assert_eq!(ids(&s.person_menu()), [EDIT, DELETE]);
    assert_eq!(ids(&s.empty_menu()), [NEW, SELECT]);
    let all = PeopleCapabilities::all().with_export().with_merge();
    let (_, s) = state_with(book_with(&["ada"], all));
    assert_eq!(ids(&s.person_menu()), [EDIT, EXPORT, MERGE, DELETE]);
}

#[test]
fn merge_is_locked_with_a_reason_until_two_are_marked() {
    let all = PeopleCapabilities::all().with_merge();
    let (_, mut s) = state_with(book_with(&["ada", "bob"], all));
    let merge = |s: &PeopleState| {
        s.commands()
            .into_iter()
            .find(|c| c.id.as_str() == MERGE)
            .map(|c| c.state)
    };
    s.toggle_marking();
    s.toggle_mark("ada");
    let one = merge(&s).unwrap();
    assert!(one.is_locked(), "one marked is not enough");
    assert!(one.reason().is_some(), "a locked line always says why");
    s.toggle_mark("bob");
    assert_eq!(merge(&s), Some(State::Enabled));
}

#[test]
fn a_locked_line_runs_nothing() {
    let all = PeopleCapabilities::all().with_merge();
    let (_, mut s) = state_with(book_with(&["ada", "bob"], all));
    assert_eq!(s.run_command(MERGE, None), Ran::Nothing);
    // Editing while editing is locked too: the second press of a double click starts nothing new.
    s.begin_edit();
    let before = s.editing().map(|e| e.id);
    assert_eq!(s.run_command(EDIT, None), Ran::Nothing);
    assert_eq!(s.editing().map(|e| e.id), before);
}

#[test]
fn delete_from_the_menu_asks_first() {
    let (book, mut s) = state_with(Book::with(&["ada", "bob"]));
    assert_eq!(s.run_command(DELETE, None), Ran::Done);
    assert!(s.delete_asked());
    assert!(book.name("ada").is_some(), "asking deletes nothing");
    assert_eq!(s.run_command("people.unknown", None), Ran::Nothing);
}

#[test]
fn copy_and_mail_go_where_they_belong() {
    let (_, mut s) = state_with(Book::with(&["ada"]));
    assert_eq!(
        s.run_command(COPY, Some("ada@x.org")),
        Ran::Copy("ada@x.org".into()),
        "the workbench copies by itself"
    );
    assert_eq!(s.run_command(COPY, None), Ran::Nothing, "nothing to copy");
    let ran = s.run_command(MAIL, Some("ada@x.org"));
    let Ran::Hand(request) = ran else {
        panic!("a mail is the program's: {ran:?}")
    };
    assert_eq!(request.id, MAIL);
    assert_eq!(request.value.as_deref(), Some("ada@x.org"));
    assert_eq!(request.keys, ["ada"]);
    s.copied("ada@x.org");
    assert_eq!(s.notice(), Some("Copied: ada@x.org"));
}

#[test]
fn export_acts_on_the_marked_people_else_on_the_one_shown() {
    let all = PeopleCapabilities::all().with_export();
    let (_, mut s) = state_with(book_with(&["ada", "bob", "cy"], all));
    let keys = |s: &mut PeopleState| match s.run_command(EXPORT, None) {
        Ran::Hand(PeopleRequest { keys, .. }) => keys,
        other => panic!("{other:?}"),
    };
    assert_eq!(keys(&mut s), ["ada"]);
    s.toggle_marking();
    assert_eq!(keys(&mut s), ["ada"], "nobody marked yet: the person shown");
    s.toggle_mark("cy");
    s.toggle_mark("bob");
    assert_eq!(keys(&mut s), ["cy", "bob"], "in the order of marking");
}

#[test]
fn choosing_marks_and_unmarks_and_a_press_in_the_mode_does_not_open() {
    let (_, mut s) = state_with(Book::with(&["ada", "bob"]));
    s.toggle_mark("bob");
    assert!(s.marked().is_empty(), "outside the mode nothing is marked");
    s.run_command(SELECT, None);
    assert!(s.marking());
    s.run_command(&mark_id("bob"), None);
    assert!(s.is_marked("bob"));
    assert_eq!(s.selected(), Some("ada"), "the person shown stays");
    s.run_command(&mark_id("bob"), None);
    assert!(!s.is_marked("bob"));
}

#[test]
fn leaving_the_mode_forgets_the_marks() {
    let (_, mut s) = state_with(Book::with(&["ada", "bob"]));
    s.toggle_marking();
    s.toggle_mark("bob");
    s.toggle_marking();
    s.toggle_marking();
    assert!(s.marked().is_empty());
}

#[test]
fn a_group_narrows_the_list_and_everyone_brings_it_back() {
    let book = Book::with(&["ada", "bob", "cy"]);
    *book.groups.lock().unwrap() = vec![("team".into(), "Team".into(), vec!["bob".into(), "cy".into()])];
    let (_, mut s) = state_with(book);
    assert_eq!(s.list_title(), "All contacts");
    assert_eq!(s.rows().len(), 3);
    s.run_command(&group_id(Some("team")), None);
    let keys: Vec<&str> = s.rows().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["bob", "cy"]);
    assert_eq!(s.list_title(), "Team", "the title of the list is the group");
    s.run_command(&group_id(None), None);
    assert_eq!(s.rows().len(), 3);
    assert_eq!(s.list_title(), "All contacts");
}

#[test]
fn the_menu_of_the_list_chooses_one_group_of_several() {
    let book = Book::with(&["ada"]);
    *book.groups.lock().unwrap() = vec![
        ("a".into(), "Alpha".into(), vec![]),
        ("b".into(), "Beta".into(), vec![]),
    ];
    let (_, mut s) = state_with(book);
    s.set_group(Some("b"));
    let on: Vec<(String, bool)> = s
        .list_menu()
        .iter()
        .filter_map(|i| match i {
            Resolved::Command(c) => Some((
                c.id.as_str().to_owned(),
                matches!(&c.toggle, fs_components_commands::Toggle::Radio { on: true, .. }),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(
        on,
        [
            (SELECT.to_owned(), false),
            (group_id(None), false),
            (group_id(Some("a")), false),
            (group_id(Some("b")), true),
        ]
    );
    let (_, plain) = state_with(Book::with(&["ada"]));
    assert_eq!(ids(&plain.list_menu()), [SELECT], "no groups, no filter");
}

#[test]
fn the_keymap_runs_the_shortcuts_and_leaves_delete_to_lists() {
    let (_, s) = state_with(Book::with(&["ada"]));
    let keymap = s.keymap(Platform::Linux);
    let ctrl = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    let press = |m: Modifiers, key: &str| KeyPress::new(m, key).unwrap();
    assert_eq!(
        keymap.dispatch(&press(ctrl, "n"), FocusContext::List),
        Outcome::Run(NEW.into())
    );
    assert_eq!(
        keymap.dispatch(&press(ctrl, "e"), FocusContext::List),
        Outcome::Run(EDIT.into())
    );
    let delete = press(Modifiers::default(), "Delete");
    assert_eq!(keymap.dispatch(&delete, FocusContext::List), Outcome::Run(DELETE.into()));
    assert_eq!(
        keymap.dispatch(&delete, FocusContext::TextField),
        Outcome::NoMatch,
        "in a text field Delete belongs to the text"
    );
    assert_eq!(keymap.dispatch(&delete, FocusContext::Global), Outcome::NoMatch);
}

#[test]
fn every_literal_shortcut_parses_so_no_command_loses_its_key() {
    let all = PeopleCapabilities::all().with_export().with_merge();
    let (_, s) = state_with(book_with(&["ada"], all));
    let with_key: Vec<String> = s
        .commands()
        .iter()
        .filter(|c| !c.shortcuts.is_empty())
        .map(|c| c.id.as_str().to_owned())
        .collect();
    assert_eq!(with_key, [NEW, EDIT, DELETE, EXPORT, MERGE]);
}

#[test]
fn a_field_says_what_it_holds_and_a_guess_is_only_a_guess() {
    assert_eq!(FieldKind::guess("ada@example.org"), FieldKind::Mail);
    assert_eq!(FieldKind::guess("+49 30 1234567"), FieldKind::Phone);
    assert_eq!(FieldKind::guess("12 34"), FieldKind::Text, "too few digits");
    assert_eq!(FieldKind::guess("ada @ example"), FieldKind::Text);
    assert_eq!(FieldKind::guess("Ada Lovelace"), FieldKind::Text);
    let said = PersonField::new("Home", "10 Downing Street").kind(FieldKind::Address);
    assert_eq!(said.effective_kind(), FieldKind::Address);
    let said_text = PersonField::new("Note", "ada@example.org").kind(FieldKind::Date);
    assert_eq!(said_text.effective_kind(), FieldKind::Date, "what the source says wins");
    assert_eq!(PersonField::new("Mail", "ada@example.org").effective_kind(), FieldKind::Mail);
}

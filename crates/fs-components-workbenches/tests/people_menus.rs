//! The context menus of the people workbench as a widget: what a menu acts on, which lines it offers, what is handed to the program.
//!
//! Tests: input-level tests through `PeopleWidget::open_menu` and the hooks; no window (the program's shell opens the menu).
//!
//! Mutants (each must turn a test red):
//! - a press on a row outside the selection leaves the old selection as the target -> `a_row_outside_the_selection_becomes_the_selection`
//! - a press on a row inside the selection acts on that row only                    -> `a_row_inside_the_marked_people_acts_on_all_of_them`
//! - empty space offers the lines of a person                                       -> `empty_space_offers_new_and_choose_and_acts_on_nobody`
//! - the trigger is replaced by a pointer at the origin                             -> `the_trigger_reaches_the_program_unchanged`
//! - a request is not handed to the program                                         -> `a_mail_is_handed_to_the_program`
//! - the keymap is not asked                                                        -> `a_shortcut_on_a_row_runs_its_command`
//! - a menu with no lines is still handed out                                       -> `a_menu_with_no_line_is_not_asked_for`

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::{Arc, Mutex};

use fs_components_commands::{Area, Point, Resolved, Target, Trigger};
use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_workbenches::people::{
    MAIL, MenuFor, PeopleCapabilities, PeopleLabels, PeopleRequest, PeopleWidget,
};
use people_common::Book;

type Seen = Arc<Mutex<Vec<(Trigger, Target, Vec<String>)>>>;

fn widget(book: Book) -> (PeopleWidget, Seen) {
    let seen: Seen = Arc::default();
    let log = Arc::clone(&seen);
    let w = PeopleWidget::new(Arc::new(book), PeopleLabels::default()).on_context(Arc::new(
        move |trigger, target, items| {
            let ids = items
                .iter()
                .filter_map(|i| match i {
                    Resolved::Command(c) => Some(c.id.as_str().to_owned()),
                    _ => None,
                })
                .collect();
            log.lock().unwrap().push((*trigger, target.clone(), ids));
        },
    ));
    (w, seen)
}

fn pointer(x: f32, y: f32) -> Trigger {
    Trigger::Pointer {
        at: Point { x, y },
        shift: false,
    }
}

fn selected(w: &PeopleWidget) -> Option<String> {
    w.with_state(|s| s.selected().map(str::to_owned))
}

#[test]
fn a_row_outside_the_selection_becomes_the_selection() {
    let (w, seen) = widget(Book::with(&["ada", "bob"]));
    assert_eq!(selected(&w).as_deref(), Some("ada"));
    w.open_menu(&MenuFor::Row("bob".into()), pointer(5.0, 5.0));
    assert_eq!(
        selected(&w).as_deref(),
        Some("bob"),
        "the row under the pointer is shown"
    );
    let seen = seen.lock().unwrap();
    assert_eq!(seen[0].1.ids, ["bob"]);
    assert_eq!(seen[0].1.kind, "person");
    assert_eq!(seen[0].2, ["people.edit", "people.delete"]);
}

#[test]
fn a_row_inside_the_marked_people_acts_on_all_of_them() {
    let (w, seen) = widget(Book::with(&["ada", "bob", "cy"]));
    w.with_state(|s| {
        s.toggle_marking();
        s.toggle_mark("ada");
        s.toggle_mark("bob");
    });
    w.open_menu(&MenuFor::Row("bob".into()), pointer(1.0, 1.0));
    let seen = seen.lock().unwrap();
    assert_eq!(seen[0].1.ids, ["ada", "bob"]);
    assert_eq!(
        selected(&w).as_deref(),
        Some("ada"),
        "the person shown does not change"
    );
    assert_eq!(seen[0].2, ["people.delete"], "Edit is for one person only");
}

#[test]
fn empty_space_offers_new_and_choose_and_acts_on_nobody() {
    let (w, seen) = widget(Book::with(&["ada"]));
    w.open_menu(&MenuFor::Empty, pointer(300.0, 500.0));
    let seen = seen.lock().unwrap();
    assert_eq!(seen[0].1.ids, [] as [String; 0]);
    assert_eq!(seen[0].2, ["people.new", "people.select"]);
}

#[test]
fn the_trigger_reaches_the_program_unchanged() {
    let (w, seen) = widget(Book::with(&["ada"]));
    let key = Trigger::MenuKey {
        element: Area {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 56.0,
        },
    };
    w.open_menu(&MenuFor::Row("ada".into()), key);
    w.open_menu(&MenuFor::Person, pointer(7.0, 9.0));
    let seen = seen.lock().unwrap();
    assert_eq!(seen[0].0, key);
    assert_eq!(seen[1].0, pointer(7.0, 9.0));
    assert_eq!(
        seen[1].1.ids,
        ["ada"],
        "the three dots act on the person shown"
    );
}

#[test]
fn a_menu_with_no_line_is_not_asked_for() {
    let mut book = Book::with(&["ada"]);
    book.capabilities = PeopleCapabilities::read_only();
    let (w, seen) = widget(book);
    w.open_menu(&MenuFor::Row("ada".into()), pointer(1.0, 1.0));
    assert!(
        seen.lock().unwrap().is_empty(),
        "read only has no line for a person"
    );
}

#[test]
fn without_a_hook_a_right_click_still_shows_the_row() {
    let w = PeopleWidget::new(
        Arc::new(Book::with(&["ada", "bob"])),
        PeopleLabels::default(),
    );
    w.open_menu(&MenuFor::Row("bob".into()), pointer(1.0, 1.0));
    assert_eq!(selected(&w).as_deref(), Some("bob"));
}

#[test]
fn a_mail_is_handed_to_the_program() {
    let got: Arc<Mutex<Vec<PeopleRequest>>> = Arc::default();
    let log = Arc::clone(&got);
    let w = PeopleWidget::new(Arc::new(Book::with(&["ada"])), PeopleLabels::default())
        .on_request(Arc::new(move |r| log.lock().unwrap().push(r.clone())));
    w.run(MAIL, Some("ada@x.org"));
    let got = got.lock().unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].value.as_deref(), Some("ada@x.org"));
    assert_eq!(got[0].keys, ["ada"]);
}

#[test]
fn a_shortcut_on_a_row_runs_its_command() {
    let w = PeopleWidget::new(
        Arc::new(Book::with(&["ada", "bob"])),
        PeopleLabels::default(),
    );
    // Ctrl+E on the row of bob: bob is shown first, then edited.
    let mut event = KeyEvent::new(Key::Other(69));
    event.ctrl = true;
    w.key(Some("bob"), &event);
    assert_eq!(selected(&w).as_deref(), Some("bob"));
    assert!(w.with_state(|s| s.editing().is_some_and(|e| e.key.as_deref() == Some("bob"))));
    // A key nobody knows does nothing.
    w.key(None, &KeyEvent::new(Key::Other(200)));
}

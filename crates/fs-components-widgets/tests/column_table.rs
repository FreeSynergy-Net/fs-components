//! The model of the table of columns: saving is blocked with the reason, a row with cards needs a target, the limit cell is a
//! number or empty, rows move.
//!
//! Pattern: Model tests with a program's check as a Stub (a Mutex-free closure over the rows).
//!
//! Mutants: Save does not look at the problems (`save_is_blocked_with_the_first_reason`); a row with cards is removed without a
//! target (`a_row_with_cards_needs_a_target`); "12abc" is taken as 12 or as no limit (`the_limit_is_a_number_or_empty`); `move_row`
//! inserts at the old place (`rows_move`); the callback of Save runs although blocked (`save_runs_the_program_only_when_allowed`).

use std::sync::{Arc, Mutex};

use fs_components_widgets::column_table::{
    AllowAll, ColumnCheck, ColumnRow, ColumnTable, Problem, RemoveError, Rows, parse_wip,
};

/// The program says: every status needs a column, and "ghost" is no status.
struct Check;

impl ColumnCheck for Check {
    fn check(&self, rows: &[ColumnRow]) -> Vec<Problem> {
        rows.iter()
            .position(|r| r.status == "ghost")
            .map(|n| Problem {
                row: Some(n),
                text: "Status ghost has no column.".into(),
            })
            .into_iter()
            .collect()
    }
}

fn rows() -> Rows {
    Rows::new(vec![
        ColumnRow::new("todo", "To do").status("open").cards(3),
        ColumnRow::new("doing", "Doing")
            .status("running")
            .wip(2)
            .cards(1),
        ColumnRow::new("done", "Done").status("done"),
    ])
}

#[test]
fn save_is_blocked_with_the_first_reason() {
    let mut r = rows();
    assert!(r.save(&Check).allowed);
    r.set_status(1, "ghost");
    let s = r.save(&Check);
    assert!(!s.allowed);
    assert_eq!(s.reason.as_deref(), Some("Status ghost has no column."));
    // the table's own check: no name
    let mut empty = rows();
    empty.set_name(0, "   ");
    let s = empty.save(&AllowAll);
    assert!(!s.allowed);
    assert_eq!(s.reason.as_deref(), Some("A column needs a name."));
}

#[test]
fn a_row_with_cards_needs_a_target() {
    let mut r = rows();
    assert_eq!(r.remove(0, None), Err(RemoveError::NeedsTarget(3)));
    assert_eq!(r.remove(0, Some(0)), Err(RemoveError::BadTarget));
    assert_eq!(r.remove(0, Some(9)), Err(RemoveError::BadTarget));
    assert_eq!(r.rows().len(), 3, "nothing was removed");
    assert_eq!(r.remove(0, Some(1)), Ok(()));
    assert_eq!(r.rows()[0].key, "doing");
    assert_eq!(r.rows()[0].cards, 4, "the cards went to the target");
    assert_eq!(
        r.remove(1, None),
        Ok(()),
        "an empty row goes without a target"
    );
    assert_eq!(
        r.remove(0, None),
        Err(RemoveError::LastRow),
        "the last row stays, cards or not"
    );
    assert_eq!(r.remove(7, None), Err(RemoveError::NoSuchRow));
}

#[test]
fn the_last_row_stays() {
    let mut r = Rows::new(vec![ColumnRow::new("only", "Only")]);
    assert_eq!(r.remove(0, None), Err(RemoveError::LastRow));
}

#[test]
fn the_limit_is_a_number_or_empty() {
    assert_eq!(parse_wip(""), Ok(None));
    assert_eq!(parse_wip("  "), Ok(None));
    assert_eq!(parse_wip("5"), Ok(Some(5)));
    assert_eq!(parse_wip(" 12 "), Ok(Some(12)));
    assert!(parse_wip("12abc").is_err());
    assert!(parse_wip("-1").is_err());
    let mut r = rows();
    assert!(r.set_wip(0, "7"));
    assert_eq!(r.rows()[0].wip, Some(7));
    assert!(!r.set_wip(0, "many"));
    assert_eq!(r.rows()[0].wip, Some(7), "a bad text changes nothing");
    assert!(r.set_wip(0, ""));
    assert_eq!(r.rows()[0].wip, None);
}

#[test]
fn rows_move() {
    let mut r = rows();
    assert!(r.move_row(0, 2));
    let keys: Vec<_> = r.rows().iter().map(|x| x.key.as_str()).collect();
    assert_eq!(keys, ["doing", "done", "todo"]);
    assert!(!r.move_row(1, 1));
    assert!(!r.move_row(0, 9));
    let n = r.add("New column");
    assert_eq!(r.rows()[n].key, "new-1");
    assert_eq!(r.add("Another"), n + 1);
    assert_eq!(r.rows()[n + 1].key, "new-2", "keys stay fresh");
}

#[test]
fn save_runs_the_program_only_when_allowed() {
    let saved: Arc<Mutex<Vec<usize>>> = Arc::default();
    let sink = Arc::clone(&saved);
    let table = ColumnTable::new(
        rows().rows().to_vec(),
        Arc::new(Check),
        vec![
            ("open".into(), "Open".into()),
            ("running".into(), "Running".into()),
        ],
    )
    .on_save(Arc::new(move |r: &[ColumnRow]| {
        sink.lock().unwrap().push(r.len())
    }));
    assert!(table.save());
    table.edit(|r| r.set_status(2, "ghost"));
    assert!(!table.save(), "blocked");
    assert_eq!(
        *saved.lock().unwrap(),
        [3],
        "the program heard of the first save only"
    );
    assert!(!table.save_state().allowed);
    assert_eq!(table.next_status("open").as_deref(), Some("running"));
    assert_eq!(
        table.next_status("running").as_deref(),
        Some("open"),
        "wraps"
    );
    assert_eq!(
        table.next_status("zzz").as_deref(),
        Some("open"),
        "unknown: the first"
    );
}

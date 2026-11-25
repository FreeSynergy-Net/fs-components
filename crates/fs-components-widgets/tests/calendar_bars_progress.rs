//! The channels `progress` and `depends` of `bars-in-time`: an unreadable `progress` drops only the bar, never the row.
//!
//! Tests: Table tests over `read_bars` (33.3, above 100, negative, text, nothing; several `depends`).

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_widgets::calendar_view::{BarEntry, read_bars};

fn one(pairs: &[(&str, &str)]) -> Option<BarEntry> {
    let input = FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: vec![FormRow::of("r", pairs)],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    };
    let read = read_bars(&input);
    assert!(read.invalid.is_empty(), "the row stays readable");
    read.entries.into_iter().next()
}

fn progress(text: &str) -> Option<u8> {
    one(&[("start", "2026-11-25"), ("progress", text)])
        .expect("the row is kept")
        .progress
}

#[test]
fn progress_is_rounded_cut_and_never_costs_the_row() {
    assert_eq!(progress("33.3"), Some(33));
    assert_eq!(progress("33.5"), Some(34));
    assert_eq!(progress("150"), Some(100));
    assert_eq!(progress("-20"), Some(0));
    assert_eq!(progress(" 40 "), Some(40));
    assert_eq!(progress("half"), None);
    assert_eq!(progress("NaN"), None);
}

#[test]
fn no_progress_is_no_bar() {
    let e = one(&[("start", "2026-11-25")]).unwrap();
    assert_eq!(e.progress, None);
}

#[test]
fn depends_keeps_all_keys_in_order() {
    let row = FormRow {
        key: "r".to_owned(),
        values: vec![
            ("start".to_owned(), vec!["2026-11-25".to_owned()]),
            ("depends".to_owned(), vec!["a".to_owned(), "b".to_owned()]),
        ],
        may: Vec::new(),
    };
    let input = FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows: vec![row],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    };
    let e = read_bars(&input).entries.pop().unwrap();
    assert_eq!(e.depends, vec!["a".to_owned(), "b".to_owned()]);
    let none = one(&[("start", "2026-11-25")]).unwrap().depends;
    assert_eq!(none.len(), 0);
}

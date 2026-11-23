//! SW-2: the write port. `NoWrite` allows nothing and refuses every call; `WriteDone` carries the `result` of an invoke.
//!
//! Pattern: none (tests).
use fs_components_core::form_write::{
    FormWrite, NoWrite, WriteArg, WriteDone, WriteRefused, WriteValue,
};

#[test]
fn no_write_may_nothing_and_refuses_with_a_code() {
    let w = NoWrite;
    assert!(!w.can("move", Some("card-1")));
    assert!(!w.can("create", None));
    let e = w.send("move", Some("card-1"), &[]).unwrap_err();
    assert_eq!(e, WriteRefused::new("unsupported"));
}

#[test]
fn args_are_named_and_typed() {
    assert_eq!(
        WriteArg::text("column", "doing").value,
        WriteValue::Text("doing".into())
    );
    assert_eq!(
        WriteArg::number("index", 1.0).value,
        WriteValue::Number(1.0)
    );
}

#[test]
fn a_done_call_names_the_record_and_a_delete_names_none() {
    let made = WriteDone {
        record: Some("t-9".into()),
        text: None,
    };
    let deleted = WriteDone::default();
    assert_eq!(made.record.as_deref(), Some("t-9"));
    assert_eq!(deleted.record, None);
}

//! Tests: the three commands `notification.tell/ask/close`: the description in `fs-components-commands` and the call in this crate
//! agree, and a call by text does what the facade does (ADR 0106 point 3).
//!
//! Mutants: a parameter is renamed on one side only (`the_description_and_the_call_agree`); `ask` is run as `tell`
//! (`ask_by_command_is_a_question`); a missing title runs anyway (`a_bad_call_is_a_code`); the commands are not reserved
//! (see the tests of `fs-components-commands`).
mod support;

use fs_components_commands::{
    export_blocks, notification_commands,
    standard::{NOTIFICATION_ASK, NOTIFICATION_CLOSE, NOTIFICATION_TELL},
    validate_commands,
};
use fs_notification::{ASK, CLOSE, NotificationCode, TELL};
use support::{all_forms, facade};

#[test]
fn the_description_and_the_call_agree() {
    assert_eq!(
        (TELL, ASK, CLOSE),
        (NOTIFICATION_TELL, NOTIFICATION_ASK, NOTIFICATION_CLOSE)
    );
    let list = notification_commands();
    assert_eq!(validate_commands(&list), vec![]);
    // every parameter name the description has is one `run` reads, and the other way round
    let read = [
        "title",
        "body",
        "urgency",
        "category",
        "scope",
        "replaces",
        "expire-ms",
        "params",
        "actions",
    ];
    for id in [TELL, ASK] {
        let c = list.get(&id.into()).unwrap();
        let mut names: Vec<&str> = c.params.iter().map(|p| p.name.as_str()).collect();
        names.sort_unstable();
        let mut want = read.to_vec();
        want.sort_unstable();
        assert_eq!(names, want, "{id}");
    }
    let close = list.get(&CLOSE.into()).unwrap();
    assert_eq!(close.params.len(), 1);
    assert_eq!(close.params[0].name, "id");
    // a question needs its buttons, a message does not
    let required = |id: &str| {
        list.get(&id.into())
            .unwrap()
            .params
            .iter()
            .find(|p| p.name == "actions")
            .unwrap()
            .required
    };
    assert!(required(ASK));
    assert!(!required(TELL));
}

#[test]
fn the_commands_are_workflow_blocks() {
    let blocks = export_blocks(&notification_commands()).unwrap();
    let ids: Vec<&str> = blocks.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, [TELL, ASK, CLOSE]);
}

#[test]
fn tell_by_command_shows_and_close_by_command_closes() {
    let out = all_forms();
    let n = facade(&out);
    let id = n
        .run(
            TELL,
            &[
                ("title", "saved-as"),
                ("params", "name=report"),
                ("urgency", "low"),
                ("scope", "editor"),
                ("expire-ms", "2000"),
            ],
            0,
        )
        .unwrap()
        .unwrap();
    let open = &n.open()[0];
    assert_eq!(open.notification.title.key(), "saved-as");
    assert_eq!(open.notification.title.params().count(), 1);
    assert_eq!(open.deadline_ms, Some(2000));
    assert_eq!(n.run(CLOSE, &[("id", id.as_str())], 0).unwrap(), None);
    assert!(n.open().is_empty(), "expected nothing");
}

#[test]
fn ask_by_command_is_a_question() {
    let out = all_forms();
    let n = facade(&out);
    let id = n
        .run(
            ASK,
            &[
                ("title", "update-now"),
                ("actions", "now=button-now"),
                ("actions", "later=button-later"),
            ],
            0,
        )
        .unwrap()
        .unwrap();
    assert!(n.open()[0].notification.is_question());
    assert_eq!(n.open()[0].notification.actions.len(), 2);
    assert_eq!(out.lines(), [format!("show modal {}", id.as_str())]);
}

#[test]
fn a_bad_call_is_a_code() {
    let n = facade(&all_forms());
    let bad = |command: &str, args: &[(&str, &str)]| n.run(command, args, 0).unwrap_err().code();
    assert_eq!(bad(TELL, &[]), NotificationCode::BadCommand, "no title");
    assert_eq!(
        bad(TELL, &[("title", "a"), ("urgency", "loud")]),
        NotificationCode::BadCommand
    );
    assert_eq!(
        bad(TELL, &[("title", "a"), ("expire-ms", "soon")]),
        NotificationCode::BadCommand
    );
    assert_eq!(
        bad(TELL, &[("title", "a"), ("params", "novalue")]),
        NotificationCode::BadCommand
    );
    assert_eq!(
        bad(ASK, &[("title", "a")]),
        NotificationCode::ActionsInvalid,
        "a question without buttons"
    );
    assert_eq!(bad(CLOSE, &[]), NotificationCode::BadCommand);
    assert_eq!(
        bad("notification.explode", &[]),
        NotificationCode::BadCommand
    );
    assert_eq!(bad(CLOSE, &[("id", "n77")]), NotificationCode::UnknownId);
    assert!(n.open().is_empty(), "expected nothing");
}

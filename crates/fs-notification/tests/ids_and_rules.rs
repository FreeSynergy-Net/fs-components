//! Tests: ids that belong to one instance (r1 S1), a question without a button is refused even through the public fields (S2), a
//! failure stays (S8).
//!
//! Pattern: fake outlet (`support::Recorder`) + exact clock.
//!
//! Mutants (each was drawn and its test turned red; see BERICHT-a1.md, section r1):
//! - every instance counts from `n1` again (`two_instances_issue_different_ids`);
//! - the mark of the instance is not checked as a name (`a_mark_must_make_a_name`);
//! - `show` does not check the buttons again (`a_question_without_a_button_is_refused_even_through_the_fields`);
//! - a notification of the category `error` expires like a message (`a_failure_stays_until_it_is_closed`);
//! - the answer is not stored, or not under the id (`the_answer_is_readable_by_its_id`); the store grows without bound
//!   (`only_the_newest_outcomes_are_kept`).
mod support;

use std::sync::Arc;

use fs_notification::{
    ActionId, Category, CloseReason, ENDED_KEPT, ERROR_CATEGORY, Expire, NoteId, Notification,
    NotificationCode, Notifications, Outcome, Outlet, Policy,
};
use support::{all_forms, facade, msg, question};

#[test]
fn two_instances_issue_different_ids() {
    let out = all_forms();
    let o: Arc<dyn Outlet> = out.clone();
    let a = Notifications::new(Policy::default(), vec![Arc::clone(&o)]);
    let b = Notifications::new(Policy::default(), vec![o]);
    let ida = a.tell(Notification::tell(msg("x")), 0).unwrap();
    let idb = b.tell(Notification::tell(msg("x")), 0).unwrap();
    assert_ne!(ida, idb, "an id of an earlier run must not hit this one");
    let idc = a.tell(Notification::tell(msg("y")), 0).unwrap();
    assert_ne!(ida, idc);
    // what it issued is a name like any other: it comes back from outside and is found
    assert!(NoteId::new(ida.as_str()).is_ok(), "{}", ida.as_str());
    assert!(a.close(&NoteId::new(ida.as_str()).unwrap()).is_ok());
    // and the other instance does not know it
    assert_eq!(
        b.close(&NoteId::new(idc.as_str()).unwrap())
            .unwrap_err()
            .code(),
        NotificationCode::UnknownId
    );
}

#[test]
fn a_mark_must_make_a_name() {
    let out = all_forms();
    let o: Arc<dyn Outlet> = out.clone();
    for bad in ["N", "a b", "x--", "-x"] {
        let r = Notifications::with_instance(Policy::default(), vec![Arc::clone(&o)], bad);
        assert_eq!(
            r.err().map(|c| c.code()),
            Some(NotificationCode::NameInvalid),
            "{bad:?}"
        );
    }
    let n = Notifications::with_instance(Policy::default(), vec![o], "t-").unwrap();
    assert_eq!(
        n.tell(Notification::tell(msg("x")), 0).unwrap().as_str(),
        "t-1"
    );
}

#[test]
fn a_question_without_a_button_is_refused_even_through_the_fields() {
    let out = all_forms();
    let n = facade(&out);
    let mut q = question("q");
    q.actions.clear();
    assert_eq!(
        n.ask(q, 0).unwrap_err().code(),
        NotificationCode::ActionsInvalid
    );
    let mut twice = question("q");
    twice.actions.push(twice.actions[0].clone());
    assert_eq!(
        n.ask(twice, 0).unwrap_err().code(),
        NotificationCode::ActionsInvalid
    );
    assert!(n.open().is_empty(), "expected nothing");
    assert!(out.lines().is_empty(), "nothing was shown");
}

#[test]
fn a_failure_stays_until_it_is_closed() {
    let out = all_forms();
    let n = facade(&out);
    let error = Category::new(ERROR_CATEGORY).unwrap();
    let failed = n
        .tell(Notification::tell(msg("failed")).category(error.clone()), 0)
        .unwrap(); // urgency normal
    let timed = n
        .tell(
            Notification::tell(msg("failed-soon"))
                .category(error)
                .expire(Expire::After(1000)),
            0,
        )
        .unwrap();
    let plain = n.tell(Notification::tell(msg("saved")), 0).unwrap();
    assert_eq!(
        n.tick(10_000_000),
        vec![timed, plain],
        "an explicit time wins"
    );
    assert_eq!(n.open().len(), 1);
    assert!(n.close(&failed).is_ok());
}

#[test]
fn the_answer_is_readable_by_its_id() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.ask(question("q"), 0).unwrap();
    assert_eq!(n.outcome(&id), None, "open: no outcome yet");
    n.answered(&id, &ActionId::new("later").unwrap()).unwrap();
    assert_eq!(
        n.outcome(&id),
        Some(Outcome {
            reason: CloseReason::Dismissed,
            answer: Some(ActionId::new("later").unwrap())
        })
    );
    // a message closed by the program, one that ran out of time, an id that never was
    let a = n.tell(Notification::tell(msg("a")), 0).unwrap();
    n.close(&a).unwrap();
    let b = n.tell(Notification::tell(msg("b")), 0).unwrap();
    n.tick(10_000_000);
    let reason = |i: &NoteId| n.outcome(i).map(|o| (o.reason, o.answer));
    assert_eq!(reason(&a), Some((CloseReason::Closed, None)));
    assert_eq!(reason(&b), Some((CloseReason::Expired, None)));
    assert_eq!(reason(&NoteId::new("n99").unwrap()), None);
}

#[test]
fn only_the_newest_outcomes_are_kept() {
    let out = all_forms();
    let n = facade(&out);
    let first = n.tell(Notification::tell(msg("a")), 0).unwrap();
    n.close(&first).unwrap();
    for _ in 0..ENDED_KEPT {
        let id = n.tell(Notification::tell(msg("a")), 0).unwrap();
        n.close(&id).unwrap();
    }
    assert_eq!(n.outcome(&first), None, "the oldest is forgotten");
}

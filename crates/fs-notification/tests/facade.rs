//! Tests: the facade `Notifications`: show, replace, expiry, scope, observers with a lifetime, the rule of forms, the
//! answers from outside (ADR 0106 points 2, 4, 8; ADR 0065 point 4).
//!
//! Pattern: fake outlet (`support::Recorder`) + exact clock.
//!
//! Intended mutants (only those marked `drawn` were really drawn and seen red, BERICHT-a1.md; the rest is intent, not proof):
//! - drawn: a critical message expires like a normal one (`critical_stays_until_closed`);
//! - a normal message never expires (`a_normal_message_expires_by_itself`);
//! - drawn: the observer is still called after its `Subscription` is dropped (`an_observer_lives_as_long_as_its_subscription`);
//! - drawn: an answer for an unknown action runs (`an_answer_for_an_unknown_action_is_dropped_and_logged`);
//! - drawn: an id that was not issued here is taken for `replaces` (`an_id_from_outside_is_never_reused`);
//! - drawn: the system outlet gets the values of the texts (`the_system_form_gets_no_parameters`);
//! - leaving a scope closes notifications of another scope, or none (`leaving_a_scope_closes_only_its_own`);
//! - a modal with no modal outlet is lost instead of falling back to a toast (`a_modal_without_an_outlet_becomes_a_toast`);
//! - a question as a toast expires like a message (`a_question_is_a_modal_when_calm_and_a_standing_toast_when_not`);
//! - `tell` accepts a question (`tell_and_ask_keep_their_kinds`).
mod support;

use std::sync::Arc;

use fs_notification::{
    ActionId, Event, Expire, NoteId, Notification, NotificationCode, Scope, Urgency,
};
use fs_types::coded::Param;
use support::{act, all_forms, collect, facade, msg, question};

fn code(
    r: Result<impl std::fmt::Debug, fs_types::coded::Coded<NotificationCode>>,
) -> NotificationCode {
    r.unwrap_err().code()
}

#[test]
fn a_message_is_shown_as_a_toast_and_gets_an_id() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.tell(Notification::tell(msg("saved")), 0).unwrap();
    assert_eq!(id.as_str(), "n1");
    assert_eq!(out.lines(), ["show toast n1"]);
    assert_eq!(n.open().len(), 1);
}

#[test]
fn a_normal_message_expires_by_itself() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.tell(Notification::tell(msg("saved")), 1000).unwrap();
    assert!(n.tick(6999).is_empty(), "expected nothing");
    assert_eq!(n.tick(7000), vec![id]);
    assert!(n.open().is_empty(), "expected nothing");
    assert_eq!(out.lines(), ["show toast n1", "close toast n1"]);
}

#[test]
fn critical_stays_until_closed() {
    let out = all_forms();
    let n = facade(&out);
    let id = n
        .tell(
            Notification::tell(msg("failed")).urgency(Urgency::Critical),
            0,
        )
        .unwrap();
    assert!(n.tick(10_000_000).is_empty(), "expected nothing");
    n.close(&id).unwrap();
    assert!(n.open().is_empty(), "expected nothing");
}

#[test]
fn an_explicit_expiry_wins_over_urgency() {
    let out = all_forms();
    let n = facade(&out);
    let a = n
        .tell(
            Notification::tell(msg("a"))
                .urgency(Urgency::Critical)
                .expire(Expire::After(50)),
            0,
        )
        .unwrap();
    let b = n
        .tell(Notification::tell(msg("b")).expire(Expire::Never), 0)
        .unwrap();
    assert_eq!(n.tick(50), vec![a]);
    assert_eq!(n.open().len(), 1);
    assert_eq!(n.open()[0].id, b);
}

#[test]
fn leaving_a_scope_closes_only_its_own() {
    let out = all_forms();
    let n = facade(&out);
    let list = Scope::new("list").unwrap();
    let a = n
        .tell(Notification::tell(msg("a")).scope(list.clone()), 0)
        .unwrap();
    let _b = n
        .tell(
            Notification::tell(msg("b")).scope(Scope::new("other").unwrap()),
            0,
        )
        .unwrap();
    let _c = n.tell(Notification::tell(msg("c")), 0).unwrap();
    assert_eq!(n.leave_scope(&list), vec![a]);
    assert_eq!(n.open().len(), 2);
}

#[test]
fn an_observer_lives_as_long_as_its_subscription() {
    let out = all_forms();
    let n = facade(&out);
    let (sub, got) = collect(&n);
    let id = n.tell(Notification::tell(msg("a")), 0).unwrap();
    assert_eq!(got.lock().unwrap().len(), 1);
    drop(sub);
    n.close(&id).unwrap();
    n.tell(Notification::tell(msg("b")), 0).unwrap();
    assert_eq!(got.lock().unwrap().len(), 1, "nothing after the drop");
}

#[test]
fn observers_hear_shown_answered_closed_in_order() {
    let out = all_forms();
    let n = facade(&out);
    let (_sub, got) = collect(&n);
    let id = n.ask(question("update"), 0).unwrap();
    n.answered(&id, &ActionId::new("later").unwrap()).unwrap();
    let kinds: Vec<String> = got
        .lock()
        .unwrap()
        .iter()
        .map(|e| match e {
            Event::Shown { forms, .. } => format!("shown {}", forms[0].as_str()),
            Event::Answered { action, .. } => format!("answered {}", action.as_str()),
            Event::Closed { reason, .. } => format!("closed {}", reason.as_str()),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(kinds, ["shown modal", "answered later", "closed dismissed"]);
    assert!(n.open().is_empty(), "expected nothing");
}

#[test]
fn an_observer_may_call_the_facade_again() {
    let out = all_forms();
    let n = facade(&out);
    let again = n.clone();
    let _sub = n.subscribe(Arc::new(move |e: &Event| {
        if let Event::Shown { id, .. } = e {
            let _ = again.close(id);
        }
    }));
    n.tell(Notification::tell(msg("a")), 0).unwrap();
    assert!(
        n.open().is_empty(),
        "closed from inside the observer, no deadlock"
    );
}

#[test]
fn an_answer_for_an_unknown_action_is_dropped_and_logged() {
    let out = all_forms();
    let n = facade(&out);
    let (_sub, got) = collect(&n);
    let id = n.ask(question("update"), 0).unwrap();
    let err = n
        .answered(&id, &ActionId::new("delete-everything").unwrap())
        .unwrap_err();
    assert_eq!(err.code(), NotificationCode::UnknownAction);
    assert_eq!(n.open().len(), 1, "the question stays open");
    assert!(
        got.lock()
            .unwrap()
            .iter()
            .all(|e| !matches!(e, Event::Answered { .. })),
        "no answer reached anybody"
    );
    let log = n.rejected();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].code(), NotificationCode::UnknownAction);
    assert!(
        log[0]
            .params()
            .any(|(k, v)| k == "action" && *v == Param::Text("delete-everything".into()))
    );
}

#[test]
fn an_answer_for_an_unknown_id_is_dropped_and_logged() {
    let out = all_forms();
    let n = facade(&out);
    let (_sub, got) = collect(&n);
    let err = n
        .answered(&NoteId::new("n99").unwrap(), &ActionId::new("now").unwrap())
        .unwrap_err();
    assert_eq!(err.code(), NotificationCode::UnknownId);
    assert_eq!(
        code(n.dismissed(&NoteId::new("n99").unwrap())),
        NotificationCode::UnknownId
    );
    assert!(got.lock().unwrap().is_empty(), "expected nothing");
    assert_eq!(n.rejected().len(), 2);
}

#[test]
fn the_log_of_dropped_answers_is_bounded() {
    let out = all_forms();
    let n = facade(&out);
    for i in 0..(fs_notification::REJECTED_KEPT + 20) {
        let _ = n.dismissed(&NoteId::new(&format!("x{i}")).unwrap());
    }
    assert_eq!(n.rejected().len(), fs_notification::REJECTED_KEPT);
}

#[test]
fn an_id_from_outside_is_never_reused() {
    let out = all_forms();
    let n = facade(&out);
    let id = n
        .tell(
            Notification::tell(msg("a")).replaces(NoteId::new("n42").unwrap()),
            0,
        )
        .unwrap();
    assert_eq!(id.as_str(), "n1", "a fresh id, not the one from outside");
}

#[test]
fn replacing_an_open_one_keeps_the_id_and_the_place() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.tell(Notification::tell(msg("progress-1")), 0).unwrap();
    let again = n
        .tell(
            Notification::tell(msg("progress-2")).replaces(id.clone()),
            100,
        )
        .unwrap();
    assert_eq!(again, id);
    assert_eq!(n.open().len(), 1);
    assert_eq!(n.open()[0].notification.title.key(), "progress-2");
    assert_eq!(out.lines(), ["show toast n1", "show toast n1"]);
}

#[test]
fn tell_and_ask_keep_their_kinds() {
    let out = all_forms();
    let n = facade(&out);
    assert_eq!(code(n.tell(question("q"), 0)), NotificationCode::WrongKind);
    assert_eq!(
        code(n.ask(Notification::tell(msg("m")), 0)),
        NotificationCode::WrongKind
    );
    assert!(out.lines().is_empty(), "expected nothing");
}

#[test]
fn a_question_needs_an_action_and_actions_are_unique() {
    assert_eq!(
        Notification::ask(msg("q"), vec![]).unwrap_err().code(),
        NotificationCode::ActionsInvalid
    );
    assert_eq!(
        Notification::ask(msg("q"), vec![act("a"), act("a")])
            .unwrap_err()
            .code(),
        NotificationCode::ActionsInvalid
    );
    assert_eq!(
        Notification::tell(msg("m"))
            .action(act("a"))
            .unwrap()
            .action(act("a"))
            .unwrap_err()
            .code(),
        NotificationCode::ActionsInvalid
    );
}

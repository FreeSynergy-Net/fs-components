//! Tests: the facade `Notifications`: show, replace, expiry, scope, observers with a lifetime, the rule of forms, the
//! answers from outside (ADR 0106 points 2, 4, 8; ADR 0065 point 4).
//!
//! Pattern: fake outlet (`support::Recorder`) + exact clock.
//!
//! Mutants (each has a test that turns red):
//! - a critical message expires like a normal one (`critical_stays_until_closed`);
//! - a normal message never expires (`a_normal_message_expires_by_itself`);
//! - the observer is still called after its `Subscription` is dropped (`an_observer_lives_as_long_as_its_subscription`);
//! - an answer for an unknown action runs (`an_answer_for_an_unknown_action_is_dropped_and_logged`);
//! - an id that was not issued here is taken for `replaces` (`an_id_from_outside_is_never_reused`);
//! - the system outlet gets the parameters of the texts (`the_system_form_gets_no_parameters`);
//! - leaving a scope closes notifications of another scope, or none (`leaving_a_scope_closes_only_its_own`);
//! - a modal with no modal outlet is lost instead of falling back to a toast (`a_modal_without_an_outlet_becomes_a_toast`);
//! - a question as a toast expires like a message (`a_question_is_a_modal_when_calm_and_a_standing_toast_when_not`);
//! - `tell` accepts a question (`tell_and_ask_keep_their_kinds`).
mod support;

use std::sync::Arc;

use fs_notification::{
    ActionId, CloseReason, Event, Expire, Form, NoteId, Notification, NotificationCode,
    Notifications, Outlet, Policy, Scope, Situation, Urgency, Window,
};
use fs_types::coded::Param;
use support::{Recorder, act, all_forms, collect, facade, msg, question};

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
    assert!(n.tick(6999).is_empty());
    assert_eq!(n.tick(7000), vec![id]);
    assert!(n.open().is_empty());
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
    assert!(n.tick(10_000_000).is_empty());
    n.close(&id).unwrap();
    assert!(n.open().is_empty());
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
    assert!(n.open().is_empty());
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
    assert!(got.lock().unwrap().is_empty());
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
    assert!(out.lines().is_empty());
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

#[test]
fn a_question_is_a_modal_when_calm_and_a_standing_toast_when_not() {
    let out = all_forms();
    let n = facade(&out);
    n.ask(question("q1"), 0).unwrap();
    n.set_situation(Situation {
        window: Window::Front,
        calm: false,
    });
    let id = n.ask(question("q2"), 0).unwrap();
    assert_eq!(out.lines(), ["show modal n1", "show toast n2"]);
    // the standing toast has the same buttons and does not expire: a question stays until it is answered
    assert!(n.tick(10_000_000).is_empty());
    let open = n.open();
    assert_eq!(open[1].id, id);
    assert_eq!(open[1].notification.actions.len(), 2);
}

#[test]
fn a_modal_without_an_outlet_becomes_a_toast() {
    let out = Recorder::new(&[Form::Toast]);
    let n = facade(&out);
    n.ask(question("q"), 0).unwrap();
    assert_eq!(out.lines(), ["show toast n1"]);
}

#[test]
fn the_system_form_is_added_when_the_window_is_away_and_instead_when_there_is_none() {
    let out = all_forms();
    let n = facade(&out);
    n.set_situation(Situation {
        window: Window::Away,
        calm: true,
    });
    let a = n.tell(Notification::tell(msg("a")), 0).unwrap();
    n.set_situation(Situation {
        window: Window::Absent,
        calm: true,
    });
    let b = n.tell(Notification::tell(msg("b")), 0).unwrap();
    let forms = |id: &NoteId| n.open().into_iter().find(|o| &o.id == id).unwrap().forms;
    assert_eq!(forms(&a), [Form::Toast, Form::System]);
    assert_eq!(forms(&b), [Form::System]);
}

#[test]
fn no_window_and_no_system_form_is_a_code_not_silence() {
    let out = all_forms();
    let n = Notifications::new(
        Policy {
            system: fs_notification::SystemWhen::Never,
            ..Policy::default()
        },
        vec![out.clone() as Arc<dyn Outlet>],
    );
    n.set_situation(Situation {
        window: Window::Absent,
        calm: true,
    });
    assert_eq!(
        code(n.tell(Notification::tell(msg("a")), 0)),
        NotificationCode::NoForm
    );
    assert!(n.open().is_empty());
}

#[test]
fn nothing_that_can_show_it_is_a_code() {
    let n = Notifications::new(Policy::default(), vec![]);
    assert_eq!(
        code(n.tell(Notification::tell(msg("a")), 0)),
        NotificationCode::NoOutlet
    );
    let broken = Recorder::failing(&[Form::Toast]);
    let n = facade(&broken);
    assert_eq!(
        code(n.tell(Notification::tell(msg("a")), 0)),
        NotificationCode::ShowFailed
    );
    assert!(n.open().is_empty(), "what was shown nowhere is not open");
}

#[test]
fn the_system_form_gets_no_parameters() {
    let system = Recorder::new(&[Form::System]);
    let window = Recorder::new(&[Form::Toast]);
    let outlets: Vec<Arc<dyn Outlet>> = vec![window.clone(), system.clone()];
    let n = Notifications::new(
        Policy {
            system: fs_notification::SystemWhen::Always,
            ..Policy::default()
        },
        outlets,
    );
    let title = msg("invite").with_text("name", "Maria Example").unwrap();
    n.tell(Notification::tell(title), 0).unwrap();
    let sys = system.seen.lock().unwrap();
    assert_eq!(
        sys[0].1.title.params().count(),
        0,
        "redacted for the system"
    );
    assert_eq!(sys[0].1.title.key(), "invite", "the text itself stays");
    let win = window.seen.lock().unwrap();
    assert_eq!(
        win[0].1.title.params().count(),
        1,
        "the window shows it whole"
    );
}

#[test]
fn closing_reaches_every_outlet_that_showed_it() {
    let system = Recorder::new(&[Form::System]);
    let window = Recorder::new(&[Form::Toast]);
    let outlets: Vec<Arc<dyn Outlet>> = vec![window.clone(), system.clone()];
    let n = Notifications::new(
        Policy {
            system: fs_notification::SystemWhen::Always,
            ..Policy::default()
        },
        outlets,
    );
    let id = n.tell(Notification::tell(msg("a")), 0).unwrap();
    n.close(&id).unwrap();
    assert_eq!(window.lines(), ["show toast n1", "close toast n1"]);
    assert_eq!(system.lines(), ["show system n1", "close system n1"]);
    assert_eq!(
        code(n.close(&id)),
        NotificationCode::UnknownId,
        "a second close is a code"
    );
}

#[test]
fn the_person_closing_it_is_dismissed_the_program_closing_it_is_closed() {
    let out = all_forms();
    let n = facade(&out);
    let (_s, got) = collect(&n);
    let a = n.tell(Notification::tell(msg("a")), 0).unwrap();
    let b = n.tell(Notification::tell(msg("b")), 0).unwrap();
    n.dismissed(&a).unwrap();
    n.close(&b).unwrap();
    let reasons: Vec<CloseReason> = got
        .lock()
        .unwrap()
        .iter()
        .filter_map(|e| {
            if let Event::Closed { reason, .. } = e {
                Some(*reason)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(reasons, [CloseReason::Dismissed, CloseReason::Closed]);
}

#[test]
fn a_new_rule_applies_to_the_next_notification() {
    let out = all_forms();
    let n = facade(&out);
    n.set_policy(Policy {
        modal: fs_notification::ModalWhen::Never,
        ..Policy::default()
    });
    n.ask(question("q"), 0).unwrap();
    assert_eq!(out.lines(), ["show toast n1"]);
}

//! Tests: which form a notification takes and where it goes: the rule at work through the facade, the fallback of a modal, the
//! system form without parameters, closing in every outlet, a changed rule (ADR 0106 points 4 and 8).
//!
//! Pattern: fake outlet (`support::Recorder`).
//!
//! Intended mutants (none drawn unless marked `drawn`; BERICHT-a1.md lists what was drawn): a question as a toast expires like a message (`a_question_is_a_modal_when_calm_and_a_standing_toast_when_not`); a modal with no
//! modal outlet is lost instead of falling back to a toast (`a_modal_without_an_outlet_becomes_a_toast`); drawn: the system outlet gets the
//! parameters of the texts (`the_system_form_gets_no_parameters`); drawn (as "the modal rule never is ignored"): the old rule stays after `set_policy` (`a_new_rule_applies_to_the_next_notification`).
mod support;

use std::sync::Arc;

use fs_types::coded::Param;

use fs_notification::{
    CloseReason, Event, Form, NoteId, Notification, NotificationCode, Notifications, Outlet,
    Policy, Situation, Window,
};
use support::{Recorder, all_forms, collect, facade, msg, question};

fn code(
    r: Result<impl std::fmt::Debug, fs_types::coded::Coded<NotificationCode>>,
) -> NotificationCode {
    r.unwrap_err().code()
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
    assert!(n.tick(10_000_000).is_empty(), "expected nothing");
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
    assert!(n.open().is_empty(), "expected nothing");
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
    let masked: Vec<_> = sys[0].1.title.params().collect();
    assert_eq!(
        masked,
        [("name", &Param::Text(fs_notification::MASK.into()))],
        "the name stays, the value is masked for the system"
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
    let n = Notifications::with_instance(
        Policy {
            system: fs_notification::SystemWhen::Always,
            ..Policy::default()
        },
        outlets,
        "n",
    )
    .unwrap();
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

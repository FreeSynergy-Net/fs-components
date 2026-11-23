//! Tests: the safety paths of the facade (ADR 0106 point 8; review r1 M1, M2, M3): replacing closes the old forms, an answer counts
//! once and never after the close, the redacted text is a sentence and not a key.
//!
//! Pattern: fake outlet (`support::Recorder`) + observers that call the facade again (deterministic, no timing) + a few threads.
//!
//! Mutants (each was drawn and its test turned red; see BERICHT-a1.md, section r1):
//! - M1: the forms of the old notification are not closed after a replace (`replacing_closes_the_forms_that_are_gone`,
//!   `replacing_closes_the_system_form_that_is_gone`);
//! - M2: `answered` checks and acts in two steps, the notification is removed only after the `Answered` event
//!   (`a_second_answer_in_the_observer_is_dropped`, `a_close_in_the_observer_after_the_answer_finds_nothing`,
//!   `many_answers_at_once_run_one_action`);
//! - M3: `redacted` drops the parameters (`a_redacted_text_is_a_sentence_not_a_key`);
//! - M3b: `redacted` masks a number to a text (`a_redacted_plural_text_is_still_a_sentence`).
mod support;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, TextId, Words};
use fs_notification::{
    ActionId, Event, Form, Message, NoteId, Notification, NotificationCode, Situation, Window,
};
use fs_types::coded::Param;
use support::{all_forms, collect, facade, msg, question};

#[test]
fn replacing_closes_the_forms_that_are_gone() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.ask(question("q"), 0).unwrap(); // calm: a modal
    let again = n
        .tell(Notification::tell(msg("done")).replaces(id.clone()), 0)
        .unwrap(); // a toast now
    assert_eq!(again, id);
    assert_eq!(
        out.lines(),
        ["show modal n1", "show toast n1", "close modal n1"]
    );
    assert_eq!(n.open()[0].forms, [Form::Toast]);
    // and it is closed once, later, in the form it has now
    n.close(&id).unwrap();
    assert_eq!(out.lines().last().unwrap(), "close toast n1");
}

#[test]
fn replacing_closes_the_system_form_that_is_gone() {
    let out = all_forms();
    let n = facade(&out);
    n.set_situation(Situation {
        window: Window::Away,
        calm: true,
    });
    let id = n.tell(Notification::tell(msg("a")), 0).unwrap(); // toast + system
    n.set_situation(Situation {
        window: Window::Front,
        calm: true,
    });
    n.tell(Notification::tell(msg("b")).replaces(id), 0)
        .unwrap(); // toast only
    assert!(
        out.lines().contains(&"close system n1".to_owned()),
        "{:?}",
        out.lines()
    );
    assert!(
        !out.lines().contains(&"close toast n1".to_owned()),
        "the form that stays is not closed: {:?}",
        out.lines()
    );
}

fn answer(id: &NoteId) -> (NoteId, ActionId) {
    (id.clone(), ActionId::new("now").unwrap())
}

#[test]
fn a_second_answer_in_the_observer_is_dropped() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.ask(question("q"), 0).unwrap();
    let (id2, now) = answer(&id);
    let again: Arc<Mutex<Vec<Result<(), NotificationCode>>>> = Arc::default();
    let once = Arc::new(AtomicBool::new(false));
    let (facade2, sink) = (n.clone(), Arc::clone(&again));
    let _sub = n.subscribe(Arc::new(move |e: &Event| {
        if matches!(e, Event::Answered { .. }) && !once.swap(true, Ordering::SeqCst) {
            sink.lock()
                .unwrap()
                .push(facade2.answered(&id2, &now).map_err(|c| c.code()));
        }
    }));
    let (_seen, events) = collect(&n);
    let (_, now) = answer(&id);
    n.answered(&id, &now).unwrap();
    let answers = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e, Event::Answered { .. }))
        .count();
    assert_eq!(answers, 1, "one answer, one action");
    assert_eq!(
        again.lock().unwrap().as_slice(),
        [Err(NotificationCode::UnknownId)]
    );
}

#[test]
fn a_close_in_the_observer_after_the_answer_finds_nothing() {
    let out = all_forms();
    let n = facade(&out);
    let id = n.ask(question("q"), 0).unwrap();
    let results: Arc<Mutex<Vec<Result<(), NotificationCode>>>> = Arc::default();
    let (facade2, sink, id2) = (n.clone(), Arc::clone(&results), id.clone());
    let _sub = n.subscribe(Arc::new(move |e: &Event| {
        if matches!(e, Event::Answered { .. }) {
            sink.lock()
                .unwrap()
                .push(facade2.close(&id2).map_err(|c| c.code()));
        }
    }));
    let (_, now) = answer(&id);
    assert!(n.answered(&id, &now).is_ok(), "the answer itself counted");
    assert_eq!(
        results.lock().unwrap().as_slice(),
        [Err(NotificationCode::UnknownId)],
        "after the answer it is gone"
    );
    assert!(n.open().is_empty(), "expected nothing");
}

#[test]
fn many_answers_at_once_run_one_action() {
    for _ in 0..20 {
        let out = all_forms();
        let n = facade(&out);
        let id = n.ask(question("q"), 0).unwrap();
        let hits = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hits);
        let _sub = n.subscribe(Arc::new(move |e: &Event| {
            if matches!(e, Event::Answered { .. }) {
                h.fetch_add(1, Ordering::SeqCst);
            }
        }));
        let oks: usize = std::thread::scope(|s| {
            let workers: Vec<_> = (0..8)
                .map(|_| {
                    s.spawn(|| {
                        let (_, now) = answer(&id);
                        usize::from(n.answered(&id, &now).is_ok())
                    })
                })
                .collect();
            workers.into_iter().map(|w| w.join().unwrap()).sum()
        });
        assert_eq!(oks, 1, "exactly one answer counts");
        assert_eq!(hits.load(Ordering::SeqCst), 1, "exactly one Answered");
        assert!(n.open().is_empty(), "expected nothing");
    }
}

fn say(texts: &Mf2Texts, m: &Message) -> (bool, String) {
    let mut args = Args::new();
    for (name, value) in m.params() {
        args = match value {
            Param::Text(t) => args.text(name, t.clone()),
            Param::Number(n) => args.count(name, *n),
            _ => args,
        };
    }
    let said = texts.say(&TextId::new(m.key()), &args);
    (said.is_found(), said.text().to_owned())
}

#[test]
fn a_redacted_text_is_a_sentence_not_a_key() {
    let sources = [Source {
        language: "en",
        text: "invite = {$name} invites you to {$n} groups",
    }];
    let texts = Mf2Texts::new(&sources, &Locale::english());
    let m = msg("invite")
        .with_text("name", "Maria Example")
        .unwrap()
        .with_number("n", 3)
        .unwrap();
    let (found, whole) = say(&texts, &m);
    assert!(found, "{whole}");
    assert!(whole.contains("Maria Example"), "{whole}");
    let (found, hidden) = say(&texts, &m.redacted());
    assert!(
        found,
        "the redacted text is still a sentence, not `{hidden}`"
    );
    assert_ne!(hidden, "invite");
    assert!(!hidden.contains("Maria"), "{hidden}");
    assert!(hidden.contains("invites you"), "{hidden}");
}

/// Review r1 M3b: a number is not masked to a text; `.input {$count :number}` takes no text (mf2-runtime: `BadOperand`), the sentence
/// would not resolve and the system form would show the raw key. Mutant: `redacted` masks the number too -> this test is red.
#[test]
fn a_redacted_plural_text_is_still_a_sentence() {
    let sources = [Source {
        language: "en",
        text: "invites =\n  .input {$count :number}\n  .match $count\n  one {{{$name} invites you to {$count} group}}\n  * {{{$name} invites you to {$count} groups}}",
    }];
    let texts = Mf2Texts::new(&sources, &Locale::english());
    for count in [1, 3] {
        let m = msg("invites")
            .with_text("name", "Maria Example")
            .unwrap()
            .with_number("count", count)
            .unwrap();
        let (found, whole) = say(&texts, &m);
        assert!(found, "{whole}");
        let (found, hidden) = say(&texts, &m.redacted());
        assert!(
            found,
            "the redacted plural text is a sentence, not `{hidden}`"
        );
        assert_ne!(hidden, "invites");
        assert!(!hidden.contains("Maria"), "{hidden}");
        assert!(hidden.contains("invites you to"), "{hidden}");
        assert!(
            hidden.contains(if count == 1 { "group" } else { "groups" }),
            "{hidden}"
        );
    }
}

//! Tests: asking and remembering (Q2, ENTWURF-einstellungen-als-daten section 9): the box, the write through `fs-settings`, the
//! lock, human-only, `NoRemember` and the lines after.
//!
//! Pattern: fake outlet (`support::Recorder`) + a real `Settings` over `MemoryStore` (the module writes, the test only reads).
//!
//! Mutants. Drawn, each test turned red:
//! - M1: `answered_with` ignores `remember` (`g_q1_the_box_on_writes_the_answer_as_the_setting`, G-Q3, G-Q4);
//! - M2: the write happens without the box (`g_q2_the_box_off_writes_nothing`, G-Q4);
//! - M3: `cancel` (a word outside the list) is written too (`g_q5_a_word_outside_the_list_is_never_remembered`).
//! Intended, not drawn: the box offered although `Unavailable` and the yellow line every time (G-Q4); a bot writes a human-only key
//! (G-Q7); the system form takes a question with the box (G-Q8).
mod support;

use std::sync::Arc;

use fs_components_mf2::{Source, check_pairs};
use fs_notification::{
    ActionId, Event, Form, Notification, NotificationCode, Notifications, Offer, Outlet, Policy,
    Remember, Remembers, SettingsRemember, Situation, Window,
};
use fs_settings::{
    Actor, Admin, Catalog, Declaration, GroupId, Kind, LockRights, MemoryStore, NoSystem, Owner,
    OwnerKind, ProgramName, SettingKey, Settings, SettingsStore, Target, Value,
};
use support::{Recorder, act, all_forms, collect, msg};

struct Yes;
impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

fn key() -> SettingKey {
    SettingKey::new(
        &Owner::new(OwnerKind::Framework, "open").unwrap(),
        "when-open",
    )
    .unwrap()
}

fn catalog(human_only: bool) -> Catalog {
    let mut d = Declaration::new(
        key(),
        Kind::Choice {
            keys: ["ask", "switch", "tab", "window"]
                .map(String::from)
                .to_vec(),
        },
        Value::Choice("ask".into()),
        GroupId::new("open").unwrap(),
    );
    if human_only {
        d = d.human_only();
    }
    Catalog::of(vec![d]).unwrap()
}

fn settings(store: &MemoryStore, human_only: bool) -> Arc<Settings> {
    Arc::new(Settings::open(
        ProgramName::new("demo").unwrap(),
        catalog(human_only),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    ))
}

fn facade(out: &Arc<Recorder>, r: Arc<dyn Remembers>) -> Notifications {
    let o: Arc<dyn Outlet> = out.clone();
    Notifications::with_instance(Policy::default(), vec![o], "n")
        .unwrap()
        .with_remembers(r)
}

fn ask_open() -> Notification {
    Notification::ask(
        msg("open-where"),
        vec![act("tab"), act("window"), act("cancel")],
    )
    .unwrap()
    .remember(Remember::new(key(), Target::AllPrograms))
}

fn titles(n: &Notifications) -> Vec<String> {
    n.open()
        .iter()
        .map(|o| o.notification.title.key().to_owned())
        .collect()
}

fn id(s: &str) -> ActionId {
    ActionId::new(s).unwrap()
}

#[test]
fn g_q1_the_box_on_writes_the_answer_as_the_setting() {
    let store = MemoryStore::new();
    let s = settings(&store, false);
    let out = all_forms();
    let n = facade(&out, Arc::new(SettingsRemember::new(s.clone())));
    let (_sub, got) = collect(&n);
    let q = n.ask(ask_open(), 0).unwrap();
    assert!(matches!(n.open()[0].offer, Some(Offer::Yes { .. })));
    n.answered_with(&q, &id("window"), true).unwrap();
    let e = s.get(&key()).unwrap();
    assert_eq!(e.value, Value::Choice("window".into()));
    assert_eq!(
        e.from,
        fs_settings::Layer::Person,
        "all programs = the person's layer"
    );
    let events = got.lock().unwrap().clone();
    let order: Vec<&str> = events
        .iter()
        .map(|e| match e {
            Event::Answered { .. } => "answered",
            Event::Remembered { .. } => "remembered",
            Event::Closed { .. } => "closed",
            _ => "other",
        })
        .collect();
    assert_eq!(
        order,
        ["other", "answered", "remembered", "other", "closed"]
    ); // Shown, .., the line (Shown), Closed
    // the line afterwards, with the jump to the setting
    let line = n.open().pop().unwrap();
    assert_eq!(
        line.notification.title.key(),
        "notification-notice-remembered"
    );
    assert_eq!(line.notification.actions[0].id.as_str(), "settings");
    let again = n.ask(ask_open(), 0).unwrap();
    assert!(n.open().iter().any(|o| o.id == again));
}

#[test]
fn g_q2_the_box_off_writes_nothing() {
    let store = MemoryStore::new();
    let s = settings(&store, false);
    let n = facade(&all_forms(), Arc::new(SettingsRemember::new(s.clone())));
    let q = n.ask(ask_open(), 0).unwrap();
    let before = store.stamp();
    n.answered_with(&q, &id("window"), false).unwrap();
    assert_eq!(store.stamp(), before, "no value.set");
    assert_eq!(s.get(&key()).unwrap().value, Value::Choice("ask".into()));
    assert_eq!(n.outcome(&q).unwrap().answer, Some(id("window")));
    assert!(titles(&n).is_empty(), "no line either");
}

#[test]
fn g_q3_a_lock_means_no_box_and_a_late_lock_is_a_visible_warning() {
    let store = MemoryStore::new();
    let live = settings(&store, false);
    let boss = Admin {
        store: Arc::new(store.clone()),
        rights: Arc::new(Yes),
        by: fs_types::BasicValue::parse("https://example.org/people/admin").unwrap(),
        clock: fs_settings::system_clock,
        catalog: catalog(false),
    };
    let first = SettingsRemember::new(live.clone());
    let n = facade(&all_forms(), Arc::new(first));
    // locked after the question was asked: the box was there, the write is refused, the answer counts once
    let ask1 = n.ask(ask_open(), 0).unwrap();
    boss.lock(&key(), &Value::Choice("tab".into()), "house rule")
        .unwrap();
    let err = n.answered_with(&ask1, &id("window"), true).unwrap_err();
    assert_eq!(err.code(), NotificationCode::RememberFailed);
    assert_eq!(
        n.outcome(&ask1).unwrap().answer,
        Some(id("window")),
        "the answer counted"
    );
    assert_eq!(titles(&n), ["notification-notice-remember-failed"]);
    // locked before: no box at all
    let s2 = settings(&store, false);
    let r2 = SettingsRemember::new(s2.clone());
    assert!(matches!(r2.offers(&key()), Offer::Locked { .. }));
    let n2 = facade(&all_forms(), Arc::new(r2));
    n2.ask(ask_open(), 0).unwrap();
    assert!(n2.open()[0].offer.is_none());
    assert!(n2.open()[0].notification.remembering().is_none());
    assert_eq!(
        s2.get(&key()).unwrap().value,
        Value::Choice("tab".into()),
        "the lock holds"
    );
}

#[test]
fn g_q4_no_remember_offers_no_box_and_one_yellow_line() {
    let out = all_forms();
    let o: Arc<dyn Outlet> = out.clone();
    let n = Notifications::with_instance(Policy::default(), vec![o], "n").unwrap(); // NoRemember
    let q = n.ask(ask_open(), 0).unwrap();
    assert!(n.open()[0].offer.is_none(), "no box");
    assert!(
        n.open()[0].notification.remembering().is_none(),
        "no where either"
    );
    let q2 = n.ask(ask_open(), 0).unwrap();
    let yellow = titles(&n)
        .iter()
        .filter(|t| *t == "notification-notice-remember-unavailable")
        .count();
    assert_eq!(yellow, 1, "once per facade, not per question");
    // the answer works; asking for the write is an honest error, the answer still counts
    n.answered_with(&q, &id("window"), false).unwrap();
    let e = n.answered_with(&q2, &id("window"), true).unwrap_err();
    assert_eq!(e.code(), NotificationCode::RememberInvalid);
    assert_eq!(n.outcome(&q2).unwrap().answer, Some(id("window")));
}

#[test]
fn g_q5_a_word_outside_the_list_is_never_remembered() {
    let store = MemoryStore::new();
    let s = settings(&store, false);
    let n = facade(&all_forms(), Arc::new(SettingsRemember::new(s.clone())));
    let q = n.ask(ask_open(), 0).unwrap();
    let before = store.stamp();
    n.answered_with(&q, &id("cancel"), true).unwrap();
    assert_eq!(store.stamp(), before, "`cancel` is no word of the list");
    assert_eq!(titles(&n), Vec::<String>::new());
    // no action is a word: refused when asked, not when answered
    let none = Notification::ask(msg("q"), vec![act("cancel"), act("ask")])
        .unwrap()
        .remember(Remember::new(key(), Target::ThisProgram));
    let e = n.ask(none, 0).unwrap_err();
    assert_eq!(e.code(), NotificationCode::RememberInvalid);
    assert_eq!(n.open().len(), 0);
}

#[test]
fn g_q7_a_human_only_key_is_closed_to_a_bot() {
    let store = MemoryStore::new();
    let s = settings(&store, true);
    let bot = SettingsRemember::acting_as(s.clone(), Actor::Bot);
    assert_eq!(bot.offers(&key()), Offer::HumanOnly);
    assert!(bot.remember(&key(), "window", Target::AllPrograms).is_err());
    let n = facade(&all_forms(), Arc::new(bot));
    let q = n.ask(ask_open(), 0).unwrap();
    assert!(n.open()[0].offer.is_none());
    let before = store.stamp();
    let _ = n.answered_with(&q, &id("window"), true);
    assert_eq!(store.stamp(), before, "nothing written for a bot");
    assert!(matches!(
        SettingsRemember::new(s).offers(&key()),
        Offer::Yes { .. }
    ));
}

#[test]
fn g_q8_a_question_with_the_box_is_never_only_the_system_form() {
    let store = MemoryStore::new();
    let s = settings(&store, false);
    let out = all_forms();
    let n = facade(&out, Arc::new(SettingsRemember::new(s)));
    n.set_situation(Situation {
        window: Window::Absent,
        calm: true,
    });
    n.ask(ask_open(), 0).unwrap();
    assert!(!n.open()[0].forms.contains(&Form::System));
    assert_ne!(n.open()[0].forms.len(), 0);
}

#[test]
fn g_q9_texts_exist_in_both_languages() {
    let sources = [
        Source {
            language: "en",
            text: fs_notification::TEXT_EN,
        },
        Source {
            language: "de",
            text: fs_notification::TEXT_DE,
        },
    ];
    assert_eq!(check_pairs(&sources), Vec::<String>::new());
    for k in [
        "notification-notice-remembered",
        "notification-notice-remember-unavailable",
        "notification-notice-remember-failed",
        "notification-action-settings",
    ] {
        assert!(
            fs_notification::TEXT_DE.contains(&format!("{k} =")),
            "{k} de"
        );
        assert!(
            fs_notification::TEXT_EN.contains(&format!("{k} =")),
            "{k} en"
        );
    }
}

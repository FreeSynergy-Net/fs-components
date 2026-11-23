//! Tests: the form rule as a setting (`Policy::choose`, `Policy::from_settings`, the declarations `fs.notification.*`).
//!
//! Intended mutants (none drawn unless marked `drawn`; BERICHT-a1.md lists what was drawn): a critical message is a toast when calm (`critical_and_questions_take_a_modal_when_calm`); the modal rule `never` is
//! ignored (`never_means_never`); the system form is added with the window in front under `away`
//! (`the_system_rule_by_the_window`); the setting value is not read, the default always wins (`settings_change_the_rule`); the
//! default in the declaration and `Policy::default()` drift apart (`declarations_and_defaults_agree`); an `expire-ms` outside
//! its limits is taken as it is (`the_expiry_is_kept_in_its_limits`).
mod support;

use std::sync::Arc;

use fs_notification::{
    Expire, Form, ModalWhen, Notification, NotificationSettings, Policy, Situation, SystemWhen,
    Urgency, Window,
};
use fs_settings::{Declares, MemoryStore, NoSystem, ProgramName, Raw, Settings, Target, Value};
use support::{act, msg};

fn at(window: Window, calm: bool) -> Situation {
    Situation { window, calm }
}

fn tell() -> Notification {
    Notification::tell(msg("a"))
}

fn ask() -> Notification {
    Notification::ask(msg("q"), vec![act("yes")]).unwrap()
}

#[test]
fn critical_and_questions_take_a_modal_when_calm() {
    let p = Policy::default();
    let calm = at(Window::Front, true);
    assert_eq!(p.choose(&tell(), &calm), [Form::Toast]);
    assert_eq!(
        p.choose(&tell().urgency(Urgency::Low), &calm),
        [Form::Toast]
    );
    assert_eq!(
        p.choose(&tell().urgency(Urgency::Critical), &calm),
        [Form::Modal]
    );
    assert_eq!(p.choose(&ask(), &calm), [Form::Modal]);
    let busy = at(Window::Front, false);
    assert_eq!(
        p.choose(&ask(), &busy),
        [Form::Toast],
        "not calm: a standing toast with the same buttons"
    );
    assert_eq!(
        p.choose(&tell().urgency(Urgency::Critical), &busy),
        [Form::Toast]
    );
}

#[test]
fn never_means_never() {
    let p = Policy {
        modal: ModalWhen::Never,
        ..Policy::default()
    };
    assert_eq!(p.choose(&ask(), &at(Window::Front, true)), [Form::Toast]);
}

#[test]
fn the_system_rule_by_the_window() {
    let away = Policy::default();
    assert_eq!(
        away.choose(&tell(), &at(Window::Front, true)),
        [Form::Toast]
    );
    assert_eq!(
        away.choose(&tell(), &at(Window::Away, true)),
        [Form::Toast, Form::System]
    );
    assert_eq!(
        away.choose(&tell(), &at(Window::Absent, true)),
        [Form::System]
    );
    let always = Policy {
        system: SystemWhen::Always,
        ..Policy::default()
    };
    assert_eq!(
        always.choose(&tell(), &at(Window::Front, true)),
        [Form::Toast, Form::System]
    );
    let never = Policy {
        system: SystemWhen::Never,
        ..Policy::default()
    };
    assert_eq!(
        never.choose(&tell(), &at(Window::Away, true)),
        [Form::Toast]
    );
    assert!(
        never.choose(&tell(), &at(Window::Absent, true)).is_empty(),
        "expected nothing"
    );
}

#[test]
fn lifetimes_by_urgency_question_and_explicit_value() {
    let p = Policy::default();
    assert_eq!(p.lifetime_ms(&tell()), Some(6000));
    assert_eq!(p.lifetime_ms(&tell().urgency(Urgency::Critical)), None);
    assert_eq!(p.lifetime_ms(&ask()), None);
    assert_eq!(p.lifetime_ms(&tell().expire(Expire::After(10))), Some(10));
    assert_eq!(p.lifetime_ms(&tell().expire(Expire::Never)), None);
}

fn open() -> (Settings, MemoryStore) {
    let store = MemoryStore::new();
    let s = Settings::open(
        ProgramName::new("demo").unwrap(),
        NotificationSettings::catalog().unwrap(),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    );
    (s, store)
}

#[test]
fn declarations_and_defaults_agree() {
    let decls = NotificationSettings.declarations().unwrap();
    let keys: Vec<&str> = decls.iter().map(|d| d.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "fs.notification.modal",
            "fs.notification.system",
            "fs.notification.expire-ms"
        ]
    );
    let (s, _) = open();
    assert_eq!(Policy::from_settings(&s), Policy::default());
}

#[test]
fn settings_change_the_rule() {
    let (s, _store) = open();
    let key = |n: &str| NotificationSettings::key(n).unwrap();
    s.save(
        &key("modal"),
        Some(&Value::Choice("never".into())),
        Target::ThisProgram,
    )
    .unwrap();
    s.save(
        &key("system"),
        Some(&Value::Choice("always".into())),
        Target::ThisProgram,
    )
    .unwrap();
    s.save(
        &key("expire-ms"),
        Some(&Value::Whole(3000)),
        Target::ThisProgram,
    )
    .unwrap();
    s.refresh();
    assert_eq!(
        Policy::from_settings(&s),
        Policy {
            modal: ModalWhen::Never,
            system: SystemWhen::Always,
            expire_ms: 3000
        }
    );
}

#[test]
fn a_value_out_of_range_is_refused_by_the_setting_not_taken() {
    let (s, _store) = open();
    let key = NotificationSettings::key("expire-ms").unwrap();
    assert!(
        s.save(&key, Some(&Value::Whole(5)), Target::ThisProgram)
            .is_err()
    );
    assert!(
        s.save(
            &key,
            Some(&Value::Choice("calm".into())),
            Target::ThisProgram
        )
        .is_err()
    );
    assert_eq!(Policy::from_settings(&s).expire_ms, 6000);
    let _ = Raw::Bool(true);
}

#[test]
fn the_expiry_is_kept_in_its_limits() {
    use fs_settings::{Effective, Layer};
    let eff = |ms| Effective {
        value: Value::Whole(ms),
        from: Layer::Person,
        lock: None,
    };
    assert_eq!(
        Policy::from_values(None, None, Some(&eff(5))).expire_ms,
        Policy::EXPIRE_MIN_MS
    );
    assert_eq!(
        Policy::from_values(None, None, Some(&eff(10_000_000))).expire_ms,
        Policy::EXPIRE_MAX_MS
    );
    assert_eq!(
        Policy::from_values(None, None, Some(&eff(-4))).expire_ms,
        6000 // a negative value is no value: the default stays
    );
}

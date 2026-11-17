//! K5, K6, K7, K16 of ENTWURF section 8: the shell's declarations, the walk over the layers, locks, the frontend choice.
mod support;

use std::collections::BTreeMap;

use fs_settings::{
    Declares, Effective, Instance, LANGUAGE, Layer, Lock, Raw, STYLE, Snapshot, Sources, Value,
    person_choice, resolve,
};
use fs_types::{BasicValue, Language};
use support::{choice, key, name, shell, text};

fn snap(items: &[(&str, &str, Raw)]) -> Snapshot {
    let mut values = BTreeMap::new();
    for (owner, local, raw) in items {
        values.insert(key((owner, local)), raw.clone());
    }
    Snapshot {
        values,
        problems: Vec::new(),
    }
}

fn lock() -> Lock {
    Lock {
        reason: "why".to_owned(),
        locked_by: "admin".to_owned(),
        locked_at: 7,
    }
}

fn style_decl() -> fs_settings::Declaration {
    shell()
        .declarations()
        .unwrap()
        .into_iter()
        .find(|d| d.key == key(STYLE))
        .unwrap()
}

fn lang_decl() -> fs_settings::Declaration {
    shell()
        .declarations()
        .unwrap()
        .into_iter()
        .find(|d| d.key == key(LANGUAGE))
        .unwrap()
}

/// K5: the shell declares its keys with their defaults (four of the draft plus the refresh beat of b7's O7).
#[test]
fn k5_shell_declarations() {
    let d = shell().declarations().unwrap();
    let keys: Vec<&str> = d.iter().map(|x| x.key.as_str()).collect();
    assert_eq!(
        keys,
        [
            "fs.shell.language",
            "fs.look.style",
            "fs.look.scheme",
            "fs.shell.frontend",
            "fs.shell.refresh-ms"
        ]
    );
    assert_eq!(
        d[0].default,
        Value::Language(Language::parse("en").unwrap())
    );
    assert_eq!(d[1].default, choice("simple"));
    assert_eq!(d[2].default, choice("light"));
    assert_eq!(d[3].default, choice("simple"));
    assert_eq!(d[4].default, Value::Whole(1000));
}

fn walk(
    instance: &Instance,
    system: Option<&Raw>,
    person: &Snapshot,
    program: &Snapshot,
) -> (
    Effective,
    Vec<fs_settings::coded_stub::Coded<fs_settings::SettingCode>>,
) {
    resolve(
        &style_decl(),
        Sources {
            named: None,
            instance,
            system,
            person,
            program,
        },
    )
}

/// K6: the highest layer that holds a fitting value wins; a value of the wrong type is a problem of its layer.
#[test]
fn k6_resolve_layers() {
    let st = |v: &str| Raw::Str(v.to_owned());
    let empty = Snapshot::default();
    let inst = Instance {
        values: snap(&[("look", "style", st("craft"))]),
        ..Instance::default()
    };
    let sys = st("calm");
    let person = snap(&[("look", "style", st("simple"))]);
    let program = snap(&[("look", "style", st("craft"))]);
    let none = Instance::default();

    assert_eq!(walk(&none, None, &empty, &empty).0.from, Layer::Default);
    assert_eq!(walk(&inst, None, &empty, &empty).0.from, Layer::Instance);
    assert_eq!(
        walk(&inst, Some(&sys), &empty, &empty).0.from,
        Layer::System
    );
    let (e, _) = walk(&inst, Some(&sys), &person, &empty);
    assert_eq!((e.from, e.value), (Layer::Person, choice("simple")));
    let (e, p) = walk(&inst, Some(&sys), &person, &program);
    assert_eq!(
        (e.from, e.value, p.len()),
        (Layer::Program, choice("craft"), 0)
    );

    // A program value of the wrong type: the person's value counts, the problem names the program layer.
    let wrong = snap(&[("look", "style", Raw::Int(3))]);
    let (e, p) = walk(&inst, None, &person, &wrong);
    assert_eq!((e.from, e.value), (Layer::Person, choice("simple")));
    assert_eq!(p.len(), 1);
    assert_eq!(name(&p[0]), "wrong-type");
    assert_eq!(text(&p[0], "layer").as_deref(), Some("program"));
    assert_eq!(text(&p[0], "key").as_deref(), Some("fs.look.style"));
    let (e, p) = resolve(
        &lang_decl(),
        Sources {
            named: None,
            instance: &none,
            system: None,
            person: &empty,
            program: &snap(&[("shell", "language", Raw::Int(1))]),
        },
    );
    assert_eq!((e.from, p.len()), (Layer::Default, 1));
}

/// K7: a lock beats the program profile; a lock without a fitting value holds nothing and says so.
#[test]
fn k7_lock_in_resolve() {
    let st = |v: &str| Raw::Str(v.to_owned());
    let empty = Snapshot::default();
    let program = snap(&[("look", "style", st("craft"))]);

    let mut locked = Instance {
        values: snap(&[("look", "style", st("calm"))]),
        ..Instance::default()
    };
    locked.locks.insert(key(STYLE), lock());
    let (e, p) = walk(&locked, None, &empty, &program);
    assert_eq!(
        (e.from, e.value, e.lock),
        (Layer::Instance, choice("calm"), Some(lock()))
    );
    assert!(p.is_empty(), "locked beats the program profile");

    let mut no_value = Instance::default();
    no_value.locks.insert(key(STYLE), lock());
    let (e, p) = walk(&no_value, None, &empty, &program);
    // review N2: the lock fails CLOSED: the block's default holds (not the program profile), the lock stays visible
    assert_eq!((e.from, e.lock), (Layer::Default, Some(lock())));
    assert_eq!(e.value, choice("simple"));
    assert_eq!(p.iter().map(name).collect::<Vec<_>>(), ["lock-refused"]);

    let mut wrong = Instance {
        values: snap(&[("look", "style", Raw::Int(1))]),
        ..Instance::default()
    };
    wrong.locks.insert(key(STYLE), lock());
    let (e, p) = walk(&wrong, None, &empty, &empty);
    assert_eq!((e.from, e.lock), (Layer::Default, Some(lock())));
    assert_eq!(
        p.iter().map(name).collect::<Vec<_>>(),
        ["wrong-type", "lock-refused"]
    );
    // a person's value does not slip through a broken lock
    let person = snap(&[("look", "style", Raw::Str("craft".to_owned()))]);
    let (e, _) = walk(&wrong, None, &person, &person);
    assert_eq!(
        (e.from, e.value, e.lock.is_some()),
        (Layer::Default, choice("simple"), true)
    );
    assert_eq!(text(&p[0], "layer").as_deref(), Some("instance"));
}

/// K16: `person_choice` hands the starter only what the person chose, or a lock.
#[test]
fn k16_person_choice() {
    let e = |from: Layer, lock: Option<Lock>| Effective {
        value: choice("compact"),
        from,
        lock,
    };
    assert_eq!(person_choice(&e(Layer::Default, None)), None);
    assert_eq!(person_choice(&e(Layer::Instance, None)), None);
    assert_eq!(
        person_choice(&e(Layer::Person, None)),
        Some("compact".to_owned())
    );
    assert_eq!(
        person_choice(&e(Layer::Program, None)),
        Some("compact".to_owned())
    );
    assert_eq!(
        person_choice(&e(Layer::Instance, Some(lock()))),
        Some("compact".to_owned())
    );
}

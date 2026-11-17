//! K8 to K15 of ENTWURF section 8 and the new cases of b7's decisions: two programs over one store, the two save buttons,
//! locks, live changes (O7: the beat), who may lock (O17).
mod support;

use std::sync::{Arc, Mutex};

use fs_setting::{
    Admin, Change, Layer, LockRights, MemoryStore, NoLockRights, OnChange, Place, Raw, SCHEME,
    STYLE, Scope, SettingKey, Settings, Store, Target, Value, lock_view,
};
use support::{catalog, choice, key, name, open, program, text};

struct Rights(bool);

impl LockRights for Rights {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        self.0
    }
}

fn admin(store: &MemoryStore, rights: bool) -> Admin {
    Admin {
        store: Arc::new(store.clone()),
        rights: Arc::new(Rights(rights)),
        by: "admin-1".to_owned(),
        catalog: catalog("tasks"),
    }
}

fn from(s: &Settings, k: &SettingKey) -> (Value, Layer) {
    let e = s.get(k).unwrap();
    (e.value, e.from)
}

/// K8: "for this program" writes the own profile only; "for all" writes the general one and drops the own override.
#[test]
fn k8_two_targets() {
    let store = MemoryStore::new();
    let (a, b) = (open("tasks", &store), open("calendars", &store));
    let st = key(STYLE);
    a.save(&st, Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    b.refresh();
    assert_eq!(from(&a, &st), (choice("craft"), Layer::Program));
    assert_eq!(from(&b, &st), (choice("simple"), Layer::Default));

    b.save(&st, Some(&choice("calm")), Target::ThisProgram)
        .unwrap();
    a.save(&st, Some(&choice("simple")), Target::AllPrograms)
        .unwrap();
    b.refresh();
    assert_eq!(
        from(&a, &st),
        (choice("simple"), Layer::Person),
        "own override removed, follows the person"
    );
    assert_eq!(
        from(&b, &st),
        (choice("calm"), Layer::Program),
        "the other program's own value stays"
    );

    a.save(&st, Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    a.save(&st, None, Target::ThisProgram).unwrap();
    assert_eq!(
        from(&a, &st),
        (choice("simple"), Layer::Person),
        "reset: back to the person"
    );
}

/// K9: not declared, wrong kind; nothing is written (the stamp stays).
#[test]
fn k9_save_refused() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let before = store.stamp();
    let foreign = SettingKey::parse("program.calendars.week-start").unwrap();
    let e = a
        .save(&foreign, Some(&Value::Whole(2)), Target::ThisProgram)
        .unwrap_err();
    assert_eq!(
        (name(&e), text(&e, "key").as_deref()),
        ("not-declared", Some("program.calendars.week-start"))
    );
    let e = a
        .save(&key(STYLE), Some(&Value::Whole(2)), Target::ThisProgram)
        .unwrap_err();
    assert_eq!(name(&e), "wrong-type");
    let e = a
        .save(&key(STYLE), Some(&choice("nope")), Target::AllPrograms)
        .unwrap_err();
    assert_eq!(name(&e), "unknown-choice");
    assert_eq!(store.stamp(), before);
}

/// K10: saving a locked key is refused for both targets, with key, reason and who.
#[test]
fn k10_save_locked() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    admin(&store, true)
        .lock(&key(STYLE), &choice("calm"), "house style", 5)
        .unwrap();
    a.refresh();
    for target in [Target::ThisProgram, Target::AllPrograms] {
        let e = a
            .save(&key(STYLE), Some(&choice("craft")), target)
            .unwrap_err();
        assert_eq!(name(&e), "locked");
        assert_eq!(text(&e, "key").as_deref(), Some("fs.look.style"));
        assert_eq!(text(&e, "reason").as_deref(), Some("house style"));
        assert_eq!(text(&e, "by").as_deref(), Some("admin-1"));
    }
}

/// K11: the admin's handle keeps the rules: no right, locked also for the locker, a second lock, unlock.
#[test]
fn k11_admin_rules() {
    let store = MemoryStore::new();
    let st = key(STYLE);
    let boss = admin(&store, true);
    let stranger = admin(&store, false);
    assert_eq!(
        name(&stranger.lock(&st, &choice("calm"), "x", 1).unwrap_err()),
        "no-right"
    );
    boss.lock(&st, &choice("calm"), "x", 1).unwrap();
    assert_eq!(
        name(&boss.set_default(&st, Some(&choice("craft"))).unwrap_err()),
        "locked"
    );
    assert_eq!(
        name(&boss.lock(&st, &choice("craft"), "y", 2).unwrap_err()),
        "locked"
    );
    assert_eq!(name(&stranger.unlock(&st).unwrap_err()), "no-right");
    boss.unlock(&st).unwrap();
    let inst = store.instance();
    assert_eq!(inst.locks.len(), 0);
    assert_eq!(
        inst.values.values.get(&st),
        Some(&Raw::Str("calm".to_owned())),
        "the instance value stays as the default"
    );
    boss.set_default(&st, Some(&choice("craft"))).unwrap();
    boss.set_default(&key(SCHEME), None).unwrap();
}

/// K12: the person's own value stays stored under a lock and counts again after the unlock.
#[test]
fn k12_person_value_survives_lock() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let st = key(STYLE);
    a.save(&st, Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    let boss = admin(&store, true);
    boss.lock(&st, &choice("calm"), "r", 1).unwrap();
    a.refresh();
    let e = a.get(&st).unwrap();
    assert_eq!(
        (e.value, e.from, e.lock.is_some()),
        (choice("calm"), Layer::Instance, true)
    );
    boss.unlock(&st).unwrap();
    a.refresh();
    assert_eq!(from(&a, &st), (choice("craft"), Layer::Program));
}

/// K13: the button depends on the right, the locked state is shown to everyone.
#[test]
fn k13_lock_view() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let st = key(STYLE);
    let e = a.get(&st).unwrap();
    let (none, some) = (NoLockRights, Rights(true));
    assert!(!lock_view(&none, &st, &e).button);
    assert!(lock_view(&some, &st, &e).button);
    admin(&store, true)
        .lock(&st, &choice("calm"), "r", 3)
        .unwrap();
    a.refresh();
    let e = a.get(&st).unwrap();
    for (rights, button) in [
        (&none as &dyn LockRights, false),
        (&some as &dyn LockRights, true),
    ] {
        let v = lock_view(rights, &st, &e);
        assert_eq!((v.button, v.locked.is_some()), (button, true));
    }
}

struct Told(Arc<Settings>, Mutex<Vec<(String, Value)>>);

impl OnChange for Told {
    fn changed(&self, c: &Change) {
        // The observer asks the handle: it already holds the new state.
        let now = self.0.get(&c.key).unwrap().value;
        self.1
            .lock()
            .unwrap()
            .push((c.key.as_str().to_owned(), now));
    }
}

/// K14: a change by another program is told once, after the new state is in place; nothing new, nothing told.
#[test]
fn k14_live_observer() {
    let store = MemoryStore::new();
    let a = Arc::new(open("tasks", &store));
    let b = open("calendars", &store);
    let told = Arc::new(Told(a.clone(), Mutex::new(Vec::new())));
    a.observe(told.clone());
    b.save(&key(STYLE), Some(&choice("craft")), Target::AllPrograms)
        .unwrap();
    assert_eq!(a.refresh().len(), 1);
    assert_eq!(
        *told.1.lock().unwrap(),
        [("fs.look.style".to_owned(), choice("craft"))]
    );
    assert_eq!(a.refresh().len(), 0);
    assert_eq!(told.1.lock().unwrap().len(), 1);
}

/// K15: a key the program does not declare in its own profile is a hint; in the general profile it is none.
#[test]
fn k15_unknown_key() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let foreign = SettingKey::parse("program.calendars.week-start").unwrap();
    store
        .write(
            &Scope::General,
            Place::Profile,
            0,
            &foreign,
            Some(&Raw::Int(3)),
        )
        .unwrap();
    a.refresh();
    assert_eq!(a.problems().len(), 0);
    store
        .write(
            &Scope::Program(program("tasks")),
            Place::Profile,
            0,
            &foreign,
            Some(&Raw::Int(3)),
        )
        .unwrap();
    a.refresh();
    let p = a.problems();
    assert_eq!(p.len(), 1);
    assert_eq!(
        (name(&p[0]), text(&p[0], "layer").as_deref()),
        ("unknown-key", Some("program"))
    );
}

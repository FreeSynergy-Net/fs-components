//! K37 to K41 and the second part of K44: named profiles over `resolve` and `Settings` (ENTWURF section 16).
mod support;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use fs_settings::{
    Change, Instance, Layer, Lock, MemoryStore, OnChange, Place, ProfileName, ProfileRef, Raw,
    SCHEME, STYLE, Scope, Snapshot, Sources, Target, resolve,
};
use support::{choice, key, name, open, program, shell, text};

fn snap(v: &str) -> Snapshot {
    let mut values = BTreeMap::new();
    values.insert(key(STYLE), Raw::Str(v.to_owned()));
    Snapshot {
        values,
        problems: Vec::new(),
    }
}

fn style_of(
    inst: &Instance,
    program: &Snapshot,
    person: &Snapshot,
    named: Option<(&Snapshot, &Snapshot)>,
) -> (Layer, bool) {
    use fs_settings::Declares;
    let d = shell()
        .declarations()
        .unwrap()
        .into_iter()
        .find(|d| d.key == key(STYLE))
        .unwrap();
    let (e, _) = resolve(
        &d,
        Sources {
            instance: inst,
            system: None,
            person,
            program,
            named,
        },
    );
    (e.from, e.lock.is_some())
}

/// K37: the active named profile's program part beats its general part, which beats the base's program part.
#[test]
fn k37_resolve_named_layers() {
    let (inst, none) = (Instance::default(), Snapshot::default());
    let (prog, person) = (snap("simple"), snap("calm"));
    let (np, npr) = (snap("craft"), snap("craft"));
    assert_eq!(
        style_of(&inst, &prog, &person, Some((&np, &npr))).0,
        Layer::NamedProgram
    );
    assert_eq!(
        style_of(&inst, &prog, &person, Some((&np, &none))).0,
        Layer::NamedPerson
    );
    assert_eq!(
        style_of(&inst, &prog, &person, Some((&none, &none))).0,
        Layer::Program
    );
    assert_eq!(
        style_of(&inst, &none, &person, Some((&none, &none))).0,
        Layer::Person
    );
    assert_eq!(style_of(&inst, &prog, &person, None).0, Layer::Program);
}

/// K38: a lock beats the active named profile's own value.
#[test]
fn k38_lock_beats_named() {
    let mut inst = Instance {
        values: snap("calm"),
        ..Instance::default()
    };
    inst.locks.insert(
        key(STYLE),
        Lock {
            reason: "r".to_owned(),
            locked_by: "a".to_owned(),
            locked_at: fs_types::Utc::from_millis(1),
        },
    );
    let (np, npr) = (snap("craft"), snap("craft"));
    assert_eq!(
        style_of(&inst, &snap("simple"), &snap("simple"), Some((&np, &npr))),
        (Layer::Instance, true)
    );
}

struct Told(Mutex<Vec<Change>>);

impl OnChange for Told {
    fn changed(&self, c: &Change) {
        self.0.lock().unwrap().push(c.clone());
    }
}

fn id_of(s: &fs_settings::Settings, n: &str) -> u64 {
    *s.profiles()
        .get(&ProfileRef::Named(ProfileName::new(n).unwrap()))
        .unwrap()
}

/// K39: switching the profile is live: each changed key is told exactly once.
#[test]
fn k39_switch_live() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    a.create_profile("Vortrag").unwrap();
    a.switch_profile(Some("Vortrag")).unwrap();
    a.save(&key(STYLE), Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    a.save(&key(SCHEME), Some(&choice("dark")), Target::AllPrograms)
        .unwrap();
    a.switch_profile(None).unwrap();
    assert_eq!(a.get(&key(STYLE)).unwrap().from, Layer::Default);
    let told = Arc::new(Told(Mutex::new(Vec::new())));
    a.observe(told.clone());
    let changes = a.switch_profile(Some("vortrag")).unwrap();
    assert_eq!(changes.len(), 2);
    let mut keys: Vec<String> = told
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|c| c.key.as_str().to_owned())
        .collect();
    keys.sort();
    assert_eq!(keys, ["fs.look.scheme", "fs.look.style"]);
    assert_eq!(a.get(&key(STYLE)).unwrap().from, Layer::NamedProgram);
    assert_eq!(a.get(&key(SCHEME)).unwrap().from, Layer::NamedPerson);
}

/// K40: with a named profile active, saving writes its part and leaves the base untouched; back on the base it is gone.
#[test]
fn k40_save_in_active_profile() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    a.save(&key(STYLE), Some(&choice("calm")), Target::AllPrograms)
        .unwrap();
    a.create_profile("Arbeit").unwrap();
    a.switch_profile(Some("Arbeit")).unwrap();
    a.save(&key(STYLE), Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    let id = id_of(&a, "Arbeit");
    let own = Scope::Program(program("tasks"));
    let part = fs_settings::SettingsStore::profile(&store, &own, Place::Profile, id);
    assert_eq!(
        part.values.get(&key(STYLE)),
        Some(&Raw::Str("craft".to_owned()))
    );
    let base = fs_settings::SettingsStore::profile(&store, &Scope::General, Place::Profile, 0);
    assert_eq!(
        base.values.get(&key(STYLE)),
        Some(&Raw::Str("calm".to_owned()))
    );
    assert_eq!(
        fs_settings::SettingsStore::profile(&store, &own, Place::Profile, 0)
            .values
            .len(),
        0
    );
    a.switch_profile(None).unwrap();
    let e = a.get(&key(STYLE)).unwrap();
    assert_eq!((e.value, e.from), (choice("calm"), Layer::Person));
}

/// K41: deleting the active profile tells the changes, the base is active; a new profile of the same name is empty.
#[test]
fn k41_delete_active_and_recreate() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    a.create_profile("Arbeit").unwrap();
    a.switch_profile(Some("Arbeit")).unwrap();
    a.save(&key(STYLE), Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    let old = id_of(&a, "Arbeit");
    let changes = a.delete_profile("Arbeit").unwrap();
    assert!(
        changes.iter().any(|c| c.key == key(STYLE)),
        "the change is told"
    );
    assert_eq!(a.profiles().active(), &ProfileRef::Base);
    assert_eq!(a.get(&key(STYLE)).unwrap().from, Layer::Default);
    a.create_profile("Arbeit").unwrap();
    assert_ne!(id_of(&a, "Arbeit"), old, "an id is never reused");
    a.switch_profile(Some("Arbeit")).unwrap();
    assert_eq!(
        a.get(&key(STYLE)).unwrap().from,
        Layer::Default,
        "the new profile is empty"
    );
}

/// K44 (second part): a refused profile operation is a `profile` code with `cause` and `name`.
#[test]
fn k44_settings_error_mapping() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let e = a.create_profile("").unwrap_err();
    assert_eq!(name(&e), "profile");
    assert_eq!(text(&e, "cause").as_deref(), Some("name-invalid"));
    assert_eq!(text(&e, "name").as_deref(), Some(""));
    a.create_profile("Arbeit").unwrap();
    let e = a.create_profile("ARBEIT").unwrap_err();
    assert_eq!(
        (text(&e, "cause").as_deref(), text(&e, "name").as_deref()),
        (Some("exists"), Some("ARBEIT"))
    );
    let e = a.switch_profile(Some("Fehlt")).unwrap_err();
    assert_eq!(text(&e, "cause").as_deref(), Some("unknown"));
    a.rename_profile("Arbeit", "arbeit").unwrap();
    assert_eq!(
        text(&a.delete_profile("Fehlt").unwrap_err(), "cause").as_deref(),
        Some("unknown")
    );
}

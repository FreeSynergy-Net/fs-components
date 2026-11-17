//! K22 to K25 of ENTWURF section 8 (legacy language, instance locks, stamp, live over files) and the new case of b7's O16 (two writers lose no change).
mod support;

use std::sync::Arc;

use fs_setting::{Owner, OwnerKind, Place, Raw, Scope, SettingKey, Settings, Store, Target};
use support::{catalog, choice, files, general, name, program, put, str_raw, style, tmp};

/// K22: the old language file counts only while `settings.toml` has no language; it is never written.
#[test]
fn k22_legacy_language() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    let lang = SettingKey::parse("fs.shell.language").unwrap();
    let old = t.path().join("cfg/freesynergy/language");
    put(&old, "de\n");
    let s = f.profile(&Scope::General, Place::Profile);
    assert_eq!(s.values.get(&lang), Some(&str_raw("de")));
    put(&general(t.path()), "\"fs.shell.language\" = \"en\"\n");
    assert_eq!(
        f.profile(&Scope::General, Place::Profile).values.get(&lang),
        Some(&str_raw("en"))
    );
    f.write(
        &Scope::General,
        Place::Profile,
        &style(),
        Some(&str_raw("calm")),
    )
    .unwrap();
    assert_eq!(std::fs::read_to_string(&old).unwrap(), "de\n");
}

/// K23: locks of the instance file: complete, or a field missing.
#[test]
fn k23_instance_locks() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    let inst = t.path().join("etc/freesynergy/instance.toml");
    put(
        &inst,
        "[values]\n\"fs.look.style\" = \"calm\"\n[locks]\n\"fs.look.style\" = { reason = \"r\", by = \"a\", at = 9 }\n",
    );
    let i = f.instance();
    let l = i.locks.get(&style()).unwrap();
    assert_eq!(
        (l.reason.as_str(), l.locked_by.as_str(), l.locked_at),
        ("r", "a", 9)
    );
    put(
        &inst,
        "[locks]\n\"fs.look.style\" = { reason = \"r\", by = \"a\" }\n",
    );
    let i = f.instance();
    assert_eq!(i.locks.len(), 0);
    assert_eq!(
        i.values.problems.iter().map(name).collect::<Vec<_>>(),
        ["wrong-type"]
    );
}

/// K24: the stamp changes with a write and with the instance file, and stays without a change.
#[test]
fn k24_stamp() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    let s0 = f.stamp();
    assert_eq!(f.stamp(), s0);
    f.write(
        &Scope::General,
        Place::Profile,
        &style(),
        Some(&str_raw("craft")),
    )
    .unwrap();
    let s1 = f.stamp();
    assert_ne!(s0, s1);
    put(
        &t.path().join("etc/freesynergy/instance.toml"),
        "[values]\n\"fs.look.style\" = \"calm\"\n",
    );
    let s2 = f.stamp();
    assert_ne!(s1, s2);
    assert_eq!(f.stamp(), s2);
}

/// K25: two programs over the files: B saves for all, A sees it on its next `refresh`.
#[test]
fn k25_live_over_files() {
    let t = tmp();
    let open = |n: &str| {
        Settings::open(
            program(n),
            catalog(n),
            Arc::new(files(t.path(), n)),
            Arc::new(fs_setting::NoSystem),
        )
    };
    let (a, b) = (open("tasks"), open("calendars"));
    b.save(&style(), Some(&choice("craft")), Target::AllPrograms)
        .unwrap();
    let changes = a.refresh();
    assert_eq!(changes.len(), 1);
    assert_eq!(a.get(&style()).unwrap().value, choice("craft"));
}

/// O16 (b7): two writers at the same moment lose no change (flock around read-change-replace).
#[test]
fn k34_two_writers_lose_nothing() {
    let t = tmp();
    let n = 40;
    let owner = Owner::new(OwnerKind::Program, "tasks").unwrap();
    let k = |p: &str, i: usize| SettingKey::new(&owner, &format!("{p}{i}")).unwrap();
    std::thread::scope(|s| {
        for (prog, p) in [("tasks", "a"), ("calendars", "b")] {
            let f = files(t.path(), prog);
            let k = &k;
            s.spawn(move || {
                for i in 0..n {
                    f.write(
                        &Scope::General,
                        Place::Profile,
                        &k(p, i),
                        Some(&Raw::Int(i64::try_from(i).unwrap())),
                    )
                    .unwrap();
                }
            });
        }
    });
    let s = files(t.path(), "tasks").profile(&Scope::General, Place::Profile);
    assert!(s.problems.is_empty(), "{:?}", s.problems);
    assert_eq!(s.values.len(), 2 * n, "a change was lost");
    let left: Vec<_> = std::fs::read_dir(t.path().join("cfg/freesynergy"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert!(
        left.iter().all(|f| !f.to_string_lossy().ends_with(".tmp")),
        "{left:?}"
    );
}

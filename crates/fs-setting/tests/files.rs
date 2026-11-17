//! K17 to K25 of ENTWURF section 8 (the XDG files) and the new case of b7's O16 (two writers lose no change).
mod support;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fs_setting::{
    Owner, OwnerKind, Place, Raw, STYLE, Scope, SettingKey, Settings, Store, Target, XdgFiles,
};
use support::{catalog, choice, key, name, number, program, text};

fn os(p: &Path) -> Option<OsString> {
    Some(p.as_os_str().to_owned())
}

/// The files of `prog` in a temp folder (config, state, home all below it); no real variable is read.
fn files(root: &Path, prog: &str) -> XdgFiles {
    XdgFiles::new(
        program(prog),
        os(&root.join("cfg")),
        os(&root.join("state")),
        os(root),
        os(&root.join("etc")),
    )
}

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn general(root: &Path) -> PathBuf {
    root.join("cfg/freesynergy/settings.toml")
}

fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn style() -> SettingKey {
    key(STYLE)
}

fn str_raw(v: &str) -> Raw {
    Raw::Str(v.to_owned())
}

/// K17: the variables decide the places; relative ones are ignored; no folder is a code.
#[test]
fn k17_places() {
    let t = tmp();
    let root = t.path();
    let rel = XdgFiles::new(
        program("tasks"),
        Some("rel/cfg".into()),
        None,
        os(root),
        None,
    );
    assert_eq!(
        rel.file(&Scope::General, Place::Profile).unwrap(),
        root.join(".config/freesynergy/settings.toml")
    );
    assert_eq!(
        rel.file(&Scope::General, Place::Device).unwrap(),
        root.join(".local/state/freesynergy/settings.toml")
    );
    let none = XdgFiles::new(program("tasks"), None, None, None, None);
    let e = none.file(&Scope::General, Place::Profile).unwrap_err();
    assert_eq!(
        (name(&e), text(&e, "which").as_deref()),
        ("no-folder", Some("config"))
    );
    let e = none.file(&Scope::General, Place::Device).unwrap_err();
    assert_eq!(text(&e, "which").as_deref(), Some("state"));
    let f = files(root, "tasks");
    assert_eq!(
        f.file(&Scope::General, Place::Device).unwrap(),
        root.join("state/freesynergy/settings.toml")
    );
    assert_eq!(
        f.file(&Scope::Program(program("tasks")), Place::Profile)
            .unwrap(),
        root.join("cfg/freesynergy/tasks/settings.toml")
    );
    // The instance file: the first absolute entry of XDG_CONFIG_DIRS.
    let dirs = format!("rel:{}", root.join("abs").display());
    let inst = XdgFiles::new(program("tasks"), None, None, os(root), Some(dirs.into()));
    put(
        &root.join("abs/freesynergy/instance.toml"),
        "[values]\n\"fs.look.style\" = \"calm\"\n",
    );
    assert_eq!(
        inst.instance().values.values.get(&style()),
        Some(&str_raw("calm"))
    );
}

/// K18: write and read back; other keys stay; modes 0600/0700; another program's profile is refused.
#[test]
fn k18_write_read() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    let other = SettingKey::new(
        &Owner::new(OwnerKind::Program, "tasks").unwrap(),
        "week-start",
    )
    .unwrap();
    f.write(&Scope::General, Place::Profile, &other, Some(&Raw::Int(3)))
        .unwrap();
    f.write(
        &Scope::General,
        Place::Profile,
        &style(),
        Some(&str_raw("craft")),
    )
    .unwrap();
    let s = f.profile(&Scope::General, Place::Profile);
    assert!(s.problems.is_empty());
    assert_eq!(s.values.get(&style()), Some(&str_raw("craft")));
    assert_eq!(
        s.values.get(&other),
        Some(&Raw::Int(3)),
        "the other key stays"
    );
    f.write(&Scope::General, Place::Profile, &style(), None)
        .unwrap();
    assert_eq!(f.profile(&Scope::General, Place::Profile).values.len(), 1);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&general(t.path())), 0o600);
        assert_eq!(mode(&t.path().join("cfg/freesynergy")), 0o700);
    }
    let e = f
        .write(
            &Scope::Program(program("calendars")),
            Place::Profile,
            &style(),
            Some(&str_raw("calm")),
        )
        .unwrap_err();
    assert_eq!(
        (name(&e), text(&e, "kind").as_deref()),
        ("write-failed", Some("not-own"))
    );
    assert!(!t.path().join("cfg/freesynergy/calendars").exists());
}

/// K19: a broken file is reported with file and line and never overwritten.
#[test]
fn k19_broken_file() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    let path = general(t.path());
    put(&path, "a = 1\nthis is = = broken\n");
    let before = std::fs::read(&path).unwrap();
    let s = f.profile(&Scope::General, Place::Profile);
    assert_eq!(s.problems.len(), 1);
    assert_eq!(name(&s.problems[0]), "file-broken");
    assert_eq!(
        text(&s.problems[0], "file"),
        Some(path.display().to_string())
    );
    assert_eq!(number(&s.problems[0], "line"), Some(2));
    let e = f
        .write(
            &Scope::General,
            Place::Profile,
            &style(),
            Some(&str_raw("calm")),
        )
        .unwrap_err();
    assert_eq!(name(&e), "file-broken");
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

/// K20: tables and lists are no values; an upper-case key is no key.
#[test]
fn k20_wrong_values() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    put(
        &general(t.path()),
        "\"fs.look.style\" = { a = 1 }\n\"fs.look.scheme\" = [1, 2]\nFoo = 1\n",
    );
    let s = f.profile(&Scope::General, Place::Profile);
    let mut names: Vec<_> = s.problems.iter().map(name).collect();
    names.sort_unstable();
    assert_eq!(names, ["key-invalid", "wrong-type", "wrong-type"]);
    let wt = s.problems.iter().find(|p| name(p) == "wrong-type").unwrap();
    assert!(text(wt, "key").is_some() && text(wt, "file").is_some());
    assert!(s.values.is_empty());
}

/// K21: more than 64 KiB, and a pipe (Unix; must not block).
#[test]
fn k21_size_and_kind() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    put(
        &general(t.path()),
        &format!("# {}\n", "x".repeat(64 * 1024)),
    );
    let s = f.profile(&Scope::General, Place::Profile);
    assert_eq!(
        s.problems.iter().map(name).collect::<Vec<_>>(),
        ["file-too-large"]
    );
    #[cfg(unix)]
    {
        let fifo = t.path().join("cfg/freesynergy/tasks/settings.toml");
        std::fs::create_dir_all(fifo.parent().unwrap()).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let s = f.profile(&Scope::Program(program("tasks")), Place::Profile);
        assert_eq!(
            s.problems.iter().map(name).collect::<Vec<_>>(),
            ["not-regular"]
        );
    }
}

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
    assert!(i.locks.is_empty());
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

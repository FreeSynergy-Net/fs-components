//! K17 to K21 of ENTWURF section 8 (the XDG files: places, write, broken, wrong, size).
#![cfg(feature = "xdg-files")]
mod support;

use fs_settings::{Owner, OwnerKind, Place, Raw, Scope, SettingKey, SettingsStore, XdgFiles};
use support::{files, general, name, number, os, program, put, str_raw, style, text, tmp};

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
    // (the list separator is `:` on Unix and `;` on Windows: build it with `join_paths`, never by hand)
    let dirs = std::env::join_paths([std::path::PathBuf::from("rel"), root.join("abs")]).unwrap();
    let inst = XdgFiles::new(program("tasks"), None, None, os(root), Some(dirs));
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
    f.write(
        &Scope::General,
        Place::Profile,
        0,
        &other,
        Some(&Raw::Int(3)),
    )
    .unwrap();
    f.write(
        &Scope::General,
        Place::Profile,
        0,
        &style(),
        Some(&str_raw("craft")),
    )
    .unwrap();
    let s = f.profile(&Scope::General, Place::Profile, 0);
    assert_eq!(s.problems.len(), 0);
    assert_eq!(s.values.get(&style()), Some(&str_raw("craft")));
    assert_eq!(
        s.values.get(&other),
        Some(&Raw::Int(3)),
        "the other key stays"
    );
    f.write(&Scope::General, Place::Profile, 0, &style(), None)
        .unwrap();
    assert_eq!(
        f.profile(&Scope::General, Place::Profile, 0).values.len(),
        1
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        use std::path::Path;
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&general(t.path())), 0o600);
        assert_eq!(mode(&t.path().join("cfg/freesynergy")), 0o700);
    }
    let e = f
        .write(
            &Scope::Program(program("calendars")),
            Place::Profile,
            0,
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
    let s = f.profile(&Scope::General, Place::Profile, 0);
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
            0,
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
    let s = f.profile(&Scope::General, Place::Profile, 0);
    let mut names: Vec<_> = s.problems.iter().map(name).collect();
    names.sort_unstable();
    assert_eq!(names, ["key-invalid", "wrong-type", "wrong-type"]);
    let wt = s.problems.iter().find(|p| name(p) == "wrong-type").unwrap();
    assert!(text(wt, "key").is_some() && text(wt, "file").is_some());
    assert_eq!(s.values.len(), 0);
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
    let s = f.profile(&Scope::General, Place::Profile, 0);
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
        let s = f.profile(&Scope::Program(program("tasks")), Place::Profile, 0);
        assert_eq!(
            s.problems.iter().map(name).collect::<Vec<_>>(),
            ["not-regular"]
        );
    }
}

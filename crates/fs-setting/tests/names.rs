//! K1, K2 and the owner kinds (b7 decision O13) of ENTWURF section 8.
mod support;

use fs_setting::{Owner, OwnerKind, ProgramName, SettingKey};
use support::{name, text};

fn owner(kind: OwnerKind, n: &str) -> Owner {
    Owner::new(kind, n).unwrap()
}

/// K1: valid and invalid keys, parse round trip.
#[test]
fn k1_setting_key() {
    let fw = |n: &str| owner(OwnerKind::Framework, n);
    assert_eq!(SettingKey::new(&fw("shell"), "language").unwrap().as_str(), "fs.shell.language");
    assert_eq!(SettingKey::new(&owner(OwnerKind::Program, "tasks"), "week-start").unwrap().as_str(), "program.tasks.week-start");
    assert!(SettingKey::new(&fw("look"), "a.b.c.d").is_ok());
    for bad in ["A", "a--b", "-a", "a.b.c.d.e", "", "a b", "a_b"] {
        let e = SettingKey::new(&fw("shell"), bad).unwrap_err();
        assert_eq!(name(&e), "key-invalid", "{bad}");
    }
    let long = "a".repeat(32);
    let e = SettingKey::new(&fw("shell"), &format!("{long}.{long}.{long}")).unwrap_err();
    assert_eq!(name(&e), "key-invalid");
    assert!(text(&e, "key").is_some_and(|k| k.len() > 96));
    assert_eq!(name(&Owner::new(OwnerKind::Framework, "Shell").unwrap_err()), "key-invalid");
    let k = SettingKey::new(&fw("shell"), "language").unwrap();
    assert_eq!(SettingKey::parse(k.as_str()).unwrap(), k);
    assert_eq!(name(&SettingKey::parse("foo.x.y").unwrap_err()), "key-invalid");
    assert_eq!(name(&SettingKey::parse("fs.shell").unwrap_err()), "key-invalid");
}

/// K2: program names are folder names.
#[test]
fn k2_program_name() {
    assert_eq!(ProgramName::new("tasks").unwrap().as_str(), "tasks");
    for bad in ["../x", "Tasks", "a/b", "", "a.b"] {
        let e = ProgramName::new(bad).unwrap_err();
        assert_eq!(name(&e), "program-invalid", "{bad}");
        assert_eq!(text(&e, "program").as_deref(), Some(bad));
    }
}

/// O13 (b7): the owner kinds include motion and sound theme; every kind has its prefix and parses back.
#[test]
fn k31_owner_kinds_motion_and_sound() {
    assert_eq!(OwnerKind::Motion.prefix(), "motion");
    assert_eq!(OwnerKind::SoundTheme.prefix(), "sound");
    assert_eq!(OwnerKind::ALL.len(), 7);
    for kind in OwnerKind::ALL {
        let k = SettingKey::new(&owner(kind, "x"), "y").unwrap();
        assert_eq!(k.as_str(), format!("{}.x.y", kind.prefix()));
        assert_eq!(SettingKey::parse(k.as_str()).unwrap(), k);
    }
    let prefixes: std::collections::HashSet<_> = OwnerKind::ALL.iter().map(|k| k.prefix()).collect();
    assert_eq!(prefixes.len(), 7);
}

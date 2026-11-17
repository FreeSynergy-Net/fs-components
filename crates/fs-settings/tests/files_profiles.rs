//! K42 and K43 of ENTWURF section 16: named profiles in the XDG files.
#![cfg(feature = "xdg-files")]
mod support;

use std::sync::Arc;

use fs_settings::{Layer, NoSystem, Settings, Target};
use support::{catalog, choice, files, name, program, put, style, text, tmp};

fn open(root: &std::path::Path, n: &str) -> Settings {
    Settings::open(
        program(n),
        catalog(n),
        Arc::new(files(root, n)),
        Arc::new(NoSystem),
    )
}

/// K42: `[profiles."<id>"]` round trip; the index files (names in config, the active one in state); a part without an
/// index entry is not read.
#[test]
fn k42_profiles_in_files() {
    let t = tmp();
    let root = t.path();
    let a = open(root, "tasks");
    a.create_profile("Arbeit").unwrap();
    a.switch_profile(Some("Arbeit")).unwrap();
    a.save(&style(), Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    a.save(&style(), Some(&choice("calm")), Target::AllPrograms)
        .unwrap();
    let general = std::fs::read_to_string(root.join("cfg/freesynergy/settings.toml")).unwrap();
    assert!(
        general.contains("[profiles.1]") || general.contains("[profiles.\"1\"]"),
        "{general}"
    );
    let names = std::fs::read_to_string(root.join("cfg/freesynergy/profiles.toml")).unwrap();
    assert!(names.contains("Arbeit"), "{names}");
    let active = std::fs::read_to_string(root.join("state/freesynergy/profiles.toml")).unwrap();
    assert!(active.contains("active = 1"), "{active}");
    // a second program on the same files: the same profile is active, the same value arrives
    let b = open(root, "calendars");
    assert_eq!(b.profiles().names().count(), 1);
    assert_eq!(b.get(&style()).unwrap().from, Layer::NamedPerson);
    // a part of an id that no index entry names is not read
    put(
        &root.join("cfg/freesynergy/tasks/settings.toml"),
        "[profiles.\"9\"]\n\"fs.look.style\" = \"craft\"\n",
    );
    a.refresh();
    assert_eq!(a.get(&style()).unwrap().from, Layer::NamedPerson);
    assert_ne!(a.profiles().active(), &fs_settings::ProfileRef::Base);
}

/// K43: an active id without a name falls back to the base and says so; a bad name is listed.
#[test]
fn k43_index_problems() {
    let t = tmp();
    let root = t.path();
    put(
        &root.join("state/freesynergy/profiles.toml"),
        "active = 7\n",
    );
    put(
        &root.join("cfg/freesynergy/profiles.toml"),
        "top = 3\n[names]\n\"\" = 3\n\"Gut\" = 2\n",
    );
    let a = open(root, "tasks");
    assert_eq!(a.profiles().active(), &fs_settings::ProfileRef::Base);
    let causes: Vec<(String, String)> = a
        .problems()
        .iter()
        .filter(|p| name(p) == "profile")
        .map(|p| {
            (
                text(p, "cause").unwrap_or_default(),
                text(p, "name").unwrap_or_default(),
            )
        })
        .collect();
    assert!(
        causes.contains(&("unknown".to_owned(), "7".to_owned())),
        "{causes:?}"
    );
    assert!(
        causes.contains(&("name-invalid".to_owned(), String::new())),
        "{causes:?}"
    );
    assert_eq!(a.profiles().names().count(), 1);
}

/// N4 (P4): id 0, an id used twice and an id above `top` are `profile{cause=id-invalid}` and left out.
#[test]
fn k45_index_ids_are_checked() {
    let t = tmp();
    let root = t.path();
    put(
        &root.join("cfg/freesynergy/profiles.toml"),
        "top = 2\n[names]\n\"Null\" = 0\n\"A\" = 1\n\"B\" = 1\n\"C\" = 9\n\"D\" = 2\n",
    );
    let a = open(root, "tasks");
    let mut bad: Vec<(String, String)> = a
        .problems()
        .iter()
        .filter(|p| name(p) == "profile")
        .map(|p| {
            (
                text(p, "cause").unwrap_or_default(),
                text(p, "name").unwrap_or_default(),
            )
        })
        .collect();
    bad.sort();
    let want: Vec<(String, String)> = ["B", "C", "Null"]
        .iter()
        .map(|n| ("id-invalid".to_owned(), (*n).to_owned()))
        .collect();
    assert_eq!(bad, want);
    let names: Vec<String> = a
        .profiles()
        .names()
        .map(|n| n.as_str().to_owned())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(
        names.contains(&"A".to_owned()) && names.contains(&"D".to_owned()),
        "{names:?}"
    );
}

/// D1 (review): deleting a profile leaves no part of it in the program's own file.
#[test]
fn k46_delete_clears_the_programs_file() {
    let t = tmp();
    let root = t.path();
    let a = open(root, "tasks");
    a.create_profile("Arbeit").unwrap();
    a.switch_profile(Some("Arbeit")).unwrap();
    a.save(&style(), Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    a.save(&style(), Some(&choice("calm")), Target::AllPrograms)
        .unwrap();
    let own = root.join("cfg/freesynergy/tasks/settings.toml");
    let general = root.join("cfg/freesynergy/settings.toml");
    a.save(&style(), Some(&choice("craft")), Target::ThisProgram)
        .unwrap();
    assert!(
        std::fs::read_to_string(&own).unwrap().contains("profiles"),
        "set up: the part is there"
    );
    a.delete_profile("Arbeit").unwrap();
    for f in [own, general] {
        let text = std::fs::read_to_string(&f).unwrap();
        assert!(!text.contains("profiles"), "{}: {text}", f.display());
    }
}

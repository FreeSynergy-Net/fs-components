//! Review A4: where the config and the state folder are one folder (no state folder of its own, or the same variable), the
//! profile file and the device file must not overwrite each other: the device file has its own name.
mod support;

use std::sync::Arc;

use fs_settings::{
    NoSystem, Place, REFRESH, Raw, STYLE, Scope, Settings, Store, Target, Value, XdgFiles,
};
use support::{catalog, choice, key, os, program, str_raw, tmp};

fn one_folder(root: &std::path::Path, name: &str) -> XdgFiles {
    let both = os(&root.join("cfg"));
    XdgFiles::new(
        program(name),
        both.clone(),
        both,
        os(root),
        os(&root.join("etc")),
    )
}

/// A4: a profile value and a device value of the same scope, written into one folder: both are read back, neither
/// overwrote the other.
#[test]
fn a4_profile_and_device_values_do_not_overwrite_each_other() {
    let dir = tmp();
    let x = one_folder(dir.path(), "tasks");
    let (profile_key, device_key) = (key(STYLE), key(REFRESH));
    assert_ne!(
        x.file(&Scope::General, Place::Profile).unwrap(),
        x.file(&Scope::General, Place::Device).unwrap()
    );
    for scope in [Scope::General, Scope::Program(program("tasks"))] {
        x.write(
            &scope,
            Place::Profile,
            0,
            &profile_key,
            Some(&str_raw("calm")),
        )
        .unwrap();
        x.write(&scope, Place::Device, 0, &device_key, Some(&Raw::Int(500)))
            .unwrap();
        let (p, d) = (
            x.profile(&scope, Place::Profile, 0),
            x.profile(&scope, Place::Device, 0),
        );
        assert_eq!(
            p.values.get(&profile_key),
            Some(&str_raw("calm")),
            "{scope:?}"
        );
        assert_eq!(p.values.get(&device_key), None, "{scope:?}");
        assert_eq!(d.values.get(&device_key), Some(&Raw::Int(500)), "{scope:?}");
        assert_eq!(d.values.get(&profile_key), None, "{scope:?}");
    }
}

/// A4 over `Settings`: a per-device key and a profile key saved for both targets, then read by a fresh handle.
#[test]
fn a4_settings_keep_both_places_in_one_folder() {
    let dir = tmp();
    let open = || {
        Settings::open(
            program("tasks"),
            catalog("tasks"),
            Arc::new(one_folder(dir.path(), "tasks")),
            Arc::new(NoSystem),
        )
    };
    let s = open();
    for target in [Target::AllPrograms, Target::ThisProgram] {
        s.save(&key(STYLE), Some(&choice("calm")), target).unwrap();
        s.save(&key(REFRESH), Some(&Value::Whole(500)), target)
            .unwrap();
        let again = open();
        assert_eq!(
            again.get(&key(STYLE)).unwrap().value,
            choice("calm"),
            "{target:?}"
        );
        assert_eq!(
            again.get(&key(REFRESH)).unwrap().value,
            Value::Whole(500),
            "{target:?}"
        );
    }
}

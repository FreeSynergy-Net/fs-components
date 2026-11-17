//! An instance file that cannot be read in full may hide a lock: the file store then writes nothing (fail closed).
#![cfg(feature = "xdg-files")]
mod support;

use fs_settings::{Place, Scope, SettingsStore};
use support::{files, name, put, str_raw, style, text, tmp};

fn write(f: &fs_settings::XdgFiles) -> Result<(), fs_settings::Coded<fs_settings::SettingCode>> {
    f.write(
        &Scope::General,
        Place::Profile,
        0,
        &style(),
        Some(&str_raw("craft")),
    )
}

#[test]
fn a_broken_or_odd_instance_file_closes_the_store() {
    for (what, body) in [
        ("broken toml", "[values\n"),
        (
            "lock without a reason",
            "[locks]\n\"fs.look.style\" = { by = \"a\", at = 9 }\n",
        ),
    ] {
        let t = tmp();
        let f = files(t.path(), "tasks");
        put(&t.path().join("etc/freesynergy/instance.toml"), body);
        let e = write(&f).unwrap_err();
        assert_eq!(
            (name(&e), text(&e, "which").as_deref()),
            ("read-only", Some("instance")),
            "{what}"
        );
        assert!(
            f.profile(&Scope::General, Place::Profile, 0)
                .values
                .is_empty(),
            "{what}"
        );
    }
}

#[test]
fn no_instance_file_or_a_good_one_writes() {
    let t = tmp();
    let f = files(t.path(), "tasks");
    write(&f).unwrap();
    put(
        &t.path().join("etc/freesynergy/instance.toml"),
        "[values]\n\"fs.look.scheme\" = \"dark\"\n",
    );
    write(&f).unwrap();
}

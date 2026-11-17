//! `fs-texts` and `texts` share one folder in the file store (feature `xdg-files`).
#![cfg(feature = "xdg-files")]
mod support;

use std::sync::Arc;

use fs_settings::{NoSystem, Place, Scope, Settings, SettingsStore};
use support::{catalog, files, name, program, str_raw, style, tmp};

/// Both spellings land in the same folder and file: written with one, read with the other; the guard against another
/// program is the same for both.
#[test]
fn l11_both_spellings_share_the_folder() {
    let dir = tmp();
    let root = dir.path();
    let (long, short) = (files(root, "fs-texts"), files(root, "texts"));
    let (own_long, own_short) = (
        Scope::Program(program("fs-texts")),
        Scope::Program(program("texts")),
    );
    assert_eq!(
        long.file(&own_long, Place::Profile).unwrap(),
        short.file(&own_short, Place::Profile).unwrap()
    );
    long.write(
        &own_long,
        Place::Profile,
        0,
        &style(),
        Some(&str_raw("calm")),
    )
    .unwrap();
    let read = short.profile(&own_short, Place::Profile, 0);
    assert_eq!(read.values.get(&style()), Some(&str_raw("calm")));
    assert_eq!(read, long.profile(&own_long, Place::Profile, 0));
    // the same program under both spellings may write; another program may not, from either handle
    short
        .write(
            &own_long,
            Place::Profile,
            0,
            &style(),
            Some(&str_raw("craft")),
        )
        .unwrap();
    let other = Scope::Program(program("calendars"));
    for f in [&long, &short] {
        let e = f
            .write(&other, Place::Profile, 0, &style(), Some(&str_raw("calm")))
            .unwrap_err();
        assert_eq!(
            (name(&e), support::text(&e, "kind").as_deref()),
            ("write-failed", Some("not-own"))
        );
    }
    // the settings of the program: saved by one handle, seen by the other, the same result
    let open = |n: &str| {
        Settings::open(
            program(n),
            catalog("texts"),
            Arc::new(files(root, n)),
            Arc::new(NoSystem),
        )
    };
    let (a, b) = (open("fs-texts"), open("texts"));
    a.save(
        &style(),
        Some(&support::choice("calm")),
        fs_settings::Target::ThisProgram,
    )
    .unwrap();
    b.refresh();
    assert_eq!(a.get(&style()), b.get(&style()));
    assert_eq!(b.get(&style()).unwrap().from, fs_settings::Layer::Program);
    assert!(!root.join("cfg/freesynergy/fs-texts").exists());
}

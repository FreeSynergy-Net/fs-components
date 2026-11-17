//! The lift after `settings_v1` (review L5 to L8): the stamp sees a new inode, one folder under two spellings, a broken
//! index is never overwritten, the text files are sound, the program's folder has no `fs-`.
mod support;

use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use fs_components_mf2::{Source, check_pairs};
use fs_settings::{
    Declares, NoSystem, Place, ProfileRef, Scope, Settings, Store, TEXT_DE, TEXT_EN, XdgFiles,
};
use support::{catalog, files, name, os, program, put, shell, str_raw, style, tmp};

fn has(text: &str, id: &str) -> bool {
    text.lines().any(|l| l.starts_with(&format!("{id} =")))
}

/// L5: a replacement of the same length with the same modification time is another inode: the stamp changes.
#[cfg(unix)]
#[test]
fn l5_stamp_sees_a_new_inode() {
    let dir = tmp();
    let root = dir.path();
    let f = files(root, "tasks");
    f.write(
        &Scope::General,
        Place::Profile,
        0,
        &style(),
        Some(&str_raw("calm")),
    )
    .unwrap();
    let file = root.join("cfg/freesynergy/settings.toml");
    let before = f.stamp();
    let (len, mtime) = {
        let m = std::fs::metadata(&file).unwrap();
        (m.len(), m.modified().unwrap())
    };
    let text = std::fs::read_to_string(&file)
        .unwrap()
        .replace("calm", "zzzz");
    let other = root.join("cfg/freesynergy/.other");
    std::fs::write(&other, text).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&other)
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    std::fs::rename(&other, &file).unwrap();
    let after = std::fs::metadata(&file).unwrap();
    assert_eq!((after.len(), after.modified().unwrap()), (len, mtime));
    assert_ne!(f.stamp(), before, "same length, same mtime, new inode");
}

fn index_with_state(root: &Path, state: &Path) -> Settings {
    let x = XdgFiles::new(
        program("tasks"),
        os(&root.join("cfg")),
        os(state),
        os(root),
        os(&root.join("etc")),
    );
    Settings::open(
        program("tasks"),
        catalog("tasks"),
        Arc::new(x),
        Arc::new(NoSystem),
    )
}

/// Runs `work` in a thread of its own and fails (instead of hanging) when it does not end.
fn within_ten_seconds<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(work());
    });
    rx.recv_timeout(Duration::from_secs(10))
        .expect("update_index hangs: one folder was locked twice")
}

fn create_and_switch(s: &Settings) -> bool {
    s.create_profile("Arbeit").unwrap();
    s.switch_profile(Some("Arbeit")).unwrap();
    *s.profiles().active() == ProfileRef::Named(fs_settings::ProfileName::new("Arbeit").unwrap())
}

/// L6 (P9): the state folder is a symlink to the config folder: `create_profile` ends, and the one file keeps names and
/// the active profile together.
#[cfg(unix)]
#[test]
fn l6_state_folder_is_a_symlink_to_the_config_folder() {
    let dir = tmp();
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join("cfg")).unwrap();
    std::os::unix::fs::symlink(root.join("cfg"), root.join("statelink")).unwrap();
    let state = root.join("statelink");
    let r = root.clone();
    assert!(within_ten_seconds(move || create_and_switch(
        &index_with_state(&r, &state)
    )));
    let text = std::fs::read_to_string(root.join("cfg/freesynergy/profiles.toml")).unwrap();
    assert!(
        text.contains("Arbeit") && text.contains("active = 1"),
        "{text}"
    );
}

/// L6: the same folder written twice in the variables (the same text) works too and keeps both parts in the file.
#[test]
fn l6_state_folder_equals_the_config_folder() {
    let dir = tmp();
    let root = dir.path().to_path_buf();
    let (r, state) = (root.clone(), root.join("cfg"));
    assert!(within_ten_seconds(move || create_and_switch(
        &index_with_state(&r, &state)
    )));
    let again = index_with_state(&root, &root.join("cfg"));
    assert_eq!(
        again.profiles().names().count(),
        1,
        "the names are still there"
    );
    assert_ne!(
        *again.profiles().active(),
        ProfileRef::Base,
        "and the active profile too"
    );
}

/// L7 (mutant R2): a broken `profiles.toml` is never overwritten by `create_profile`.
#[test]
fn l7_broken_index_is_not_overwritten() {
    let dir = tmp();
    let root = dir.path();
    let file = root.join("cfg/freesynergy/profiles.toml");
    put(&file, "top = = 3\n");
    let before = std::fs::read(&file).unwrap();
    let s = Settings::open(
        program("tasks"),
        catalog("tasks"),
        Arc::new(files(root, "tasks")),
        Arc::new(NoSystem),
    );
    let e = s.create_profile("Arbeit").unwrap_err();
    assert_eq!(name(&e), "file-broken");
    assert_eq!(std::fs::read(&file).unwrap(), before);
}

/// L7: an entry reported as `id-invalid` drops out of the file at the next update (documented).
#[test]
fn l7_invalid_entry_drops_out_at_the_next_update() {
    let dir = tmp();
    let root = dir.path();
    let file = root.join("cfg/freesynergy/profiles.toml");
    put(&file, "top = 2\n[names]\n\"Null\" = 0\n\"Gut\" = 1\n");
    let s = Settings::open(
        program("tasks"),
        catalog("tasks"),
        Arc::new(files(root, "tasks")),
        Arc::new(NoSystem),
    );
    s.create_profile("Neu").unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        !text.contains("Null") && text.contains("Gut") && text.contains("Neu"),
        "{text}"
    );
}

/// L8: the text files are sound (every id in both languages, same variables) and hold the five standard settings and
/// `reason-missing`.
#[test]
fn l8_texts_are_sound_and_complete() {
    let sources = [
        Source {
            language: "en",
            text: TEXT_EN,
        },
        Source {
            language: "de",
            text: TEXT_DE,
        },
    ];
    assert_eq!(check_pairs(&sources), Vec::<String>::new());
    let decls = shell().declarations().unwrap();
    assert_eq!(decls.len(), 5);
    for d in &decls {
        for id in [d.label_key(), d.about_key()] {
            assert!(has(TEXT_EN, &id) && has(TEXT_DE, &id), "{id}");
        }
    }
    assert!(
        has(TEXT_EN, "setting-error-reason-missing")
            && has(TEXT_DE, "setting-error-reason-missing")
    );
}

/// The folder of a program is its short name without `fs-`: `texts`, not `fs-texts`.
#[test]
fn l9_program_folder_has_no_fs_prefix() {
    let dir = tmp();
    let f = files(dir.path(), "texts");
    let own = Scope::Program(program("texts"));
    assert_eq!(
        f.file(&own, Place::Profile).unwrap(),
        dir.path().join("cfg/freesynergy/texts/settings.toml")
    );
}

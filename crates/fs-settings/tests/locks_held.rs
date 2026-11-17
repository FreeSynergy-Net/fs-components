//! Review N1 and N3: the file locks are held in the test itself (deterministic, no hope for an overlap), the index is a
//! read-change-write, and `save` reads the lock fresh.
#![cfg(feature = "xdg-files")]
mod support;

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use fs_settings::{Place, STYLE, Scope, SettingsStore};
use support::{files, key, open_files, profile_id, str_raw, tmp};

/// Holds the exclusive lock of the folder `dir` (as the library does) until the returned file is dropped.
fn hold(dir: &Path) -> std::fs::File {
    std::fs::create_dir_all(dir).unwrap();
    let f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(".settings.lock"))
        .unwrap();
    f.lock().unwrap();
    f
}

const WAIT: Duration = Duration::from_millis(300);

/// N1 (a): while the config folder is locked, `create_profile` of two programs blocks; afterwards both profiles are
/// there with different ids.
#[test]
fn n6_index_change_waits_for_the_lock() {
    let dir = tmp();
    let root = dir.path();
    let (tasks, calendars) = (open_files(root, "tasks"), open_files(root, "calendars"));
    let guard = hold(&root.join("cfg/freesynergy"));
    let (tx, rx) = mpsc::channel();
    std::thread::scope(|s| {
        for (handle, n) in [(&tasks, "Arbeit"), (&calendars, "Vortrag")] {
            let tx = tx.clone();
            s.spawn(move || {
                handle.create_profile(n).unwrap();
                tx.send(n).unwrap();
            });
        }
        assert!(
            rx.recv_timeout(WAIT).is_err(),
            "the index was changed while its lock was held"
        );
        drop(guard);
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
    });
    let both = open_files(root, "tasks");
    assert_eq!(both.profiles().names().count(), 2);
    let (first, second) = (profile_id(&both, "Arbeit"), profile_id(&both, "Vortrag"));
    assert_ne!(first, second);
    assert_eq!(first.min(second), 1);
    assert_eq!(first.max(second), 2);
}

/// N1 (b): the same for writing a value (replaces the timing proof of `n4`; that test stays).
#[test]
fn n7_value_write_waits_for_the_lock() {
    let t = tmp();
    let root = t.path();
    let f = files(root, "tasks");
    let guard = hold(&root.join("cfg/freesynergy"));
    let (tx, rx) = mpsc::channel();
    std::thread::scope(|s| {
        s.spawn(|| {
            f.write(
                &Scope::General,
                Place::Profile,
                0,
                &key(STYLE),
                Some(&str_raw("calm")),
            )
            .unwrap();
            tx.send(()).unwrap();
        });
        assert!(
            rx.recv_timeout(WAIT).is_err(),
            "a value was written while the lock was held"
        );
        drop(guard);
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
    });
    assert_eq!(
        f.profile(&Scope::General, Place::Profile, 0)
            .values
            .get(&key(STYLE)),
        Some(&str_raw("calm"))
    );
}

/// N1 (c) = P3: two handles create one profile each, one after the other: both names, ids 1 and 2.
#[test]
fn n8_two_handles_two_ids() {
    let t = tmp();
    let (a, b) = (
        open_files(t.path(), "tasks"),
        open_files(t.path(), "calendars"),
    );
    a.create_profile("Arbeit").unwrap();
    b.create_profile("Vortrag").unwrap();
    assert_eq!(b.profiles().names().count(), 2);
    assert_eq!(
        (profile_id(&b, "Arbeit"), profile_id(&b, "Vortrag")),
        (1, 2)
    );
    a.refresh();
    assert_eq!(a.profiles().names().count(), 2);
}

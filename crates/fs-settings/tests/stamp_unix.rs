//! L5 (Unix only: inodes): the stamp sees a replacement of the same length with the same modification time.
#![cfg(unix)]
mod support;

use fs_settings::{Place, Scope, Store};
use support::{files, str_raw, style, tmp};

/// L5: a replacement of the same length with the same modification time is another inode: the stamp changes.
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

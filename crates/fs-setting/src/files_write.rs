//! Reading a profile file (with the old ADR 0057 language files) and writing one atomically.
//!
//! Write: read the whole table (an unreadable or broken file is NEVER overwritten: other blocks' keys would be lost),
//! set or remove the one key, write a new file beside it (`create_new`, 0600), sync, rename over the old one, sync the
//! folder (as fs-components-words `setting.rs:122-140`). Two programs writing the same file at the same moment lose no
//! change (b7 decision O16, not "last wins"): the whole read-change-replace runs under an exclusive file lock
//! (`flock` on Unix, `LockFileEx` on Windows; `std::fs::File::lock`) on `.settings.lock` beside the profile; the lock
//! is on that file, not on the profile, because the rename replaces the profile.
//!
//! Pattern: Template method (read → change → replace) + Adapter over `toml`.

use std::io::Write;
use std::path::Path;

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::declare::Place;
use crate::files::{XdgFiles, kind_token, read_table, read_text};
use crate::layer::Scope;
use crate::names::SettingKey;
use crate::store::Snapshot;
use crate::value::Raw;

/// The flat keys of `table` as a snapshot; anything else is a problem with `key` and `file`.
pub(crate) fn snapshot(table: &toml::Table, file: &Path) -> Snapshot {
    let mut s = Snapshot::default();
    for (k, v) in table {
        let raw = match v {
            toml::Value::Boolean(b) => Some(Raw::Bool(*b)),
            toml::Value::Integer(n) => Some(Raw::Int(*n)),
            toml::Value::String(t) => Some(Raw::Str(t.clone())),
            _ => None,
        };
        match (SettingKey::parse(k), raw) {
            (Ok(key), Some(r)) => {
                s.values.insert(key, r);
            }
            (Ok(_), None) => s.problems.push(
                Coded::new(SettingCode::WrongType).with_text(keys::KEY, k.clone()).with_text(keys::FILE, file.display().to_string()),
            ),
            (Err(p), _) => s.problems.push(p),
        }
    }
    s
}

/// The profile `scope` at `place`; problems are inside, never thrown.
pub(crate) fn profile(x: &XdgFiles, scope: &Scope, place: Place) -> Snapshot {
    let file = match x.file(scope, place) {
        Ok(f) => f,
        Err(p) => return Snapshot { values: Default::default(), problems: vec![p] },
    };
    let mut s = match read_table(&file) {
        Ok(t) => snapshot(&t, &file),
        Err(p) => Snapshot { values: Default::default(), problems: vec![p] },
    };
    let language = SettingKey::parse("fs.shell.language");
    if let (Place::Profile, Ok(key), Some(old)) = (place, language, x.legacy(scope)) {
        if !s.values.contains_key(&key) {
            match read_text(&old) {
                Ok(Some(t)) if !t.trim().is_empty() => {
                    s.values.insert(key, Raw::Str(t.trim().to_owned()));
                }
                Ok(_) => {}
                Err(p) => s.problems.push(p),
            }
        }
    }
    s
}

/// Sets or removes `key` in the profile `scope` at `place`.
pub(crate) fn write(x: &XdgFiles, scope: &Scope, place: Place, key: &SettingKey, raw: Option<&Raw>) -> Result<(), Coded<SettingCode>> {
    let file = x.file(scope, place)?;
    let fail = |kind: &str| Coded::new(SettingCode::WriteFailed).with_text(keys::FILE, file.display().to_string()).with_text(keys::KIND, kind);
    if matches!(scope, Scope::Program(p) if *p != x.program) {
        return Err(fail("not-own"));
    }
    let dir = file.parent().ok_or_else(|| fail("other"))?;
    make_dir(dir).map_err(|e| fail(kind_token(&e)))?;
    let _guard = lock_dir(dir).map_err(|e| fail(kind_token(&e)))?;
    let mut table = read_table(&file)?;
    match raw {
        Some(Raw::Bool(b)) => table.insert(key.as_str().to_owned(), toml::Value::Boolean(*b)),
        Some(Raw::Int(n)) => table.insert(key.as_str().to_owned(), toml::Value::Integer(*n)),
        Some(Raw::Str(t)) => table.insert(key.as_str().to_owned(), toml::Value::String(t.clone())),
        None => table.remove(key.as_str()),
    };
    let text = toml::to_string(&table).map_err(|_| fail("other"))?;
    replace(&file, text.as_bytes()).map_err(|e| fail(kind_token(&e)))
}

/// Creates `dir` (0700 on Unix).
fn make_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// The exclusive lock of the folder's profile files; held until the returned file is dropped.
fn lock_dir(dir: &Path) -> std::io::Result<std::fs::File> {
    let mut open = std::fs::OpenOptions::new();
    open.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut open, 0o600);
    let f = open.open(dir.join(".settings.lock"))?;
    f.lock()?;
    Ok(f)
}

/// Writes `bytes` to a new file beside `file` and renames it over `file`.
fn replace(file: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = file.parent().ok_or(std::io::ErrorKind::NotFound)?;
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = dir.join(format!(".settings.{}.{n}.tmp", std::process::id()));
    let mut open = std::fs::OpenOptions::new();
    open.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut open, 0o600);
    let written = open.open(&tmp).and_then(|mut f| f.write_all(bytes).and_then(|()| f.sync_all()));
    if let Err(e) = written.and_then(|()| std::fs::rename(&tmp, file)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    // The folder is synced where a folder can be opened (Unix); elsewhere the rename is the last step.
    #[cfg(unix)]
    std::fs::File::open(dir).and_then(|d| d.sync_all())?;
    Ok(())
}

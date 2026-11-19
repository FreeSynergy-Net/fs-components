//! Helpers of the tests: names, the shell's catalog, handles over one memory store, and reading code parameters.
#![allow(dead_code)]

#[cfg(feature = "xdg-files")]
use std::ffi::OsString;
use std::path::Path;
#[cfg(feature = "xdg-files")]
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "xdg-files")]
use fs_settings::XdgFiles;
use fs_settings::{
    Catalog, Declares, GroupId, Kind, MemoryStore, NoSystem, Owner, OwnerKind, ProgramName, Raw,
    STYLE, SettingCode, SettingKey, Settings, ShellSettings, Value,
};
use fs_types::coded::{Coded, ErrorCode, Param};

pub fn s(x: &str) -> String {
    x.to_owned()
}

pub fn program(name: &str) -> ProgramName {
    ProgramName::new(name).unwrap()
}

pub fn shell() -> ShellSettings {
    ShellSettings::new(
        vec![s("simple"), s("craft"), s("calm")],
        s("simple"),
        vec![s("simple"), s("compact")],
        s("simple"),
    )
}

pub fn key(which: (&str, &str)) -> SettingKey {
    ShellSettings::key(which).unwrap()
}

/// The shell's catalog plus `program.<name>.week-start` (1..=7).
pub fn catalog(name: &str) -> Catalog {
    let mut c = Catalog::of(shell().declarations().unwrap()).unwrap();
    c.add(fs_settings::Declaration::new(
        SettingKey::new(&Owner::new(OwnerKind::Program, name).unwrap(), "week-start").unwrap(),
        Kind::Whole { min: 1, max: 7 },
        Value::Whole(1),
        GroupId::new("calendar").unwrap(),
    ))
    .unwrap();
    c
}

pub fn open(name: &str, store: &MemoryStore) -> Settings {
    Settings::open(
        program(name),
        catalog(name),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    )
}

pub fn choice(x: &str) -> Value {
    Value::Choice(s(x))
}

/// The text parameter `k` of a coded failure.
pub fn text(c: &Coded<SettingCode>, k: &str) -> Option<String> {
    c.params()
        .find(|(n, _)| *n == k)
        .and_then(|(_, v)| match v {
            Param::Text(t) => Some(t.clone()),
            _ => None,
        })
}

pub fn number(c: &Coded<SettingCode>, k: &str) -> Option<u64> {
    c.params()
        .find(|(n, _)| *n == k)
        .and_then(|(_, v)| match v {
            Param::Number(n) => Some(*n),
            _ => None,
        })
}

pub fn name(c: &Coded<SettingCode>) -> &'static str {
    c.code().as_str()
}

/// An actor reference (an absolute IRI) for `name`.
pub fn iri(name: &str) -> fs_types::Reference {
    fs_types::BasicValue::parse(&format!("https://example.org/people/{name}")).unwrap()
}

/// The fixed clock of the tests: 2023-11-14T22:13:20.123Z.
pub fn clock() -> fs_types::Utc {
    fs_types::Utc::from_millis(1_700_000_000_123)
}

/// The argument form of `XdgFiles::new` (an optional variable).
#[cfg(feature = "xdg-files")]
#[allow(clippy::unnecessary_wraps)]
pub fn os(p: &Path) -> Option<OsString> {
    Some(p.as_os_str().to_owned())
}

/// The files of `prog` in a temp folder (config, state, home all below it); no real variable is read.
#[cfg(feature = "xdg-files")]
pub fn files(root: &Path, prog: &str) -> XdgFiles {
    XdgFiles::new(
        program(prog),
        os(&root.join("cfg")),
        os(&root.join("state")),
        os(root),
        os(&root.join("etc")),
    )
}

pub fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[cfg(feature = "xdg-files")]
pub fn general(root: &Path) -> PathBuf {
    // joined part by part: a text with `/` in it gives mixed separators on Windows when shown as text
    root.join("cfg").join("freesynergy").join("settings.toml")
}

pub fn put(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

pub fn style() -> SettingKey {
    key(STYLE)
}

pub fn str_raw(v: &str) -> Raw {
    Raw::Str(v.to_owned())
}

/// A program's settings over the files below `root`.
#[cfg(feature = "xdg-files")]
pub fn open_files(root: &Path, n: &str) -> fs_settings::Settings {
    fs_settings::Settings::open(
        program(n),
        catalog(n),
        Arc::new(files(root, n)),
        Arc::new(fs_settings::NoSystem),
    )
}

/// The id of the named profile `n` of `s`.
pub fn profile_id(s: &fs_settings::Settings, n: &str) -> u64 {
    *s.profiles()
        .get(&fs_settings::ProfileRef::Named(
            fs_settings::ProfileName::new(n).unwrap(),
        ))
        .unwrap()
}

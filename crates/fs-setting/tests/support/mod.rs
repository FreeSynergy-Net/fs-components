//! Helpers of the tests: names, the shell's catalog, handles over one memory store, and reading code parameters.
#![allow(dead_code)]

use std::sync::Arc;

use fs_setting::coded_stub::{Coded, ErrorCode, Param};
use fs_setting::{
    Catalog, Declares, GroupId, Kind, MemoryStore, NoSystem, Owner, OwnerKind, ProgramName, SettingCode, SettingKey, Settings,
    ShellSettings, Value,
};

pub fn s(x: &str) -> String {
    x.to_owned()
}

pub fn program(name: &str) -> ProgramName {
    ProgramName::new(name).unwrap()
}

pub fn shell() -> ShellSettings {
    ShellSettings::new(vec![s("simple"), s("craft"), s("calm")], s("simple"), vec![s("simple"), s("compact")], s("simple"))
}

pub fn key(which: (&str, &str)) -> SettingKey {
    ShellSettings::key(which).unwrap()
}

/// The shell's catalog plus `program.<name>.week-start` (1..=7).
pub fn catalog(name: &str) -> Catalog {
    let mut c = Catalog::of(shell().declarations().unwrap()).unwrap();
    c.add(fs_setting::Declaration::new(
        SettingKey::new(&Owner::new(OwnerKind::Program, name).unwrap(), "week-start").unwrap(),
        Kind::Whole { min: 1, max: 7 },
        Value::Whole(1),
        GroupId::new("calendar").unwrap(),
    ))
    .unwrap();
    c
}

pub fn open(name: &str, store: &MemoryStore) -> Settings {
    Settings::open(program(name), catalog(name), Arc::new(store.clone()), Arc::new(NoSystem))
}

pub fn choice(x: &str) -> Value {
    Value::Choice(s(x))
}

/// The text parameter `k` of a coded failure.
pub fn text(c: &Coded<SettingCode>, k: &str) -> Option<String> {
    c.params().find(|(n, _)| *n == k).and_then(|(_, v)| match v {
        Param::Text(t) => Some(t.clone()),
        _ => None,
    })
}

pub fn number(c: &Coded<SettingCode>, k: &str) -> Option<u64> {
    c.params().find(|(n, _)| *n == k).and_then(|(_, v)| match v {
        Param::Number(n) => Some(*n),
        _ => None,
    })
}

pub fn name(c: &Coded<SettingCode>) -> &'static str {
    c.code().as_str()
}

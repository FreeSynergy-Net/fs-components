//! K6-a4 (review K6-a2 K2): a broken file that holds the human-only key tells a bot no line: the line could be the one of
//! the key. The file and the code stay (the file is broken for everyone); a person gets the line as before.
#![cfg(feature = "xdg-files")]
mod support;

use std::sync::Arc;

use fs_settings::{
    Actor, Declaration, GroupId, Kind, NoSystem, Owner, OwnerKind, SettingCode, SettingKey,
    Settings, Surface, Value,
};
use fs_types::coded::{Coded, Param};
use support::{catalog, files, program, put, tmp};

fn mcp() -> SettingKey {
    SettingKey::new(&Owner::new(OwnerKind::Framework, "mcp").unwrap(), "enabled").unwrap()
}

fn param(p: &Coded<SettingCode>, key: &str) -> Option<Param> {
    p.params().find(|(k, _)| *k == key).map(|(_, v)| v.clone())
}

#[test]
fn a_broken_file_tells_a_bot_no_line() {
    let t = tmp();
    // the key is on line 2, and the file is broken there
    put(
        &t.path().join("etc/freesynergy/instance.toml"),
        "[values]\n\"fs.mcp.enabled\" = = true\n",
    );
    let mut cat = catalog("tasks");
    let decl = Declaration::new(
        mcp(),
        Kind::Flag,
        Value::Flag(false),
        GroupId::new("ai").unwrap(),
    );
    cat.add(decl.per_device().human_only()).unwrap();
    let settings = Settings::open(
        program("tasks"),
        cat,
        Arc::new(files(t.path(), "tasks")),
        Arc::new(NoSystem),
    );
    settings.refresh();
    let broken = |a: Actor| {
        Surface::new(&settings, a)
            .problems()
            .into_iter()
            .find(|p| p.code() == SettingCode::FileBroken)
    };
    let me = broken(Actor::Person).expect("a person is told the file is broken");
    assert_eq!(param(&me, "line"), Some(Param::Number(2)));
    assert!(param(&me, "file").is_some());
    let bot = broken(Actor::Bot).expect("a bot is told the file is broken, too");
    assert_eq!(param(&bot, "line"), None, "{bot:?}");
    assert_eq!(param(&bot, "file"), param(&me, "file"));
}

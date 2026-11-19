//! K6-a: `human_only` ("only a person"): the switch of the local AI entry (`fs.mcp.enabled`) does not exist for a bot, in
//! no way the module offers (list, read, write, search, export, history, problems); a person sees it as always.
mod support;

use std::sync::Arc;

use fs_settings::SettingCode;
use fs_settings::{
    Actor, Admin, Declaration, GroupId, InstanceStore, Kind, LockRights, MemoryStore, NoSystem,
    Owner, OwnerKind, Place, Raw, STYLE, Scope, SettingKey, Settings, SettingsStore, Stampd,
    Surface, TEXT_DE, TEXT_EN, Target, Value,
};
use fs_types::coded::Coded;
use support::{catalog, choice, clock, iri, key, name, program, text};

struct Yes;

impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

fn mcp() -> SettingKey {
    SettingKey::new(&Owner::new(OwnerKind::Framework, "mcp").unwrap(), "enabled").unwrap()
}

/// The declaration the way fs-starter will make it (K6-b): off, per device, group `ai`, only a person.
fn mcp_decl() -> Declaration {
    Declaration::new(
        mcp(),
        Kind::Flag,
        Value::Flag(false),
        GroupId::new("ai").unwrap(),
    )
    .per_device()
    .human_only()
}

struct World {
    store: MemoryStore,
    settings: Settings,
    admin: Admin,
}

fn world() -> World {
    let mut cat = catalog("tasks");
    cat.add(mcp_decl()).unwrap();
    let store = MemoryStore::new();
    let settings = Settings::open(
        program("tasks"),
        cat.clone(),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    );
    let admin = Admin {
        store: Arc::new(store.clone()),
        rights: Arc::new(Yes),
        by: iri("boss"),
        catalog: cat,
        clock,
    };
    World {
        store,
        settings,
        admin,
    }
}

fn ids(list: &[&Declaration]) -> Vec<String> {
    list.iter().map(|d| d.key.as_str().to_owned()).collect()
}

#[test]
fn the_declaration_says_only_a_person() {
    let d = mcp_decl();
    assert!(d.human_only);
    assert_eq!(d.place, Place::Device);
    assert_eq!(d.default, Value::Flag(false), "off until a person opens it");
    assert!(!Declaration::new(mcp(), Kind::Flag, Value::Flag(false), d.group.clone()).human_only);
}

#[test]
fn a_person_sees_and_changes_it_in_every_way() {
    let w = world();
    let me = Surface::new(&w.settings, Actor::Person);
    assert!(ids(&me.list()).contains(&"fs.mcp.enabled".to_owned()));
    assert_eq!(me.search("mcp").len(), 1);
    assert_eq!(me.get(&mcp()).unwrap().value, Value::Flag(false));
    me.save(&mcp(), Some(&Value::Flag(true)), Target::ThisProgram)
        .unwrap();
    assert_eq!(me.get(&mcp()).unwrap().value, Value::Flag(true));
    assert!(me.export().iter().any(|(k, _)| *k == mcp()));
    w.admin
        .set_default(&mcp(), Some(&Value::Flag(true)))
        .unwrap();
    assert!(
        me.history(&w.admin, None, 10)
            .iter()
            .any(|l| l.key == mcp())
    );
}

#[test]
fn a_bot_finds_nothing_of_it_in_any_way() {
    let w = world();
    let me = Surface::new(&w.settings, Actor::Person);
    me.save(&mcp(), Some(&Value::Flag(true)), Target::ThisProgram)
        .unwrap();
    w.admin
        .set_default(&mcp(), Some(&Value::Flag(true)))
        .unwrap();
    w.admin
        .lock(&key(STYLE), &choice("calm"), "house style")
        .unwrap();
    let bot = Surface::new(&w.settings, Actor::Bot);

    // list, search, export
    assert!(!ids(&bot.list()).contains(&"fs.mcp.enabled".to_owned()));
    assert!(!bot.list().is_empty(), "the other settings stay");
    assert_eq!(ids(&bot.search("mcp")), Vec::<String>::new());
    assert_eq!(ids(&bot.search("enabled")), Vec::<String>::new());
    assert!(bot.export().iter().all(|(k, _)| *k != mcp()));
    assert!(bot.export().iter().any(|(k, _)| *k == key(STYLE)));
    // read: the answer is the one for a key nobody declared
    let undeclared =
        SettingKey::new(&Owner::new(OwnerKind::Framework, "nope").unwrap(), "x").unwrap();
    let read = bot.get(&mcp()).unwrap_err();
    assert_eq!(name(&read), "not-declared");
    let nobody = bot.get(&undeclared).unwrap_err();
    assert_eq!(name(&read), name(&nobody));
    assert_eq!(text(&read, "key").as_deref(), Some("fs.mcp.enabled"));
    assert_eq!(text(&nobody, "key").as_deref(), Some("fs.nope.x"));
    // the whole value is the same but for the key: same parameters (names and values), nothing more
    let rest = |c: &Coded<SettingCode>| {
        c.params()
            .filter(|(k, _)| *k != "key")
            .map(|(k, v)| (k.to_owned(), v.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(rest(&read), rest(&nobody));
    assert_eq!(read.params().count(), nobody.params().count());
    // write: refused, and nothing changed
    let write = bot
        .save(&mcp(), Some(&Value::Flag(false)), Target::AllPrograms)
        .unwrap_err();
    assert_eq!(name(&write), "not-declared");
    assert!(bot.save(&mcp(), None, Target::ThisProgram).is_err());
    assert_eq!(me.get(&mcp()).unwrap().value, Value::Flag(true));
    // history: the lines of the key are gone, the others stay
    let log = bot.history(&w.admin, None, 10);
    assert!(log.iter().all(|l| l.key != mcp()));
    assert!(log.iter().any(|l| l.key == key(STYLE)));
    assert!(
        me.history(&w.admin, None, 10)
            .iter()
            .any(|l| l.key == mcp())
    );
}

#[test]
fn problems_and_changes_of_the_key_are_not_told_to_a_bot() {
    let w = world();
    let who = Stampd {
        by: iri("boss"),
        at: clock(),
    };
    // a value of the wrong type under the key, written behind the admin's check: a problem that names the key
    w.store
        .write_instance(&mcp(), Some(&Raw::Str("x".to_owned())), &who)
        .unwrap();
    w.settings.refresh();
    let named = |s: &Surface<'_>| {
        s.problems()
            .iter()
            .any(|p| text(p, "key").as_deref() == Some("fs.mcp.enabled"))
    };
    assert!(named(&Surface::new(&w.settings, Actor::Person)));
    assert!(!named(&Surface::new(&w.settings, Actor::Bot)));
    // another process changes the key; the bot's own save of a visible key does not report that change
    w.store
        .write(
            &Scope::General,
            Place::Device,
            0,
            &mcp(),
            Some(&Raw::Bool(true)),
        )
        .unwrap();
    let bot = Surface::new(&w.settings, Actor::Bot);
    let changes = bot
        .save(&key(STYLE), Some(&choice("calm")), Target::ThisProgram)
        .unwrap();
    assert!(changes.iter().any(|c| c.key == key(STYLE)));
    assert!(changes.iter().all(|c| c.key != mcp()), "{changes:?}");
}

fn line(file: &str, id: &str) -> String {
    file.lines()
        .find_map(|l| l.strip_prefix(&format!("{id} = ")))
        .unwrap_or_else(|| panic!("{id}"))
        .to_owned()
}

/// ADR 0100 E7: the switch opens an entry, it is no right: the words never say "allowed".
#[test]
fn the_words_say_open_never_allowed() {
    let d = mcp_decl();
    assert_eq!(line(TEXT_EN, &d.label_key()), "AI entry (local) open");
    assert_eq!(line(TEXT_DE, &d.label_key()), "KI-Eingang (lokal) offen");
    for (file, words) in [
        (TEXT_EN, ["allow", "permit"]),
        (TEXT_DE, ["erlaubt", "erlaubnis"]),
    ] {
        for id in [d.label_key(), d.about_key()] {
            let t = line(file, &id).to_lowercase();
            assert!(words.iter().all(|w| !t.contains(w)), "{id}: {t}");
        }
    }
}

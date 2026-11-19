//! Fail closed: an actor that is not a person is restricted, also one a later version adds (here `Actor::Other`, test only).
//!
//! Pattern: none, tests only (a table of the actors against the one rule `restricted()` of [`Surface`]).
use std::sync::Arc;

use super::{Actor, Surface};
use crate::admin::{Admin, InstanceStore};
use crate::declare::{Catalog, Declaration};
use crate::layer::Target;
use crate::lock::LockRights;
use crate::memory::MemoryStore;
use crate::names::{GroupId, Owner, OwnerKind, ProgramName, SettingKey};
use crate::settings::Settings;
use crate::store::NoSystem;
use crate::value::{Kind, Value};

struct Yes;

impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

fn key(owner: OwnerKind, name: &str, local: &str) -> SettingKey {
    SettingKey::new(&Owner::new(owner, name).unwrap(), local).unwrap()
}

fn flag(key: SettingKey) -> Declaration {
    Declaration::new(
        key,
        Kind::Flag,
        Value::Flag(false),
        GroupId::new("ai").unwrap(),
    )
}

fn world() -> (Settings, Admin) {
    let secret = key(OwnerKind::Framework, "mcp", "enabled");
    let open = key(OwnerKind::Program, "tasks", "open");
    let mut cat = Catalog::new();
    cat.add(flag(secret).human_only()).unwrap();
    cat.add(flag(open)).unwrap();
    let store = MemoryStore::new();
    let settings = Settings::open(
        ProgramName::new("tasks").unwrap(),
        cat.clone(),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    );
    let admin = Admin {
        store: Arc::new(store) as Arc<dyn InstanceStore>,
        rights: Arc::new(Yes),
        by: fs_types::BasicValue::parse("https://example.org/people/boss").unwrap(),
        catalog: cat,
        clock: crate::admin::system_clock,
    };
    (settings, admin)
}

#[test]
fn every_actor_but_a_person_is_restricted() {
    let secret = key(OwnerKind::Framework, "mcp", "enabled");
    let (settings, admin) = world();
    admin
        .set_default(&secret, Some(&Value::Flag(true)))
        .unwrap();
    for actor in [Actor::Bot, Actor::Other] {
        let s = Surface::new(&settings, actor);
        assert!(!s.sees(&secret), "{actor:?}");
        assert_eq!(s.list().len(), 1, "{actor:?}: list");
        assert!(s.search("mcp").is_empty(), "{actor:?}: search");
        assert!(s.get(&secret).is_err(), "{actor:?}: get");
        assert!(s.save(&secret, None, Target::ThisProgram).is_err());
        assert!(s.export().iter().all(|(k, _)| *k != secret), "{actor:?}");
        assert!(
            s.problems()
                .iter()
                .all(|p| p.params().all(|(k, v)| !(k == crate::codes::keys::KEY
                    && matches!(v, fs_types::coded::Param::Text(t) if t == secret.as_str())))),
            "{actor:?}: problems"
        );
        assert!(
            s.history(&admin, None, 10).iter().all(|l| l.key != secret),
            "{actor:?}: history"
        );
    }
    let me = Surface::new(&settings, Actor::Person);
    assert!(me.sees(&secret));
    assert_eq!(me.history(&admin, None, 10).len(), 1);
}

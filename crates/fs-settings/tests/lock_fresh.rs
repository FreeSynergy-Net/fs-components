//! Review N3 (P2): `save` reads the lock fresh.
mod support;

use fs_settings::{MemoryStore, Place, STYLE, Scope, SettingsStore, Target};
use support::{choice, key, name, open};

/// N3 (P2): the admin locks after the program's last refresh; `save` still sees the lock and writes nothing.
#[test]
fn n9_save_reads_the_lock_fresh() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let boss = fs_settings::Admin {
        store: std::sync::Arc::new(store.clone()),
        rights: std::sync::Arc::new(AllowAll),
        by: support::iri("admin"),
        clock: support::clock,
        catalog: a.catalog().clone(),
    };
    boss.lock(&key(STYLE), &choice("calm"), "house style")
        .unwrap(); // no refresh of `a`
    assert!(
        a.get(&key(STYLE)).unwrap().lock.is_none(),
        "`a` has not looked yet"
    );
    let before = store.stamp();
    for target in [Target::ThisProgram, Target::AllPrograms] {
        let e = a
            .save(&key(STYLE), Some(&choice("craft")), target)
            .unwrap_err();
        assert_eq!(name(&e), "locked");
    }
    assert_eq!(store.stamp(), before, "nothing was written");
    assert_eq!(
        store
            .profile(&Scope::General, Place::Profile, 0)
            .values
            .len(),
        0
    );
}

struct AllowAll;

impl fs_settings::LockRights for AllowAll {
    fn can_lock(&self, _key: &fs_settings::SettingKey) -> bool {
        true
    }
}

//! KB4: the admin's clock reaches the lock to the millisecond; unlocking needs a reason; the log keeps who, when, why.
mod support;

use std::sync::Arc;

use fs_settings::{Admin, LockRights, MemoryStore, Raw, STYLE, SettingKey};
use fs_types::Utc;
use support::{catalog, choice, clock, iri, key, name};

struct Yes;

impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

#[test]
fn kb4_lock_time_reason_and_log() {
    let store = MemoryStore::new();
    let boss = Admin {
        store: Arc::new(store.clone()),
        rights: Arc::new(Yes),
        by: iri("boss"),
        catalog: catalog("tasks"),
        clock,
    };
    let st = key(STYLE);
    assert!(boss.lock(&st, &choice("calm"), "house style").unwrap());
    let l = store.instance().locks.get(&st).cloned().unwrap();
    assert_eq!(l.locked_at, Utc::from_millis(1_700_000_000_123));
    assert_eq!(l.locked_by, "https://example.org/people/boss");
    for empty in ["", "  ", "\t"] {
        let e = boss.unlock(&st, empty).unwrap_err();
        assert_eq!(name(&e), "reason-missing");
    }
    assert!(store.instance().locks.contains_key(&st), "still locked");
    assert!(boss.unlock(&st, "Wartung vorbei").unwrap());
    assert!(!store.instance().locks.contains_key(&st));
    let log = boss.log(None, 10);
    assert_eq!(log.iter().map(|e| e.action).collect::<Vec<_>>(), ["lock", "unlock"]);
    assert_eq!(log[1].reason.as_deref(), Some("Wartung vorbei"));
    assert_eq!(log[0].new, Some(Raw::Str("calm".to_owned())));
    assert!(log.iter().all(|e| e.at == clock() && e.os_user.is_none()));
    // a second unlock changes nothing and logs nothing
    assert!(!boss.unlock(&st, "again").unwrap());
    assert_eq!(boss.log(None, 10).len(), 2);
}

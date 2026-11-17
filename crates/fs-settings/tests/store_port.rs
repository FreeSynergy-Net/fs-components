//! B1 K-cases of the port delta: the store checks the lock itself (KB2), `index_step_ok` (KB3), idempotent instance writes
//! and the log (KB5), health, the new codes' parameters.
mod support;

use fs_settings::coded_stub::{Coded, ErrorCode};
use fs_settings::{
    Health, InstanceStore, LockChange, MemoryStore, Place, ProfileRef, Raw, Scope, SettingCode,
    SettingsStore, Stampd, index_step_ok,
};
use fs_settings::{ProfileName, ProfileSet};
use support::{clock, iri, key, name, number, str_raw, style, text};

fn who(by: &str) -> Stampd {
    Stampd { by: iri(by), at: clock() }
}

fn lock(value: &str, reason: &str) -> LockChange {
    LockChange::Lock { value: str_raw(value), reason: reason.to_owned() }
}

/// KB2: `write` under an instance lock is refused with key, reason and who, the stamp stays; after unlocking it goes through.
/// Also removing, in every scope and place; `clear` of a profile is not refused.
#[test]
fn kb2_write_refuses_a_locked_key() {
    let s = MemoryStore::new();
    assert!(s.write_lock(&style(), &lock("calm", "house style"), &who("boss")).unwrap());
    let before = s.stamp();
    let own = Scope::Program(support::program("tasks"));
    for (scope, place, raw) in [
        (Scope::General, Place::Profile, Some(str_raw("craft"))),
        (own.clone(), Place::Device, Some(str_raw("craft"))),
        (Scope::General, Place::Profile, None),
    ] {
        let e = s.write(&scope, place, 0, &style(), raw.as_ref()).unwrap_err();
        assert_eq!(name(&e), "locked");
        assert_eq!(text(&e, "key").as_deref(), Some("fs.look.style"));
        assert_eq!(text(&e, "reason").as_deref(), Some("house style"));
        assert_eq!(text(&e, "by").as_deref(), Some("https://example.org/people/boss"));
    }
    assert_eq!(s.stamp(), before, "nothing was written");
    assert!(s.profile(&Scope::General, Place::Profile, 0).values.is_empty());
    s.clear(&own, 3).unwrap();
    // another key is free
    s.write(&Scope::General, Place::Profile, 0, &key(fs_settings::SCHEME), Some(&str_raw("dark"))).unwrap();
    assert!(s.write_lock(&style(), &LockChange::Unlock { reason: "done".to_owned() }, &who("boss")).unwrap());
    s.write(&Scope::General, Place::Profile, 0, &style(), Some(&str_raw("craft"))).unwrap();
}

fn set(base: u64, named: &[(&str, u64)]) -> ProfileSet<u64> {
    let mut p = ProfileSet::new(base);
    for (n, id) in named {
        let n = ProfileName::new(n).unwrap();
        p.create(n.clone(), None).unwrap();
        *p.get_mut(&ProfileRef::Named(n)).unwrap() = *id;
    }
    p
}

fn cause(r: Result<(), Coded<SettingCode>>) -> Option<String> {
    r.err().map(|e| {
        assert_eq!(e.code(), SettingCode::Profile);
        text(&e, "cause").unwrap()
    })
}

/// KB3: the one rule for a legal index step.
#[test]
fn kb3_index_step_ok() {
    let old = set(2, &[("Arbeit", 1), ("Vortrag", 2)]);
    // base smaller
    assert_eq!(cause(index_step_ok(&old, &set(1, &[("Arbeit", 1)]))).as_deref(), Some("id-invalid"));
    // a new id above the old base
    assert_eq!(cause(index_step_ok(&old, &set(3, &[("Arbeit", 1), ("Vortrag", 2), ("Neu", 3)]))), None);
    // a new id above the new base
    assert_eq!(cause(index_step_ok(&old, &set(2, &[("Arbeit", 1), ("Vortrag", 2), ("Neu", 3)]))).as_deref(), Some("id-invalid"));
    // an id not in the old index and not above its base (a deleted id, reused)
    let gone = set(2, &[("Vortrag", 2)]);
    assert_eq!(cause(index_step_ok(&gone, &set(2, &[("Vortrag", 2), ("Neu", 1)]))).as_deref(), Some("id-invalid"));
    // new id at or below the old base
    assert_eq!(cause(index_step_ok(&old, &set(3, &[("Arbeit", 1), ("Vortrag", 2), ("Neu", 2)]))).as_deref(), Some("id-invalid"));
    // an id twice, and id 0
    assert_eq!(cause(index_step_ok(&old, &set(4, &[("A", 3), ("B", 3)]))).as_deref(), Some("id-invalid"));
    assert_eq!(cause(index_step_ok(&old, &set(2, &[("Arbeit", 0)]))).as_deref(), Some("id-invalid"));
    // rename keeps the id; delete keeps the base
    assert_eq!(cause(index_step_ok(&old, &set(2, &[("Arbeit", 1), ("Talk", 2)]))), None);
    assert_eq!(cause(index_step_ok(&old, &set(2, &[("Arbeit", 1)]))), None);
    // the failing profile is named
    let e = index_step_ok(&old, &set(2, &[("Neu", 7)])).unwrap_err();
    assert_eq!(text(&e, "name").as_deref(), Some("Neu"));
}

/// KB3 (store): `update_index` of the memory store applies the rule; a refused step changes nothing.
#[test]
fn kb3_update_index_refuses_a_reused_id() {
    let s = MemoryStore::new();
    s.update_index(&mut |i| {
        i.create(ProfileName::new("A").unwrap(), None)?;
        *i.get_mut(&ProfileRef::Named(ProfileName::new("A").unwrap())).unwrap() = 1;
        *i.get_mut(&ProfileRef::Base).unwrap() = 1;
        Ok(())
    })
    .unwrap();
    s.update_index(&mut |i| i.delete(&ProfileName::new("A").unwrap()).map(|_| ())).unwrap();
    let before = s.stamp();
    let e = s
        .update_index(&mut |i| {
            i.create(ProfileName::new("B").unwrap(), None)?;
            *i.get_mut(&ProfileRef::Named(ProfileName::new("B").unwrap())).unwrap() = 1;
            Ok(())
        })
        .unwrap_err();
    assert_eq!(text(&e, "cause").as_deref(), Some("id-invalid"));
    assert_eq!((s.stamp(), s.index().0.names().count()), (before, 0));
}

/// KB5: the same instance write twice is `true` then `false`, one log line; clearing nothing is `false`; locking again
/// the same is `false`, with another value `locked`; unlocking an unlocked key is `false`.
#[test]
fn kb5_instance_writes_are_idempotent() {
    let s = MemoryStore::new();
    assert!(s.write_instance(&style(), Some(&str_raw("calm")), &who("a")).unwrap());
    let stamp = s.stamp();
    assert!(!s.write_instance(&style(), Some(&str_raw("calm")), &who("a")).unwrap());
    assert!(!s.write_instance(&key(fs_settings::SCHEME), None, &who("a")).unwrap());
    assert_eq!(s.stamp(), stamp);
    assert_eq!(s.instance_log(None, 10).len(), 1);
    assert!(s.write_lock(&style(), &lock("craft", "r"), &who("a")).unwrap());
    assert!(!s.write_lock(&style(), &lock("craft", "r"), &who("a")).unwrap());
    let e = s.write_lock(&style(), &lock("calm", "r"), &who("a")).unwrap_err();
    assert_eq!(name(&e), "locked");
    let e = s.write_instance(&style(), None, &who("a")).unwrap_err();
    assert_eq!(name(&e), "locked");
    assert_eq!(s.instance().values.values.get(&style()), Some(&Raw::Str("craft".to_owned())));
    let un = LockChange::Unlock { reason: "x".to_owned() };
    assert!(s.write_lock(&style(), &un, &who("a")).unwrap());
    assert!(!s.write_lock(&style(), &un, &who("a")).unwrap());
    let e = s.write_lock(&style(), &lock("calm", "  "), &who("a")).unwrap_err();
    assert_eq!(name(&e), "reason-missing");
}

/// The instance log: oldest first, ids rise, paging is gap-free, who and when are kept, an unknown `after` gives nothing.
#[test]
fn kb5_log_pages_without_gaps() {
    let s = MemoryStore::new();
    for n in 0..5 {
        s.write_instance(&style(), Some(&str_raw(&format!("v{n}"))), &who("a")).unwrap();
    }
    let all = s.instance_log(None, 100);
    assert_eq!(all.len(), 5);
    assert!(all.windows(2).all(|w| w[0].id < w[1].id));
    assert_eq!(all[0].action, "set");
    assert_eq!((all[0].old.clone(), all[1].old.clone()), (None, Some(str_raw("v0"))));
    assert_eq!((all[0].at, all[0].by.as_str()), (clock(), "https://example.org/people/a"));
    let mut seen = Vec::new();
    let mut after = None;
    loop {
        let page = s.instance_log(after, 2);
        if page.is_empty() {
            break;
        }
        after = page.last().map(|e| e.id);
        seen.extend(page.into_iter().map(|e| e.id));
    }
    assert_eq!(seen, all.iter().map(|e| e.id).collect::<Vec<_>>());
    assert!(s.instance_log(Some(fs_types::ObjectId([9; 16])), 5).is_empty());
}

/// The new codes carry their parameters and message keys (KB1 also in `suite.rs`).
#[test]
fn kb1_new_codes() {
    for (c, n) in [
        (SettingCode::StoreBroken, "store-broken"),
        (SettingCode::StoreNewer, "store-newer"),
        (SettingCode::StoreUnreachable, "store-unreachable"),
        (SettingCode::StoreBusy, "store-busy"),
        (SettingCode::ReadOnly, "read-only"),
        (SettingCode::IndexChanged, "index-changed"),
        (SettingCode::ImportInvalid, "import-invalid"),
        (SettingCode::ImportConflict, "import-conflict"),
        (SettingCode::ExportFailed, "export-failed"),
    ] {
        assert_eq!(c.as_str(), n);
        assert_eq!(fs_settings::coded_stub::message_key(c), format!("setting-error-{n}"));
    }
    let e = Coded::new(SettingCode::StoreNewer)
        .with_text(fs_settings::keys::FILE, "settings.db")
        .with_number(fs_settings::keys::FOUND, 6)
        .with_number(fs_settings::keys::KNOWN, 5);
    assert_eq!((number(&e, "found"), number(&e, "known")), (Some(6), Some(5)));
}

/// A memory store is healthy; the contract's `Health` is a plain value.
#[test]
fn health_of_memory_is_ok() {
    assert_eq!(MemoryStore::new().health(), Health::Ok);
}

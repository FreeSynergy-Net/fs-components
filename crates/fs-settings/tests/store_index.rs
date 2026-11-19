//! KB3: the one rule for a legal profile-index step, and the memory store applying it.
mod support;

use fs_settings::{
    MemoryStore, ProfileName, ProfileRef, ProfileSet, SettingCode, SettingsStore, index_step_ok,
};
use fs_types::coded::Coded;
use support::text;

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
    assert_eq!(
        cause(index_step_ok(&old, &set(1, &[("Arbeit", 1)]))).as_deref(),
        Some("id-invalid")
    );
    // a new id above the old base
    assert_eq!(
        cause(index_step_ok(
            &old,
            &set(3, &[("Arbeit", 1), ("Vortrag", 2), ("Neu", 3)])
        )),
        None
    );
    // a new id above the new base
    assert_eq!(
        cause(index_step_ok(
            &old,
            &set(2, &[("Arbeit", 1), ("Vortrag", 2), ("Neu", 3)])
        ))
        .as_deref(),
        Some("id-invalid")
    );
    // an id not in the old index and not above its base (a deleted id, reused)
    let gone = set(2, &[("Vortrag", 2)]);
    assert_eq!(
        cause(index_step_ok(&gone, &set(2, &[("Vortrag", 2), ("Neu", 1)]))).as_deref(),
        Some("id-invalid")
    );
    // new id at or below the old base
    assert_eq!(
        cause(index_step_ok(
            &old,
            &set(3, &[("Arbeit", 1), ("Vortrag", 2), ("Neu", 2)])
        ))
        .as_deref(),
        Some("id-invalid")
    );
    // an id twice, and id 0
    assert_eq!(
        cause(index_step_ok(&old, &set(4, &[("A", 3), ("B", 3)]))).as_deref(),
        Some("id-invalid")
    );
    assert_eq!(
        cause(index_step_ok(&old, &set(2, &[("Arbeit", 0)]))).as_deref(),
        Some("id-invalid")
    );
    // rename keeps the id; delete keeps the base
    assert_eq!(
        cause(index_step_ok(&old, &set(2, &[("Arbeit", 1), ("Talk", 2)]))),
        None
    );
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
        *i.get_mut(&ProfileRef::Named(ProfileName::new("A").unwrap()))
            .unwrap() = 1;
        *i.get_mut(&ProfileRef::Base).unwrap() = 1;
        Ok(())
    })
    .unwrap();
    s.update_index(&mut |i| i.delete(&ProfileName::new("A").unwrap()).map(|_| ()))
        .unwrap();
    let before = s.stamp();
    let e = s
        .update_index(&mut |i| {
            i.create(ProfileName::new("B").unwrap(), None)?;
            *i.get_mut(&ProfileRef::Named(ProfileName::new("B").unwrap()))
                .unwrap() = 1;
            Ok(())
        })
        .unwrap_err();
    assert_eq!(text(&e, "cause").as_deref(), Some("id-invalid"));
    assert_eq!((s.stamp(), s.index().0.names().count()), (before, 0));
}

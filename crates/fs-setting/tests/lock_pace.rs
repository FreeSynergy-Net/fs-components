//! The new cases of b7's decisions: two programs over one store, the two save buttons,
//! locks, live changes (O7: the beat), who may lock (O17).
mod support;

use std::sync::Arc;

use fs_setting::Declares;
use fs_setting::{
    Admin, GrantedLockRights, Layer, LockGrant, LockRights, MemoryStore, Pace, REFRESH, STYLE,
    Target, Value,
};
use support::{catalog, choice, key, name, open};

/// O7 (b7): the beat is adjustable and runs only while a window is open.
#[test]
fn n2_pace_window_and_interval() {
    let store = MemoryStore::new();
    let a = open("tasks", &store);
    let b = open("calendars", &store);
    let mut pace = Pace::default();
    b.save(&key(STYLE), Some(&choice("craft")), Target::AllPrograms)
        .unwrap();
    assert!(pace.tick(0, &a).is_empty(), "no window: no read");
    assert_eq!(a.get(&key(STYLE)).unwrap().from, Layer::Default);
    pace.window_opened();
    assert_eq!(pace.tick(10, &a).len(), 1, "first beat with a window");
    b.save(&key(STYLE), Some(&choice("calm")), Target::AllPrograms)
        .unwrap();
    assert!(
        pace.tick(500, &a).is_empty(),
        "inside the interval of 1000 ms"
    );
    assert_eq!(pace.tick(1010, &a).len(), 1);
    // The interval is a setting: 250 ms now.
    a.save(&key(REFRESH), Some(&Value::Whole(250)), Target::ThisProgram)
        .unwrap();
    b.save(&key(STYLE), Some(&choice("simple")), Target::AllPrograms)
        .unwrap();
    assert_eq!(pace.tick(1100, &a).len(), 0);
    assert_eq!(pace.interval_ms(), 250);
    assert_eq!(pace.tick(1300, &a).len(), 1);
    // The last window closes: the beat stops; a new window starts it again at once.
    pace.window_closed();
    b.save(&key(STYLE), Some(&choice("craft")), Target::AllPrograms)
        .unwrap();
    assert_eq!(pace.tick(9000, &a).len(), 0);
    assert_eq!(pace.windows(), 0);
    pace.window_opened();
    assert_eq!(pace.tick(9001, &a).len(), 1);
    assert_eq!(Pace::new(1).interval_ms(), Pace::MIN_MS);
    assert_eq!(Pace::new(u64::MAX).interval_ms(), Pace::MAX_MS);
}

/// O17 (b7): only a grant by an instance admin holds `can_lock`; a token or a delegation never does.
#[test]
fn n3_can_lock_only_instance_admin() {
    let st = key(STYLE);
    assert!(GrantedLockRights(LockGrant::InstanceAdmin).can_lock(&st));
    for g in [LockGrant::Token, LockGrant::Delegation, LockGrant::None] {
        assert!(!GrantedLockRights(g).can_lock(&st), "{g:?}");
    }
    let store = MemoryStore::new();
    let by = |g: LockGrant| Admin {
        store: Arc::new(store.clone()),
        rights: Arc::new(GrantedLockRights(g)),
        by: "x".to_owned(),
        catalog: catalog("tasks"),
    };
    assert_eq!(
        name(
            &by(LockGrant::Token)
                .lock(&st, &choice("calm"), "r", 1)
                .unwrap_err()
        ),
        "no-right"
    );
    assert_eq!(
        name(
            &by(LockGrant::Delegation)
                .lock(&st, &choice("calm"), "r", 1)
                .unwrap_err()
        ),
        "no-right"
    );
    by(LockGrant::InstanceAdmin)
        .lock(&st, &choice("calm"), "r", 1)
        .unwrap();
}

/// N5: the declaration of `fs.shell.refresh-ms` takes its limits from `Pace` (one place), in a group of its own.
#[test]
fn k47_refresh_declaration_equals_pace() {
    let d = support::shell()
        .declarations()
        .unwrap()
        .into_iter()
        .find(|d| d.key == key(REFRESH))
        .unwrap();
    let ms = |n: u64| i64::try_from(n).unwrap();
    assert_eq!(
        d.kind,
        fs_setting::Kind::Whole {
            min: ms(Pace::MIN_MS),
            max: ms(Pace::MAX_MS)
        }
    );
    assert_eq!(d.default, Value::Whole(ms(Pace::DEFAULT_MS)));
    assert_eq!(d.group.as_str(), "system");
}

/// N5: a value outside the limits (a declaration with a wide range) is clamped by `tick`.
#[test]
fn k48_tick_clamps() {
    let store = MemoryStore::new();
    let mut c = fs_setting::Catalog::new();
    c.add(fs_setting::Declaration::new(
        key(REFRESH),
        fs_setting::Kind::Whole {
            min: 0,
            max: 1_000_000,
        },
        Value::Whole(1000),
        fs_setting::GroupId::new("system").unwrap(),
    ))
    .unwrap();
    let s = fs_setting::Settings::open(
        support::program("tasks"),
        c,
        Arc::new(store),
        Arc::new(fs_setting::NoSystem),
    );
    let mut pace = Pace::default();
    pace.window_opened();
    s.save(&key(REFRESH), Some(&Value::Whole(10)), Target::ThisProgram)
        .unwrap();
    pace.tick(0, &s);
    assert_eq!(pace.interval_ms(), Pace::MIN_MS);
    s.save(
        &key(REFRESH),
        Some(&Value::Whole(500_000)),
        Target::ThisProgram,
    )
    .unwrap();
    pace.tick(1, &s);
    assert_eq!(pace.interval_ms(), Pace::MAX_MS);
}

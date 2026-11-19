//! K6-a2 (review A1): the history a bot gets has the shape of a log without the hidden lines: pages are full, the pointers
//! lead on, the end is the real end; a person sees everything as before.
mod support;

use std::sync::Arc;

use fs_settings::{
    Actor, Admin, Catalog, Declaration, GroupId, InstanceLogEntry, Kind, LockRights, MemoryStore,
    NoSystem, Owner, OwnerKind, SettingKey, Settings, Surface, Value,
};
use fs_types::ObjectId;
use support::{clock, iri, program};

struct Yes;

impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

fn key(owner: OwnerKind, name: &str, local: &str) -> SettingKey {
    SettingKey::new(&Owner::new(owner, name).unwrap(), local).unwrap()
}

fn flag(k: SettingKey) -> Declaration {
    Declaration::new(
        k,
        Kind::Flag,
        Value::Flag(false),
        GroupId::new("ai").unwrap(),
    )
}

/// A log of `pattern.len()` lines: `true` = a line of the human-only key, `false` = of the visible key.
fn world(pattern: &[bool]) -> (Settings, Admin) {
    let secret = key(OwnerKind::Framework, "mcp", "enabled");
    let open = key(OwnerKind::Program, "tasks", "open");
    let mut cat = Catalog::new();
    cat.add(flag(secret.clone()).human_only()).unwrap();
    cat.add(flag(open.clone())).unwrap();
    let store = MemoryStore::new();
    let settings = Settings::open(
        program("tasks"),
        cat.clone(),
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    );
    let admin = Admin {
        store: Arc::new(store),
        rights: Arc::new(Yes),
        by: iri("boss"),
        catalog: cat,
        clock,
    };
    let (mut on_secret, mut on_open) = (false, false);
    for hidden in pattern {
        let (k, on) = if *hidden {
            (&secret, &mut on_secret)
        } else {
            (&open, &mut on_open)
        };
        *on = !*on; // every write is a change: it is logged
        admin.set_default(k, Some(&Value::Flag(*on))).unwrap();
    }
    (settings, admin)
}

/// Every page the actor gets with `size`, as the list of the keys on it (`h` hidden, `v` visible), until the end.
fn pages(s: &Surface<'_>, admin: &Admin, size: u32) -> Vec<String> {
    let mut out = Vec::new();
    let mut after: Option<ObjectId> = None;
    for _ in 0..100 {
        let page: Vec<InstanceLogEntry> = s.history(admin, after, size);
        if page.is_empty() {
            return out;
        }
        out.push(
            page.iter()
                .map(|l| {
                    if l.key.as_str() == "fs.mcp.enabled" {
                        'h'
                    } else {
                        'v'
                    }
                })
                .collect(),
        );
        after = page.last().map(|l| l.id);
    }
    panic!("the pages do not end");
}

fn bot_pages(pattern: &[bool], size: u32) -> Vec<String> {
    let (settings, admin) = world(pattern);
    pages(&Surface::new(&settings, Actor::Bot), &admin, size)
}

const T: bool = true;
const F: bool = false;

#[test]
fn a_bot_sees_the_pages_of_a_log_without_the_hidden_lines() {
    // hidden line between two visible ones, size 1: the page is not empty, the paging goes on
    assert_eq!(bot_pages(&[F, T, F], 1), ["v", "v"]);
    // a run of hidden lines does not end the paging
    assert_eq!(bot_pages(&[F, T, T, T, F], 1), ["v", "v"]);
    // the same shape as a log that never had the hidden lines
    for size in 1..=5 {
        let with = bot_pages(&[F, T, F, F, T, T, F, T, F, F, F], size);
        let without = bot_pages(&[F; 7], size);
        assert_eq!(with, without, "size {size}");
    }
    // hidden lines at the end and a full last page: the end is the real one
    assert_eq!(bot_pages(&[F, F, T, T], 2), ["vv"]);
    assert_eq!(bot_pages(&[T, T, T], 2), Vec::<String>::new());
}

#[test]
fn the_pages_of_a_bot_are_full_except_the_last() {
    for pattern in [[T, F, T, F, T, F, F, T], [F, T, T, F, F, T, F, F]] {
        let pages = bot_pages(&pattern, 2);
        let (last, rest) = pages.split_last().unwrap();
        assert!(rest.iter().all(|p| p.len() == 2), "{pages:?}");
        assert!(!last.is_empty() && last.len() <= 2, "{pages:?}");
    }
}

/// The ids of the lines a bot gets, in order.
fn bot_ids(pattern: &[bool]) -> Vec<ObjectId> {
    let (settings, admin) = world(pattern);
    Surface::new(&settings, Actor::Bot)
        .history(&admin, None, 1000)
        .iter()
        .map(|l| l.id)
        .collect()
}

/// The low 56 bits of an id: where the old counter over the whole log (hidden lines included) sat.
fn low56(id: &ObjectId) -> u64 {
    let mut b = [0u8; 8];
    b[1..].copy_from_slice(&id.0[9..]);
    u64::from_be_bytes(b)
}

#[test]
fn the_ids_a_bot_gets_do_not_count_the_hidden_lines() {
    // review K6-a2 B1: with a counter over the log, `diff - 1` between two visible ids is the number of hidden lines
    let hidden = 3;
    let mut pattern = Vec::new();
    for _ in 0..40 {
        pattern.push(F);
        pattern.extend([T; 3]);
    }
    let with = bot_ids(&pattern);
    let without = bot_ids(&[F; 40]);
    assert_eq!((with.len(), without.len()), (40, 40));
    for (name, ids, gap) in [("with", &with, hidden + 1), ("without", &without, 1)] {
        assert!(ids.windows(2).all(|w| w[0] < w[1]), "{name}: ids rise");
        let diffs: Vec<u64> = ids
            .windows(2)
            .map(|w| low56(&w[1]).wrapping_sub(low56(&w[0])))
            .collect();
        let counter_like = diffs.iter().filter(|d| **d == gap).count();
        assert!(
            counter_like < 5,
            "{name}: the ids carry a counter: {diffs:?}"
        );
    }
    // the shape of the ids of both logs does not differ by a counter either: no id is just a small number of steps in
    assert!(with.iter().chain(&without).all(|id| low56(id) > 1 << 20));
}

#[test]
fn a_person_gets_every_line_as_before() {
    let (settings, admin) = world(&[F, T, T, F, T]);
    let me = Surface::new(&settings, Actor::Person);
    assert_eq!(pages(&me, &admin, 2), ["vh", "hv", "h"]);
    assert_eq!(me.history(&admin, None, 10).len(), 5);
    assert_eq!(me.history(&admin, None, 0).len(), 0);
}

#[test]
fn size_zero_and_an_unknown_pointer_give_nothing_to_a_bot() {
    let (settings, admin) = world(&[F, T, F]);
    let bot = Surface::new(&settings, Actor::Bot);
    assert_eq!(bot.history(&admin, None, 0).len(), 0);
    // a pointer of another log (the fifth line of a longer one) is unknown here
    let gone = world(&[F; 5]).1.log(None, 5)[4].id;
    assert!(admin.log(None, 10).iter().all(|l| l.id != gone));
    assert_eq!(bot.history(&admin, Some(gone), 5).len(), 0);
}

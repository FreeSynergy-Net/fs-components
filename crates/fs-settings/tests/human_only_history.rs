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

/// The 74 random bits of an id (`rand_a`, `rand_b`): the part the steps of one millisecond add to.
fn rand_of(id: &ObjectId) -> u128 {
    let b = &id.0;
    let a = (u128::from(b[6] & 0x0F) << 8) | u128::from(b[7]);
    let mut l = [0u8; 8];
    l.copy_from_slice(&b[8..]);
    (a << 62) | u128::from(u64::from_be_bytes(l) & ((1u64 << 62) - 1))
}

/// `floor(log2)` of the distance of every two neighbouring ids a bot gets, for `visible` visible lines with `hidden`
/// hidden ones after each (the test clock is fixed: all lines are in one millisecond, so the ids are steps on one chain).
fn log_gaps(visible: usize, hidden: usize) -> Vec<u32> {
    let mut pattern = Vec::new();
    for _ in 0..visible {
        pattern.push(F);
        pattern.extend(std::iter::repeat_n(T, hidden));
    }
    let ids = bot_ids(&pattern);
    assert_eq!(ids.len(), visible);
    assert!(ids.windows(2).all(|w| w[0] < w[1]), "ids rise");
    assert!(ids.windows(2).all(|w| w[0].0[..6] == w[1].0[..6]));
    ids.windows(2)
        .map(|w| (rand_of(&w[1]) - rand_of(&w[0])).ilog2())
        .collect()
}

fn median(mut v: Vec<u32>) -> u32 {
    v.sort_unstable();
    v[v.len() / 2]
}

#[test]
fn the_ids_a_bot_gets_do_not_count_the_hidden_lines() {
    // review K6-a2 B1: with a counter over the log, `diff - 1` between two visible ids is the number of hidden lines.
    // The distance of 4 steps (3 hidden lines) is exactly 4 only if all four steps are 1: P = (0,025)^4 per distance
    // (the step is log-uniform, P(step = 1) = sum 2^-b / 40 = 0,025), over 39 distances far below 1e-9 for 5 or more.
    let pattern: Vec<bool> = (0..40).flat_map(|_| [F, T, T, T]).collect();
    let ids = bot_ids(&pattern);
    assert_eq!(ids.len(), 40);
    assert!(ids.windows(2).all(|w| w[0] < w[1]), "ids rise");
    let counter_like = ids
        .windows(2)
        .filter(|w| rand_of(&w[1]) - rand_of(&w[0]) == 4)
        .count();
    assert!(counter_like < 5, "the ids carry a counter");
    // no id is just a small number of steps in (a fresh start has 72 random bits; P(<= 2^20) = 2^-52 each)
    assert!(ids.iter().all(|id| rand_of(id) > 1 << 20));
}

#[test]
fn the_steps_between_the_ids_are_log_uniform_not_uniform() {
    // Distribution (mutant "uniform step 1..2^40"): the step has b bits, b uniform in 1..=40, so floor(log2) of a distance
    // of ONE step has a median near 18 (b - 1 less a short tail); a uniform step would give about 39. With k = 3 hidden
    // lines the distance is the sum of 4 steps: about the largest of 4 exponents, median near 31 (a uniform step: 40).
    // The bands are wide: 400 distances; to leave a band a median needs 200 values beyond it, where the chance of one
    // is at most 0,3 (mean <= 120, sd <= 9, i.e. 9 sd): P < 1e-15 each, well below 1e-9 for the whole test.
    // What is NOT claimed: that k = 3 looks like k = 0 (it does not: the rest channel of the review K6-a3, accepted).
    let (one, four) = (median(log_gaps(401, 0)), median(log_gaps(401, 3)));
    assert!((12..=30).contains(&one), "one step: median {one}");
    assert!((26..=38).contains(&four), "four steps: median {four}");
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

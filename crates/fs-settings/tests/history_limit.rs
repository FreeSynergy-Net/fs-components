//! K6-a4 (review K6-a2 K1): one call of `Surface::history` reads at most `MAX_LINES_READ` lines of the store, however many
//! hidden lines the log has; when it stops there, a short page says so (`cut`) and does not look like the end.
mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fs_settings::{
    Actor, Admin, Catalog, Declaration, GroupId, Health, IndexFn, Instance, InstanceLogEntry,
    InstanceStore, Kind, LockChange, LockRights, MAX_LINES_READ, MemoryStore, NoSystem, Owner,
    OwnerKind, Place, ProfileSet, RESTRICTED_PAGE_MAX, Raw, Scope, SettingCode, SettingKey,
    Settings, SettingsStore, Snapshot, Stamp, Stampd, Surface, Value,
};
use fs_types::ObjectId;
use fs_types::coded::Coded;
use support::{clock, iri, program};

struct Yes;

impl LockRights for Yes {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        true
    }
}

/// A store that counts what the log reads give out; everything else is the memory store.
struct Counting {
    inner: MemoryStore,
    reads: AtomicUsize,
    lines: AtomicUsize,
}

type Done<T> = Result<T, Coded<SettingCode>>;

impl SettingsStore for Counting {
    fn stamp(&self) -> Stamp {
        self.inner.stamp()
    }
    fn health(&self) -> Health {
        self.inner.health()
    }
    fn instance(&self) -> Instance {
        self.inner.instance()
    }
    fn profile(&self, scope: &Scope, place: Place, profile: u64) -> Snapshot {
        self.inner.profile(scope, place, profile)
    }
    fn write(&self, s: &Scope, p: Place, id: u64, k: &SettingKey, r: Option<&Raw>) -> Done<()> {
        self.inner.write(s, p, id, k, r)
    }
    fn clear(&self, scope: &Scope, profile: u64) -> Done<()> {
        self.inner.clear(scope, profile)
    }
    fn index(&self) -> (ProfileSet<u64>, Vec<Coded<SettingCode>>) {
        self.inner.index()
    }
    fn update_index(&self, f: &mut IndexFn<'_>) -> Done<ProfileSet<u64>> {
        self.inner.update_index(f)
    }
}

impl InstanceStore for Counting {
    fn write_instance(&self, key: &SettingKey, raw: Option<&Raw>, who: &Stampd) -> Done<bool> {
        self.inner.write_instance(key, raw, who)
    }
    fn write_lock(&self, key: &SettingKey, change: &LockChange, who: &Stampd) -> Done<bool> {
        self.inner.write_lock(key, change, who)
    }
    fn instance_log(&self, after: Option<ObjectId>, size: u32) -> Vec<InstanceLogEntry> {
        let out = self.inner.instance_log(after, size);
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.lines.fetch_add(out.len(), Ordering::SeqCst);
        out
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

/// A log of `pattern.len()` lines (`true` = a line of the human-only key) and the store that counts its reads.
fn world(pattern: &[bool]) -> (Settings, Admin, Arc<Counting>) {
    let secret = key(OwnerKind::Framework, "mcp", "enabled");
    let open = key(OwnerKind::Program, "tasks", "open");
    let mut cat = Catalog::new();
    cat.add(flag(secret.clone()).human_only()).unwrap();
    cat.add(flag(open.clone())).unwrap();
    let store = Arc::new(Counting {
        inner: MemoryStore::new(),
        reads: AtomicUsize::new(0),
        lines: AtomicUsize::new(0),
    });
    let settings = Settings::open(
        program("tasks"),
        cat.clone(),
        store.clone(),
        Arc::new(NoSystem),
    );
    let admin = Admin {
        store: store.clone(),
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
    (settings, admin, store)
}

const T: bool = true;
const F: bool = false;

fn reset(store: &Counting) {
    store.reads.store(0, Ordering::SeqCst);
    store.lines.store(0, Ordering::SeqCst);
}

#[test]
fn a_call_reads_a_bounded_number_of_lines_and_says_when_it_stopped() {
    // one visible line, then twice the limit of hidden ones, then a visible line
    let mut pattern = vec![F];
    pattern.extend(vec![T; 2 * MAX_LINES_READ + 10]);
    pattern.push(F);
    let (settings, admin, store) = world(&pattern);
    let bot = Surface::new(&settings, Actor::Bot);
    let first = bot.history(&admin, None, 1);
    assert_eq!(
        (first.len(), first.cut),
        (1, false),
        "a full page is no cut"
    );
    reset(&store);
    // size 1 (it used to be one store read per line): the run of hidden lines ends the call at the limit
    let page = bot.history(&admin, first.last().map(|l| l.id), 1);
    assert!(page.is_empty() && page.cut, "{page:?}: a cut, not an end");
    assert!(store.lines.load(Ordering::SeqCst) <= MAX_LINES_READ);
    assert!(store.reads.load(Ordering::SeqCst) <= MAX_LINES_READ / 64);
    // the person is not cut: the first page of the whole log is full at once
    let me = Surface::new(&settings, Actor::Person);
    assert!(!me.history(&admin, None, 2).cut);
}

#[test]
fn a_short_page_with_a_cut_is_not_the_end() {
    // F, many hidden lines, F, F: the first call finds the first F, reads to the limit and stops; no end is told
    let mut pattern = vec![F];
    pattern.extend(vec![T; MAX_LINES_READ + 5]);
    pattern.extend([F, F]);
    let (settings, admin, _) = world(&pattern);
    let bot = Surface::new(&settings, Actor::Bot);
    let page = bot.history(&admin, None, 5);
    assert_eq!((page.len(), page.cut), (1, true));
    // the pointer is the id of a VISIBLE line: the one the page holds
    assert_eq!(
        page.last().unwrap().key,
        key(OwnerKind::Program, "tasks", "open")
    );
    // the real end is no cut: hidden lines after the last visible one, fewer than the limit
    let (settings, admin, _) = world(&[F, T, T, T]);
    let bot = Surface::new(&settings, Actor::Bot);
    let first = bot.history(&admin, None, 1);
    let end = bot.history(&admin, first.last().map(|l| l.id), 1);
    assert_eq!((first.len(), end.len(), end.cut), (1, 0, false));
}

/// 100 visible lines, then `hidden` hidden ones.
fn visible_then_hidden(hidden: usize) -> Vec<bool> {
    let mut pattern = vec![F; 100];
    pattern.extend(vec![T; hidden]);
    pattern
}

#[test]
fn the_page_shape_of_a_bot_does_not_tell_how_many_lines_are_hidden() {
    // same 100 visible lines; the window of 4096 lines holds 4000 hidden lines in one log, 3000 in the other
    let shape = |hidden: usize, size: u32| {
        let (settings, admin, _) = world(&visible_then_hidden(hidden));
        let page = Surface::new(&settings, Actor::Bot).history(&admin, None, size);
        (page.len(), page.cut)
    };
    for size in [u32::MAX, 1_000_000, 100, RESTRICTED_PAGE_MAX] {
        let (a, b) = (shape(4000, size), shape(3000, size));
        assert_eq!(
            a, b,
            "size {size}: the shape tells the number of hidden lines"
        );
        assert_eq!(a, (RESTRICTED_PAGE_MAX as usize, false), "size {size}");
    }
    // the cap pages on: the pointer of the first page leads to the 36 lines left, and that is the end
    let (settings, admin, _) = world(&visible_then_hidden(4000));
    let bot = Surface::new(&settings, Actor::Bot);
    let first = bot.history(&admin, None, u32::MAX);
    let rest = bot.history(&admin, first.last().map(|l| l.id), u32::MAX);
    assert_eq!((rest.len(), rest.cut), (36, false));
}

#[test]
fn a_person_is_not_capped() {
    // the same logs: the person gets what the read limit gives, as before the cap
    let all = |hidden: usize| {
        let (settings, admin, _) = world(&visible_then_hidden(hidden));
        let page = Surface::new(&settings, Actor::Person).history(&admin, None, u32::MAX);
        (page.len(), page.cut)
    };
    assert_eq!(all(4000), (MAX_LINES_READ, true), "the limit of the read");
    assert_eq!(all(3000), (3100, false), "the whole log");
}

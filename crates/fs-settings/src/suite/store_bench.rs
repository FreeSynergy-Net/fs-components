//! The standard provider of the store suite: the memory store with one wrong behaviour switched on per mutant.
//!
//! Pattern: Decorator ([`BenchStore`] over [`MemoryStore`]) + switch on the fake (as the settings bench).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use fs_types::ObjectId;

use super::store_fixture::StoreFixture;
use crate::admin::{InstanceLogEntry, InstanceStore, LockChange, Stampd};
use crate::codes::{SettingCode, keys};
use crate::coded_stub::Coded;
use crate::declare::Place;
use crate::layer::Scope;
use crate::memory::MemoryStore;
use crate::names::SettingKey;
use crate::profile::ProfileSet;
use crate::store::{Health, IndexFn, Instance, SettingsStore, Snapshot, Stamp};
use crate::value::Raw;

/// The wrong behaviours of a store (ids in `store_mutants.rs`).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreFault {
    /// A yes/no value comes back as a whole number.
    MangledKind,
    /// Removing a value does nothing.
    RemoveIgnored,
    /// The stamp never moves.
    StampFrozen,
    /// The stamp moves on every read.
    StampJitter,
    /// A locked key can be written.
    LockIgnored,
    /// A failed index change is kept half.
    IndexHalf,
    /// A used id can be issued again.
    IdReused,
    /// `clear` does nothing.
    ClearIgnored,
    /// A program's value lands in the general part.
    ScopeShared,
    /// A broken store says `Ok`.
    HealthHidden,
    /// Every log line is written twice.
    LogDup,
    /// Unlocking without a reason goes through.
    ReasonUnchecked,
}

/// The store of the bench: a memory store plus the switch.
#[derive(Clone, Debug)]
pub struct BenchStore {
    inner: MemoryStore,
    fault: Option<StoreFault>,
    broken: Arc<AtomicBool>,
    ticks: Arc<AtomicU64>,
}

impl BenchStore {
    fn new(fault: Option<StoreFault>) -> Self {
        let inner = MemoryStore::new();
        match fault {
            Some(StoreFault::LockIgnored) => inner.set_lax("lock"),
            Some(StoreFault::IdReused) => inner.set_lax("steps"),
            Some(StoreFault::ReasonUnchecked) => inner.set_lax("reason"),
            Some(StoreFault::LogDup) => inner.set_lax("log_dup"),
            _ => {}
        }
        Self {
            inner,
            fault,
            broken: Arc::new(AtomicBool::new(false)),
            ticks: Arc::new(AtomicU64::new(0)),
        }
    }

    fn is(&self, f: StoreFault) -> bool {
        self.fault == Some(f)
    }
}

impl SettingsStore for BenchStore {
    fn stamp(&self) -> Stamp {
        if self.is(StoreFault::StampFrozen) {
            return Stamp(0);
        }
        let jitter = if self.is(StoreFault::StampJitter) {
            self.ticks.fetch_add(1, Ordering::SeqCst) + 1
        } else {
            0
        };
        Stamp(self.inner.stamp().0 + jitter)
    }

    fn health(&self) -> Health {
        if self.broken.load(Ordering::SeqCst) && !self.is(StoreFault::HealthHidden) {
            Health::Defaults(vec![Coded::new(SettingCode::StoreBroken)
                .with_text(keys::FILE, "memory")
                .with_text(keys::CAUSE, "open")])
        } else {
            Health::Ok
        }
    }

    fn instance(&self) -> Instance {
        self.inner.instance()
    }

    fn profile(&self, scope: &Scope, place: Place, profile: u64) -> Snapshot {
        let mut s = self.inner.profile(scope, place, profile);
        if self.is(StoreFault::MangledKind) {
            for v in s.values.values_mut() {
                if let Raw::Bool(b) = v {
                    *v = Raw::Int(i64::from(*b));
                }
            }
        }
        s
    }

    fn write(
        &self,
        scope: &Scope,
        place: Place,
        profile: u64,
        key: &SettingKey,
        raw: Option<&Raw>,
    ) -> Result<(), Coded<SettingCode>> {
        if self.broken.load(Ordering::SeqCst) {
            return Err(Coded::new(SettingCode::ReadOnly).with_text(keys::WHICH, "person"));
        }
        if raw.is_none() && self.is(StoreFault::RemoveIgnored) {
            return Ok(());
        }
        let scope = if self.is(StoreFault::ScopeShared) {
            &Scope::General
        } else {
            scope
        };
        self.inner.write(scope, place, profile, key, raw)
    }

    fn clear(&self, scope: &Scope, profile: u64) -> Result<(), Coded<SettingCode>> {
        if self.is(StoreFault::ClearIgnored) {
            return Ok(());
        }
        self.inner.clear(scope, profile)
    }

    fn index(&self) -> (ProfileSet<u64>, Vec<Coded<SettingCode>>) {
        self.inner.index()
    }

    fn update_index(&self, f: &mut IndexFn<'_>) -> Result<ProfileSet<u64>, Coded<SettingCode>> {
        if !self.is(StoreFault::IndexHalf) {
            return self.inner.update_index(f);
        }
        let mut next = self.inner.index().0;
        if let Err(p) = f(&mut next) {
            self.inner.with(|i| {
                i.index = next;
                i.stamp += 1;
            });
            return Err(crate::settings_profiles::mapped(&p));
        }
        self.inner.update_index(&mut |i| {
            *i = next.clone();
            Ok(())
        })
    }
}

impl InstanceStore for BenchStore {
    fn write_instance(
        &self,
        key: &SettingKey,
        raw: Option<&Raw>,
        who: &Stampd,
    ) -> Result<bool, Coded<SettingCode>> {
        self.inner.write_instance(key, raw, who)
    }

    fn write_lock(
        &self,
        key: &SettingKey,
        change: &LockChange,
        who: &Stampd,
    ) -> Result<bool, Coded<SettingCode>> {
        self.inner.write_lock(key, change, who)
    }

    fn instance_log(&self, after: Option<ObjectId>, size: u32) -> Vec<InstanceLogEntry> {
        self.inner.instance_log(after, size)
    }
}

/// The standard provider: stores in memory, with the fault of the mutant on it.
#[derive(Debug, Default)]
pub struct StoreBench {
    fault: Option<StoreFault>,
    last: Mutex<Option<BenchStore>>,
}

impl StoreBench {
    /// A bench without a fault.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Switches `fault` on (for every store this bench makes afterwards).
    pub fn set_fault(&mut self, fault: StoreFault) {
        self.fault = Some(fault);
    }
}

impl StoreFixture for StoreBench {
    fn fresh(&mut self) -> Arc<dyn InstanceStore> {
        let store = BenchStore::new(self.fault);
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) = Some(store.clone());
        Arc::new(store)
    }

    fn break_store(&mut self, _store: &dyn InstanceStore) -> bool {
        match &*self.last.lock().unwrap_or_else(PoisonError::into_inner) {
            Some(s) => {
                s.broken.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }
}

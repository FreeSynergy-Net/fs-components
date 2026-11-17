//! The bench's store: a memory store with one wrong behaviour switched on per mutant.
//!
//! Pattern: Decorator over [`MemoryStore`] (form of groups `tests/conformance/wrap.rs:19`).

use std::sync::{Arc, Mutex, PoisonError};

use crate::coded_stub::Coded;

use crate::codes::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::admin::{InstanceLogEntry, InstanceStore, LockChange, Stampd};
use crate::memory::MemoryStore;
use crate::names::{ProgramName, SettingKey};
use crate::profile::ProfileSet;
use crate::store::{Health, IndexFn, Instance, SettingsStore, Snapshot, Stamp};
use crate::value::Raw;

/// The wrong behaviours of the store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StoreFault {
    /// A program profile write lands in the general profile.
    ScopeFlattened,
    /// A general write is also written into the profile of `other`.
    WritesEverywhere,
    /// The instance shows no locks.
    LockDropped,
    /// The stamp never changes.
    StampFrozen,
    /// Whole numbers vanish from profiles without a word.
    DropsBad,
    /// Removing a key does nothing.
    RemoveIgnored,
    /// The profile index is never kept (names and switch vanish).
    IndexIgnored,
    /// A named profile does not inherit: while one is active, the base parts read empty.
    NamedIsolated,
}

/// The decorated store; clones share content and fault.
#[derive(Clone, Debug, Default)]
pub(crate) struct Faulty {
    pub(crate) inner: MemoryStore,
    pub(crate) fault: Arc<Mutex<Option<StoreFault>>>,
}

impl Faulty {
    fn is(&self, f: StoreFault) -> bool {
        *self.fault.lock().unwrap_or_else(PoisonError::into_inner) == Some(f)
    }
}

impl SettingsStore for Faulty {
    fn stamp(&self) -> Stamp {
        if self.is(StoreFault::StampFrozen) {
            Stamp(0)
        } else {
            self.inner.stamp()
        }
    }

    fn health(&self) -> Health {
        self.inner.health()
    }

    fn instance(&self) -> Instance {
        let mut i = self.inner.instance();
        if self.is(StoreFault::LockDropped) {
            i.locks.clear();
        }
        i
    }

    fn profile(&self, scope: &Scope, place: Place, profile: u64) -> Snapshot {
        if profile == 0
            && self.is(StoreFault::NamedIsolated)
            && crate::settings::active_id(&self.inner.index().0) != 0
        {
            return Snapshot::default();
        }
        let mut s = self.inner.profile(scope, place, profile);
        if self.is(StoreFault::DropsBad) {
            s.values.retain(|_, r| !matches!(r, Raw::Int(_)));
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
        if raw.is_none() && self.is(StoreFault::RemoveIgnored) {
            return Ok(());
        }
        let scope = if self.is(StoreFault::ScopeFlattened) {
            &Scope::General
        } else {
            scope
        };
        if *scope == Scope::General
            && self.is(StoreFault::WritesEverywhere)
            && let Ok(other) = ProgramName::new("other")
        {
            self.inner
                .write(&Scope::Program(other), place, profile, key, raw)?;
        }
        self.inner.write(scope, place, profile, key, raw)
    }

    fn clear(&self, scope: &Scope, profile: u64) -> Result<(), Coded<SettingCode>> {
        self.inner.clear(scope, profile)
    }

    fn index(&self) -> (ProfileSet<u64>, Vec<Coded<SettingCode>>) {
        self.inner.index()
    }

    fn update_index(&self, f: &mut IndexFn<'_>) -> Result<ProfileSet<u64>, Coded<SettingCode>> {
        if self.is(StoreFault::IndexIgnored) {
            let mut kept = self.inner.index().0;
            f(&mut kept).map_err(|p| crate::settings_profiles::mapped(&p))?;
            Ok(kept)
        } else {
            self.inner.update_index(f)
        }
    }
}

impl InstanceStore for Faulty {
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

    fn instance_log(
        &self,
        after: Option<fs_types::ObjectId>,
        size: u32,
    ) -> Vec<InstanceLogEntry> {
        self.inner.instance_log(after, size)
    }
}

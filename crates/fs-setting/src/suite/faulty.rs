//! The bench's store: a memory store with one wrong behaviour switched on per mutant.
//!
//! Pattern: Decorator over [`MemoryStore`] (form of groups `tests/conformance/wrap.rs:19`).

use std::sync::{Arc, Mutex, PoisonError};

use crate::coded_stub::Coded;

use crate::codes::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::lock::{InstanceStore, Lock};
use crate::memory::MemoryStore;
use crate::names::{ProgramName, SettingKey};
use crate::store::{Instance, Snapshot, Stamp, Store};
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

impl Store for Faulty {
    fn stamp(&self) -> Stamp {
        if self.is(StoreFault::StampFrozen) {
            Stamp(0)
        } else {
            self.inner.stamp()
        }
    }

    fn instance(&self) -> Instance {
        let mut i = self.inner.instance();
        if self.is(StoreFault::LockDropped) {
            i.locks.clear();
        }
        i
    }

    fn profile(&self, scope: &Scope, place: Place) -> Snapshot {
        let mut s = self.inner.profile(scope, place);
        if self.is(StoreFault::DropsBad) {
            s.values.retain(|_, r| !matches!(r, Raw::Int(_)));
        }
        s
    }

    fn write(
        &self,
        scope: &Scope,
        place: Place,
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
        if *scope == Scope::General && self.is(StoreFault::WritesEverywhere) {
            if let Ok(other) = ProgramName::new("other") {
                self.inner.write(&Scope::Program(other), place, key, raw)?;
            }
        }
        self.inner.write(scope, place, key, raw)
    }
}

impl InstanceStore for Faulty {
    fn write_instance(
        &self,
        key: &SettingKey,
        raw: Option<&Raw>,
    ) -> Result<(), Coded<SettingCode>> {
        self.inner.write_instance(key, raw)
    }

    fn write_lock(&self, key: &SettingKey, lock: Option<&Lock>) -> Result<(), Coded<SettingCode>> {
        self.inner.write_lock(key, lock)
    }
}

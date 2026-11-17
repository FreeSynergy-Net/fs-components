//! A store in memory: for tests, the conformance bench, and two programs over one "disk" (clones share the content).
//!
//! Pattern: Strategy ([`Store`], [`InstanceStore`]) over shared state (`Arc<Mutex<_>>`); every write raises the stamp.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

use crate::coded_stub::Coded;

use crate::codes::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::lock::{InstanceStore, Lock};
use crate::names::SettingKey;
use crate::profile::ProfileSet;
use crate::store::{Instance, Snapshot, Stamp, Store};
use crate::value::Raw;

#[derive(Debug, Default)]
struct Inner {
    stamp: u64,
    instance: Instance,
    profiles: BTreeMap<(Scope, bool, u64), BTreeMap<SettingKey, Raw>>,
    index: ProfileSet<u64>,
}

/// The store in memory. `clone()` gives a second handle on the SAME content (a second program on the same device).
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    inner: Arc<Mutex<Inner>>,
}

fn device(place: Place) -> bool {
    place == Place::Device
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn with<T>(&self, f: impl FnOnce(&mut Inner) -> T) -> T {
        let mut g = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        f(&mut g)
    }
}

impl InstanceStore for MemoryStore {
    fn write_instance(
        &self,
        key: &SettingKey,
        raw: Option<&Raw>,
    ) -> Result<(), Coded<SettingCode>> {
        self.with(|i| {
            match raw {
                Some(r) => i.instance.values.values.insert(key.clone(), r.clone()),
                None => i.instance.values.values.remove(key),
            };
            i.stamp += 1;
        });
        Ok(())
    }

    fn write_lock(&self, key: &SettingKey, lock: Option<&Lock>) -> Result<(), Coded<SettingCode>> {
        self.with(|i| {
            match lock {
                Some(l) => i.instance.locks.insert(key.clone(), l.clone()),
                None => i.instance.locks.remove(key),
            };
            i.stamp += 1;
        });
        Ok(())
    }
}

impl Store for MemoryStore {
    fn stamp(&self) -> Stamp {
        Stamp(self.with(|i| i.stamp))
    }

    fn instance(&self) -> Instance {
        self.with(|i| i.instance.clone())
    }

    fn profile(&self, scope: &Scope, place: Place, profile: u64) -> Snapshot {
        self.with(|i| Snapshot {
            values: i
                .profiles
                .get(&(scope.clone(), device(place), profile))
                .cloned()
                .unwrap_or_default(),
            problems: Vec::new(),
        })
    }

    fn write(
        &self,
        scope: &Scope,
        place: Place,
        profile: u64,
        key: &SettingKey,
        raw: Option<&Raw>,
    ) -> Result<(), Coded<SettingCode>> {
        self.with(|i| {
            let p = i
                .profiles
                .entry((scope.clone(), device(place), profile))
                .or_default();
            match raw {
                Some(r) => p.insert(key.clone(), r.clone()),
                None => p.remove(key),
            };
            i.stamp += 1;
        });
        Ok(())
    }

    fn clear(&self, scope: &Scope, profile: u64) -> Result<(), Coded<SettingCode>> {
        self.with(|i| {
            i.profiles
                .retain(|(s, _, p), _| !(s == scope && *p == profile));
            i.stamp += 1;
        });
        Ok(())
    }

    fn index(&self) -> (ProfileSet<u64>, Vec<Coded<SettingCode>>) {
        (self.with(|i| i.index.clone()), Vec::new())
    }

    fn write_index(&self, index: &ProfileSet<u64>) -> Result<(), Coded<SettingCode>> {
        self.with(|i| {
            i.index = index.clone();
            i.stamp += 1;
        });
        Ok(())
    }
}

//! A store in memory: for tests, the conformance bench, and two programs over one "disk" (clones share the content).
//!
//! Pattern: Strategy ([`SettingsStore`], [`InstanceStore`]) over shared state (`Arc<Mutex<_>>`); every write raises the
//! stamp. The instance half is in `memory_instance.rs`. It keeps the whole contract: `write` refuses a locked key, an
//! index change must be a legal step ([`index_step_ok`]), the instance log only grows.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

use crate::coded_stub::Coded;

use crate::admin::InstanceLogEntry;
use crate::codes::SettingCode;
use crate::declare::Place;
use crate::index_step::index_step_ok;
use crate::layer::Scope;
use crate::lock::refuse_locked;
use crate::names::SettingKey;
use crate::profile::ProfileSet;
use crate::store::{Health, IndexFn, Instance, SettingsStore, Snapshot, Stamp};
use crate::value::Raw;

/// Wrong behaviours the conformance bench can switch on. Only `set_lax` (feature `conformance`) sets them, so in a real
/// store they are always off.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Lax {
    lock: bool,
    steps: bool,
    reason: bool,
    log_dup: bool,
}

impl Lax {
    pub(crate) fn lock(self) -> bool {
        self.lock
    }

    pub(crate) fn steps(self) -> bool {
        self.steps
    }

    pub(crate) fn reason(self) -> bool {
        self.reason
    }

    pub(crate) fn log_dup(self) -> bool {
        self.log_dup
    }

    #[cfg(feature = "conformance")]
    pub(crate) fn with(which: &str) -> Self {
        let mut l = Self::default();
        match which {
            "lock" => l.lock = true,
            "steps" => l.steps = true,
            "reason" => l.reason = true,
            _ => l.log_dup = true,
        }
        l
    }
}

#[derive(Debug, Default)]
pub(crate) struct Inner {
    pub(crate) stamp: u64,
    pub(crate) instance: Instance,
    pub(crate) profiles: BTreeMap<(Scope, bool, u64), BTreeMap<SettingKey, Raw>>,
    pub(crate) index: ProfileSet<u64>,
    pub(crate) log: Vec<InstanceLogEntry>,
    pub(crate) lax: Lax,
}

/// The store in memory. `clone()` gives a second handle on the SAME content (a second program on the same device).
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    pub(crate) inner: Arc<Mutex<Inner>>,
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

    /// Switches one wrong behaviour on (the bench only).
    #[cfg(feature = "conformance")]
    pub(crate) fn set_lax(&self, which: &str) {
        self.with(|i| i.lax = Lax::with(which));
    }

    pub(crate) fn with<T>(&self, f: impl FnOnce(&mut Inner) -> T) -> T {
        let mut g = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        f(&mut g)
    }
}

impl SettingsStore for MemoryStore {
    fn stamp(&self) -> Stamp {
        Stamp(self.with(|i| i.stamp))
    }

    fn health(&self) -> Health {
        Health::Ok
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
            if !i.lax.lock() {
                refuse_locked(&i.instance, key)?;
            }
            let p = i
                .profiles
                .entry((scope.clone(), device(place), profile))
                .or_default();
            match raw {
                Some(r) => p.insert(key.clone(), r.clone()),
                None => p.remove(key),
            };
            i.stamp += 1;
            Ok(())
        })
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

    fn update_index(&self, f: &mut IndexFn<'_>) -> Result<ProfileSet<u64>, Coded<SettingCode>> {
        self.with(|i| {
            let mut next = i.index.clone();
            f(&mut next).map_err(|p| crate::settings_profiles::mapped(&p))?;
            if !i.lax.steps() {
                index_step_ok(&i.index, &next)?;
            }
            i.index = next.clone();
            i.stamp += 1;
            Ok(next)
        })
    }
}

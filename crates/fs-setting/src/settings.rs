//! The handle a program holds: effective values, problems, the two ways to save, and live changes.
//!
//! Live: [`Settings::refresh`] reads again only when a stamp changed, compares, and tells every observer each changed
//! key once. Own saves refresh at once; changes by other programs (or by hand in a file) arrive when the shell calls
//! `refresh` on its timer (draft O7: every 1 s by default, changeable). No restart, ever.
//!
//! Pattern: Facade (over store, system source, catalog and [`resolve`]) + Observer ([`OnChange`]). Observers are called
//! outside the lock, after the new state is in place: an observer that calls `get` sees the new value.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::declare::{Catalog, Place};
use crate::layer::{Layer, Scope, Target};
use crate::names::{ProgramName, SettingKey};
use crate::resolve::{Effective, Sources, resolve};
use crate::store::{Snapshot, Stamp, Store, SystemSource};
use crate::value::Value;

/// One key whose effective value (or its layer or lock) changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The key.
    pub key: SettingKey,
    /// Before.
    pub before: Effective,
    /// After.
    pub after: Effective,
}

/// Told of every change, live (the shell: words in the new language, the new style, light/dark).
pub trait OnChange: Send + Sync {
    /// `change` happened; [`Settings::get`] already returns the new value.
    fn changed(&self, change: &Change);
}

#[derive(Default)]
struct State {
    seen: Option<(Stamp, Stamp)>,
    effective: BTreeMap<SettingKey, Effective>,
    problems: Vec<Coded<SettingCode>>,
}

/// The settings of one program.
pub struct Settings {
    program: ProgramName,
    catalog: Catalog,
    store: Arc<dyn Store>,
    system: Arc<dyn SystemSource>,
    state: Mutex<State>,
    observers: Mutex<Vec<Arc<dyn OnChange>>>,
}

impl Settings {
    /// The settings of `program` with `catalog` (its own and its blocks'), read once.
    #[must_use]
    pub fn open(program: ProgramName, catalog: Catalog, store: Arc<dyn Store>, system: Arc<dyn SystemSource>) -> Self {
        let s = Self { program, catalog, store, system, state: Mutex::default(), observers: Mutex::default() };
        let _first = s.refresh();
        s
    }

    /// Whose settings these are.
    #[must_use]
    pub fn program(&self) -> &ProgramName {
        &self.program
    }

    /// What can be set.
    #[must_use]
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// The effective value of `key`; `None` when no block declared it.
    #[must_use]
    pub fn get(&self, key: &SettingKey) -> Option<Effective> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).effective.get(key).cloned()
    }

    /// What went wrong at the last read (shown by the settings window, never dropped).
    #[must_use]
    pub fn problems(&self) -> Vec<Coded<SettingCode>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner).problems.clone()
    }

    /// Adds an observer.
    pub fn observe(&self, o: Arc<dyn OnChange>) {
        self.observers.lock().unwrap_or_else(PoisonError::into_inner).push(o);
    }

    /// Reads again if a stamp changed; returns the changes and tells the observers.
    pub fn refresh(&self) -> Vec<Change> {
        let stamps = (self.store.stamp(), self.system.stamp());
        if self.state.lock().unwrap_or_else(PoisonError::into_inner).seen == Some(stamps) {
            return Vec::new();
        }
        let (effective, problems) = self.read_all();
        let changes: Vec<Change> = {
            let mut st = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let changes = effective
                .iter()
                .filter_map(|(k, after)| {
                    let before = st.effective.get(k)?;
                    (before != after).then(|| Change { key: k.clone(), before: before.clone(), after: after.clone() })
                })
                .collect();
            *st = State { seen: Some(stamps), effective, problems };
            changes
        };
        let observers = self.observers.lock().unwrap_or_else(PoisonError::into_inner).clone();
        for c in &changes {
            for o in &observers {
                o.changed(c);
            }
        }
        changes
    }

    /// Saves `value` (`None` = reset: follow the layer below) for `key` with one of the two buttons.
    ///
    /// # Errors
    /// `not-declared`; `locked` (with the admin's `reason`); the kind's own code when `value` does not fit; the store's
    /// `write-failed`/`no-folder`. Nothing changed then.
    pub fn save(&self, key: &SettingKey, value: Option<&Value>, target: Target) -> Result<Vec<Change>, Coded<SettingCode>> {
        let d = self.catalog.get(key).ok_or_else(|| Coded::new(SettingCode::NotDeclared).with_text(keys::KEY, key.as_str()))?;
        if let Some(lock) = self.get(key).and_then(|e| e.lock) {
            return Err(Coded::new(SettingCode::Locked)
                .with_text(keys::KEY, key.as_str())
                .with_text(keys::REASON, lock.reason)
                .with_text(keys::BY, lock.locked_by));
        }
        let layer = if target == Target::ThisProgram { Layer::Program } else { Layer::Person };
        let raw = match value {
            Some(v) => Some(d.kind.read(key, layer, &v.to_raw())?.to_raw()),
            None => None,
        };
        let own = Scope::Program(self.program.clone());
        match target {
            Target::ThisProgram => self.store.write(&own, d.place, key, raw.as_ref())?,
            Target::AllPrograms => {
                self.store.write(&Scope::General, d.place, key, raw.as_ref())?;
                self.store.write(&own, d.place, key, None)?;
            }
        }
        Ok(self.refresh())
    }

    fn read_all(&self) -> (BTreeMap<SettingKey, Effective>, Vec<Coded<SettingCode>>) {
        let instance = self.store.instance();
        let own = Scope::Program(self.program.clone());
        let read = |scope: &Scope, place: Place| self.store.profile(scope, place);
        let profiles: [(Snapshot, Snapshot); 2] =
            [Place::Profile, Place::Device].map(|place| (read(&Scope::General, place), read(&own, place)));
        let mut problems = instance.values.problems.clone();
        for (general, program) in &profiles {
            problems.extend(general.problems.iter().cloned());
            problems.extend(program.problems.iter().cloned());
            problems.extend(program.values.keys().filter(|k| self.catalog.get(k).is_none()).map(|k| {
                Coded::new(SettingCode::UnknownKey).with_text(keys::KEY, k.as_str()).with_text(keys::LAYER, Layer::Program.as_str())
            }));
        }
        let mut effective = BTreeMap::new();
        for d in self.catalog.iter() {
            let (general, program) = &profiles[usize::from(d.place == Place::Device)];
            let system = self.system.value(&d.key);
            let sources = Sources { instance: &instance, system: system.as_ref(), person: general, program };
            let (e, p) = resolve(d, sources);
            problems.extend(p);
            effective.insert(d.key.clone(), e);
        }
        (effective, problems)
    }
}

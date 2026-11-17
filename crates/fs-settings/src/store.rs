//! Where values are kept ([`SettingsStore`]) and where the system's values come from ([`SystemSource`]).
//!
//! Pattern: Strategy. Standard store: the settings service (database, over the contract `settings`; its adapters live in
//! `fs-settings-rpc` and the program `fs-settings`); test store: [`crate::MemoryStore`]; the old file store
//! (`XdgFiles`, feature `xdg-files`) stays until the lift removes it. Standard system source: in the shell
//! (fs-components-app), over the language chain of ADR 0057 and the portal (draft O8); here only [`NoSystem`].

use std::collections::BTreeMap;

use crate::coded_stub::Coded;

use crate::codes::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::lock::Lock;
use crate::names::SettingKey;
use crate::profile::ProfileSet;
use crate::profile_name::ProfileCode;
use crate::value::Raw;

/// A mark that changes whenever what a source holds may have changed. Equal stamps = nothing to read again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Stamp(pub u64);

/// The values of one profile (or of the instance), as kept, with what went wrong while reading it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// Key → value. A key that is no [`SettingKey`] is not here; it is in `problems` as `key-invalid`.
    pub values: BTreeMap<SettingKey, Raw>,
    /// What went wrong (`file-broken`, `wrong-type` for a table or a list, `key-invalid` ...). Never silent.
    pub problems: Vec<Coded<SettingCode>>,
}

/// The instance layer: the admin's defaults and locks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instance {
    /// The defaults.
    pub values: Snapshot,
    /// The locks, by key.
    pub locks: BTreeMap<SettingKey, Lock>,
}

/// The change an index update applies (see [`SettingsStore::update_index`]).
pub type IndexFn<'a> = dyn FnMut(&mut ProfileSet<u64>) -> Result<(), Coded<ProfileCode>> + 'a;

/// Whether a store serves what it holds (see [`SettingsStore::health`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Health {
    /// The store works.
    Ok,
    /// The store serves only the built-in defaults (and the instance) and says why, never silently: `store-broken{file,
    /// cause}`, `store-newer{file, found, known}`, `store-unreachable{socket}` (one or more).
    Defaults(Vec<Coded<SettingCode>>),
}

/// Where settings are kept. One store serves all programs of the person on this device; each [`crate::Settings`] writes
/// only through its own two targets, into the active profile. (Formerly `Store`; renamed before the first release, no alias.)
pub trait SettingsStore: Send + Sync {
    /// The stamp over everything this store can show.
    fn stamp(&self) -> Stamp;

    /// `Ok`, or the store serves only the built-in defaults and says why. Never silent.
    fn health(&self) -> Health;

    /// The instance layer. Read only for programs.
    fn instance(&self) -> Instance;

    /// The values of `scope` at `place` in the profile with id `profile` (0 = base; a named profile's id from the
    /// index, never reused, so a deleted profile's values in other programs' files never come back).
    fn profile(&self, scope: &Scope, place: Place, profile: u64) -> Snapshot;

    /// Sets (`Some`) or removes (`None`) `key` there; atomic: afterwards it holds the old content or the new, never half.
    /// The store itself refuses a key that is locked in [`Self::instance`] as it is NOW (the second guard next to
    /// [`crate::Settings::save`]); `clear` of a whole profile is not refused.
    ///
    /// # Errors
    /// `locked{key, reason, by}`; `read-only` (the store is in emergency mode); `write-failed`, `no-folder`: nothing changed.
    fn write(
        &self,
        scope: &Scope,
        place: Place,
        profile: u64,
        key: &SettingKey,
        raw: Option<&Raw>,
    ) -> Result<(), Coded<SettingCode>>;

    /// Removes every value of the named profile `profile` in `scope` (after a delete; only the own scopes).
    ///
    /// # Errors
    /// As `write`.
    fn clear(&self, scope: &Scope, profile: u64) -> Result<(), Coded<SettingCode>>;

    /// The person's named profiles (name → id; names of the person, active one of this device) and what went wrong
    /// reading them (an active id without a name: base, and said).
    fn index(&self) -> (ProfileSet<u64>, Vec<Coded<SettingCode>>);

    /// Changes the person's profile index as ONE read-change-write, serialised against every other writer (b7 duty O16,
    /// review N1): reads the index, applies `f`, and keeps the result only when `f` says `Ok`; returns the index as it
    /// is afterwards. Two programs creating a profile at the same moment each get their own id.
    ///
    /// # Errors
    /// The profile codes of `f` as `profile{cause, name}`; `profile{cause=id-invalid}` when the result is no legal
    /// successor ([`crate::index_step_ok`]); `write-failed`, `no-folder`: nothing changed.
    fn update_index(&self, f: &mut IndexFn<'_>) -> Result<ProfileSet<u64>, Coded<SettingCode>>;
}

/// The system's own values (desktop language, light/dark): read, never written.
pub trait SystemSource: Send + Sync {
    /// The stamp; changes when the system says a value changed (portal `SettingChanged`).
    fn stamp(&self) -> Stamp;

    /// The system's value for `key`, if it has one for it (`fs.shell.language`, `fs.look.scheme`).
    fn value(&self, key: &SettingKey) -> Option<Raw>;
}

/// A system that says nothing (tests, and every program until the shell brings its source).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoSystem;

impl SystemSource for NoSystem {
    fn stamp(&self) -> Stamp {
        Stamp(0)
    }

    fn value(&self, _key: &SettingKey) -> Option<Raw> {
        None
    }
}

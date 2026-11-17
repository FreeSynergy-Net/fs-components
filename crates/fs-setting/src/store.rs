//! Where values are kept ([`Store`]) and where the system's values come from ([`SystemSource`]).
//!
//! Pattern: Strategy. Standard store: [`crate::XdgFiles`] (TOML files, XDG Base Directory); test store:
//! [`crate::MemoryStore`]; later an adapter over fs-connect for the instance of a server (draft O11). Standard system
//! source: in the shell (fs-components-app), over the language chain of ADR 0057 and the portal (draft O8); here only
//! [`NoSystem`].

use std::collections::BTreeMap;

use crate::coded_stub::Coded;

use crate::codes::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::lock::Lock;
use crate::names::SettingKey;
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

/// Where settings are kept. One store serves all programs of the person on this device; each [`crate::Settings`] writes
/// only through its own two targets.
pub trait Store: Send + Sync {
    /// The stamp over everything this store can show.
    fn stamp(&self) -> Stamp;

    /// The instance layer. Read only for programs.
    fn instance(&self) -> Instance;

    /// The profile `scope` at `place`.
    fn profile(&self, scope: &Scope, place: Place) -> Snapshot;

    /// Sets (`Some`) or removes (`None`) `key` in the profile `scope` at `place`; atomic: afterwards the profile holds
    /// the old content or the new, never half.
    ///
    /// # Errors
    /// `write-failed`, `no-folder`: nothing changed.
    fn write(&self, scope: &Scope, place: Place, key: &SettingKey, raw: Option<&Raw>) -> Result<(), Coded<SettingCode>>;
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

//! The admin's handle on the instance layer and the instance part of a store: [`InstanceStore`], [`LockChange`],
//! [`Stampd`] (who and when), the append-only [`InstanceLogEntry`].
//!
//! Requirement (c0 decision 6): unlocking needs a reason, like locking; every instance change is logged with who, when
//! and why, and the log is only ever appended to. An unchanged value is no change (`Ok(false)`, no log line), so an
//! admin script can run twice.
//!
//! Pattern: Facade ([`Admin`]) + Strategy ([`InstanceStore`]) + Value objects.

use std::sync::Arc;

use fs_types::{ObjectId, Reference, Utc};

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::declare::Catalog;
use crate::lock::{LockRights, refuse_locked};
use crate::names::SettingKey;
use crate::store::SettingsStore;
use crate::value::{Raw, Value};

/// Who changed the instance layer, and when. The service fills `at` (and the operating-system user) itself; a caller's
/// clock is never trusted there. A library `Admin` uses its own clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stampd {
    /// The acting person (an absolute IRI, an actor reference of fs-access).
    pub by: Reference,
    /// When.
    pub at: Utc,
}

/// A change of a lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockChange {
    /// Value and lock together, in one step (ADR 0100 decision 5).
    Lock {
        /// The instance value the lock pins.
        value: Raw,
        /// Why; empty after trimming is `reason-missing`.
        reason: String,
    },
    /// Lifts the lock; the instance value stays as the default. The reason is mandatory (c0 decision 6): empty after
    /// trimming is `reason-missing{key}` (the existing code).
    Unlock {
        /// Why.
        reason: String,
    },
}

/// One line of the instance log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceLogEntry {
    /// The line's id (a `UUIDv7`: ids rise with time and with the order of writing).
    pub id: ObjectId,
    /// When.
    pub at: Utc,
    /// Who (an IRI).
    pub by: String,
    /// The operating-system user the service saw, when it knows one.
    pub os_user: Option<String>,
    /// `set`, `clear`, `lock` or `unlock`.
    pub action: &'static str,
    /// The key.
    pub key: SettingKey,
    /// The reason (lock and unlock).
    pub reason: Option<String>,
    /// The value before.
    pub old: Option<Raw>,
    /// The value after.
    pub new: Option<Raw>,
}

/// A store whose instance layer can be written (memory; the instance service; the instance database).
pub trait InstanceStore: SettingsStore {
    /// Sets or removes an instance default. `Ok(false)` = the same value was there, nothing written, nothing logged.
    ///
    /// # Errors
    /// `locked` (also clearing a locked key); `write-failed`; `read-only`.
    fn write_instance(
        &self,
        key: &SettingKey,
        raw: Option<&Raw>,
        who: &Stampd,
    ) -> Result<bool, Coded<SettingCode>>;

    /// Locks (value and lock together) or unlocks. `Ok(false)` = nothing changed (the same lock again, or no lock to lift).
    ///
    /// # Errors
    /// `reason-missing` (empty after trimming); `locked` (locking a key that is locked with another value or reason);
    /// `write-failed`; `read-only`.
    fn write_lock(
        &self,
        key: &SettingKey,
        change: &LockChange,
        who: &Stampd,
    ) -> Result<bool, Coded<SettingCode>>;

    /// The instance log, oldest first, the lines after `after` (all when `None`), at most `size`. An unknown `after`
    /// gives no lines.
    fn instance_log(&self, after: Option<ObjectId>, size: u32) -> Vec<InstanceLogEntry>;
}

/// The clock of the library handle: the system time in UTC milliseconds.
#[must_use]
pub fn system_clock() -> Utc {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX));
    Utc::from_millis(ms)
}

/// The admin's handle on the instance layer (the configurator holds it; programs never do).
pub struct Admin {
    /// The store.
    pub store: Arc<dyn InstanceStore>,
    /// The rights of the acting admin.
    pub rights: Arc<dyn LockRights>,
    /// Who acts (actor reference, an IRI).
    pub by: Reference,
    /// What can be set.
    pub catalog: Catalog,
    /// The clock (a plain function, so a test can bring a fixed one; [`system_clock`] for real use).
    pub clock: fn() -> Utc,
}

impl Admin {
    fn who(&self) -> Stampd {
        Stampd {
            by: self.by.clone(),
            at: (self.clock)(),
        }
    }

    fn checked(&self, key: &SettingKey, value: &Value) -> Result<Raw, Coded<SettingCode>> {
        let d = self.catalog.get(key).ok_or_else(|| {
            Coded::new(SettingCode::NotDeclared).with_text(keys::KEY, key.as_str())
        })?;
        Ok(d.kind
            .read(key, crate::layer::Layer::Instance, &value.to_raw())?
            .to_raw())
    }

    fn right(&self, key: &SettingKey) -> Result<(), Coded<SettingCode>> {
        if self.rights.can_lock(key) {
            Ok(())
        } else {
            Err(Coded::new(SettingCode::NoRight).with_text(keys::KEY, key.as_str()))
        }
    }

    fn reason(key: &SettingKey, reason: &str) -> Result<(), Coded<SettingCode>> {
        if reason.trim().is_empty() {
            Err(Coded::new(SettingCode::ReasonMissing).with_text(keys::KEY, key.as_str()))
        } else {
            Ok(())
        }
    }

    /// Sets the instance default of `key` (`None` removes it). `Ok(false)`: it was so already.
    ///
    /// # Errors
    /// `no-right` without the admin right (the same right as locking, `can_lock`: only instance admins hold it, review L4/P8);
    /// `locked` while a lock holds it (also for the one who locked); `not-declared`; the kind's code; `write-failed`.
    pub fn set_default(
        &self,
        key: &SettingKey,
        value: Option<&Value>,
    ) -> Result<bool, Coded<SettingCode>> {
        self.right(key)?;
        refuse_locked(&self.store.instance(), key)?;
        let raw = value.map(|v| self.checked(key, v)).transpose()?;
        self.store.write_instance(key, raw.as_ref(), &self.who())
    }

    /// Locks `key` at `value` with `reason`: value and lock in ONE step, so a crash can leave neither half.
    ///
    /// # Errors
    /// `no-right`; `reason-missing` when `reason` is empty after trimming (O3: a lock always says why); `locked` when it
    /// is locked already (unlock first); `not-declared`; the kind's code; `write-failed`.
    pub fn lock(
        &self,
        key: &SettingKey,
        value: &Value,
        reason: &str,
    ) -> Result<bool, Coded<SettingCode>> {
        self.right(key)?;
        Self::reason(key, reason)?;
        refuse_locked(&self.store.instance(), key)?;
        let raw = self.checked(key, value)?;
        let change = LockChange::Lock {
            value: raw,
            reason: reason.to_owned(),
        };
        self.store.write_lock(key, &change, &self.who())
    }

    /// Unlocks `key` with `reason` (mandatory, c0 decision 6); its instance value stays as the instance default.
    ///
    /// # Errors
    /// `no-right`; `reason-missing` (empty after trimming: it stays locked); `write-failed`.
    pub fn unlock(&self, key: &SettingKey, reason: &str) -> Result<bool, Coded<SettingCode>> {
        self.right(key)?;
        Self::reason(key, reason)?;
        let change = LockChange::Unlock {
            reason: reason.to_owned(),
        };
        self.store.write_lock(key, &change, &self.who())
    }

    /// The instance log, oldest first, after `after`, at most `size` lines.
    #[must_use]
    pub fn log(&self, after: Option<ObjectId>, size: u32) -> Vec<InstanceLogEntry> {
        self.store.instance_log(after, size)
    }
}

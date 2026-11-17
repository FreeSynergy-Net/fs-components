//! The lock on a setting: who may lock (`can_lock`), what a lock holds (`locked_by`, `locked_at`, reason), and the
//! admin's handle that keeps the rules.
//!
//! Requirement (user via b7, 2026-11-17): every setting has a lock symbol, driven by RIGHTS; whoever has the right sees
//! and uses the button, whoever has not does not see the button at all. Locked = NOBODY changes the value, the locker
//! included, until it is unlocked. Ausgestaltung (draft §5, O3): the STATE "locked" (value read only, reason, who, when)
//! is shown to everyone (admin locks are never silent); only the BUTTON depends on the right ([`LockView`]).
//!
//! Value under a lock (decision of the draft, §5): the lock pins the INSTANCE value the locker sets when locking
//! ([`Admin::lock`] takes it), not "the effective value at lock time": an instance lock applies to every person, and each
//! person's effective value differs. The person's own values stay stored and count again after [`Admin::unlock`].
//!
//! Pattern: Strategy (`LockRights`, supplied by fs-access later; `InstanceStore`) + Facade (`Admin`) + Value objects.

use std::sync::Arc;

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::declare::Catalog;
use crate::names::SettingKey;
use crate::resolve::Effective;
use crate::store::Store;
use crate::value::{Raw, Value};

/// A lock on an instance value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock {
    /// Why, as the locker wrote it (shown as written; an empty reason is shown as "no reason given" by the frontend).
    pub reason: String,
    /// Who locked: an actor reference of fs-access (a person; an AI never holds `can_lock`, draft O18).
    pub locked_by: String,
    /// When: seconds since 1970 in UTC (normalisation rule: times in UTC).
    pub locked_at: i64,
}

/// Who may lock which setting. Granted outside this crate (fs-access rights; draft O17: only instance admins grant it).
pub trait LockRights: Send + Sync {
    /// Whether the current actor may lock and unlock `key`.
    fn can_lock(&self, key: &SettingKey) -> bool;
}

/// Nobody may lock (every program's own handle; locking happens in the configurator).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoLockRights;

impl LockRights for NoLockRights {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        false
    }
}

/// How an actor came to the right to lock (fs-access tells; b7 decision O17: `can_lock` is granted by instance admins
/// only, never by a token or a delegation, as ADR 0054 lines 175/181).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockGrant {
    /// Granted by an instance admin through fs-access: the only way to hold `can_lock`.
    InstanceAdmin,
    /// The actor acts with a token (an app, a script, an AI's key): never.
    Token,
    /// The actor acts by delegation from another actor: never.
    Delegation,
    /// No grant.
    None,
}

/// [`LockRights`] by the way the right was granted: only [`LockGrant::InstanceAdmin`] may lock.
#[derive(Clone, Copy, Debug)]
pub struct GrantedLockRights(pub LockGrant);

impl LockRights for GrantedLockRights {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        self.0 == LockGrant::InstanceAdmin
    }
}

/// What the lock component shows for one setting (component "Schloss", draft §6): it only shows; the program decides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockView {
    /// The button exists (the actor has `can_lock`). `false` = no button at all, not a greyed one.
    pub button: bool,
    /// The lock, if the value is locked: shown to everyone, with reason, who and when.
    pub locked: Option<Lock>,
}

/// The view of `key` for an actor with `rights`.
#[must_use]
pub fn lock_view(rights: &dyn LockRights, key: &SettingKey, effective: &Effective) -> LockView {
    LockView { button: rights.can_lock(key), locked: effective.lock.clone() }
}

/// A store whose instance layer can be written (memory; later the instance over fs-connect, draft O11).
pub trait InstanceStore: Store {
    /// Sets or removes an instance default.
    ///
    /// # Errors
    /// `write-failed`.
    fn write_instance(&self, key: &SettingKey, raw: Option<&Raw>) -> Result<(), Coded<SettingCode>>;

    /// Sets or removes a lock.
    ///
    /// # Errors
    /// `write-failed`.
    fn write_lock(&self, key: &SettingKey, lock: Option<&Lock>) -> Result<(), Coded<SettingCode>>;
}

/// The admin's handle on the instance layer (the configurator holds it; programs never do).
pub struct Admin {
    /// The store.
    pub store: Arc<dyn InstanceStore>,
    /// The rights of the acting admin.
    pub rights: Arc<dyn LockRights>,
    /// Who acts (actor reference).
    pub by: String,
    /// What can be set.
    pub catalog: Catalog,
}

impl Admin {
    fn locked(&self, key: &SettingKey) -> Result<(), Coded<SettingCode>> {
        match self.store.instance().locks.get(key) {
            Some(l) => Err(Coded::new(SettingCode::Locked)
                .with_text(keys::KEY, key.as_str())
                .with_text(keys::REASON, l.reason.clone())
                .with_text(keys::BY, l.locked_by.clone())),
            None => Ok(()),
        }
    }

    fn checked(&self, key: &SettingKey, value: &Value) -> Result<Raw, Coded<SettingCode>> {
        let d = self.catalog.get(key).ok_or_else(|| Coded::new(SettingCode::NotDeclared).with_text(keys::KEY, key.as_str()))?;
        Ok(d.kind.read(key, crate::layer::Layer::Instance, &value.to_raw())?.to_raw())
    }

    fn right(&self, key: &SettingKey) -> Result<(), Coded<SettingCode>> {
        if self.rights.can_lock(key) { Ok(()) } else { Err(Coded::new(SettingCode::NoRight).with_text(keys::KEY, key.as_str())) }
    }

    /// Sets the instance default of `key` (`None` removes it).
    ///
    /// # Errors
    /// `locked` while a lock holds it (also for the one who locked); `not-declared`; the kind's code; `write-failed`.
    pub fn set_default(&self, key: &SettingKey, value: Option<&Value>) -> Result<(), Coded<SettingCode>> {
        self.locked(key)?;
        let raw = value.map(|v| self.checked(key, v)).transpose()?;
        self.store.write_instance(key, raw.as_ref())
    }

    /// Locks `key` at `value` with `reason`, at `now` (UTC seconds). Value first, then the lock: a crash in between
    /// leaves an unlocked instance default, never a lock without a value.
    ///
    /// # Errors
    /// `no-right`; `locked` when it is locked already (unlock first); `not-declared`; the kind's code; `write-failed`.
    pub fn lock(&self, key: &SettingKey, value: &Value, reason: &str, now: i64) -> Result<(), Coded<SettingCode>> {
        self.right(key)?;
        self.locked(key)?;
        let raw = self.checked(key, value)?;
        self.store.write_instance(key, Some(&raw))?;
        let lock = Lock { reason: reason.to_owned(), locked_by: self.by.clone(), locked_at: now };
        self.store.write_lock(key, Some(&lock))
    }

    /// Unlocks `key`; its instance value stays as the instance default.
    ///
    /// # Errors
    /// `no-right`; `write-failed`.
    pub fn unlock(&self, key: &SettingKey) -> Result<(), Coded<SettingCode>> {
        self.right(key)?;
        self.store.write_lock(key, None)
    }
}

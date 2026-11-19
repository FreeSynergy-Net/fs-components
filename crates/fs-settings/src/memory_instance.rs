//! The instance half of the memory store: instance defaults, locks, and the append-only log.
//!
//! Pattern: Strategy ([`InstanceStore`] over the shared state of [`MemoryStore`]).

use fs_types::{BasicValue, ObjectId};

use fs_types::coded::Coded;

use crate::admin::{InstanceLogEntry, InstanceStore, LockChange, Stampd};
use crate::codes::{SettingCode, keys};
use crate::lock::{Lock, refuse_locked};
use crate::log_id::next_id;
use crate::memory::{Inner, MemoryStore};
use crate::names::SettingKey;
use crate::value::Raw;

fn append(
    i: &mut Inner,
    who: &Stampd,
    action: &'static str,
    key: &SettingKey,
    reason: Option<&str>,
    old: Option<&Raw>,
    new: Option<&Raw>,
) {
    let times = if i.lax.log_dup() { 2 } else { 1 };
    for _ in 0..times {
        let id = next_id(who.at, i.log.last().map(|e| &e.id));
        i.log.push(InstanceLogEntry {
            id,
            at: who.at,
            by: who.by.canonical(),
            os_user: None,
            action,
            key: key.clone(),
            reason: reason.map(str::to_owned),
            old: old.cloned(),
            new: new.cloned(),
        });
    }
}

fn missing(key: &SettingKey) -> Coded<SettingCode> {
    Coded::new(SettingCode::ReasonMissing).with_text(keys::KEY, key.as_str())
}

impl InstanceStore for MemoryStore {
    fn write_instance(
        &self,
        key: &SettingKey,
        raw: Option<&Raw>,
        who: &Stampd,
    ) -> Result<bool, Coded<SettingCode>> {
        self.with(|i| {
            refuse_locked(&i.instance, key)?;
            let old = i.instance.values.values.get(key).cloned();
            if old.as_ref() == raw {
                return Ok(false);
            }
            match raw {
                Some(r) => i.instance.values.values.insert(key.clone(), r.clone()),
                None => i.instance.values.values.remove(key),
            };
            i.stamp += 1;
            let action = if raw.is_some() { "set" } else { "clear" };
            append(i, who, action, key, None, old.as_ref(), raw);
            Ok(true)
        })
    }

    fn write_lock(
        &self,
        key: &SettingKey,
        change: &LockChange,
        who: &Stampd,
    ) -> Result<bool, Coded<SettingCode>> {
        self.with(|i| match change {
            LockChange::Lock { value, reason } => {
                if reason.trim().is_empty() {
                    return Err(missing(key));
                }
                if let Some(l) = i.instance.locks.get(key) {
                    if i.instance.values.values.get(key) == Some(value) && l.reason == *reason {
                        return Ok(false);
                    }
                    return refuse_locked(&i.instance, key).map(|()| false);
                }
                let old = i.instance.values.values.insert(key.clone(), value.clone());
                i.instance.locks.insert(
                    key.clone(),
                    Lock {
                        reason: reason.clone(),
                        locked_by: who.by.canonical(),
                        locked_at: who.at,
                    },
                );
                i.stamp += 1;
                append(i, who, "lock", key, Some(reason), old.as_ref(), Some(value));
                Ok(true)
            }
            LockChange::Unlock { reason } => {
                if reason.trim().is_empty() && !i.lax.reason() {
                    return Err(missing(key));
                }
                if i.instance.locks.remove(key).is_none() {
                    return Ok(false);
                }
                i.stamp += 1;
                let value = i.instance.values.values.get(key).cloned();
                append(
                    i,
                    who,
                    "unlock",
                    key,
                    Some(reason),
                    value.as_ref(),
                    value.as_ref(),
                );
                Ok(true)
            }
        })
    }

    fn instance_log(&self, after: Option<ObjectId>, size: u32) -> Vec<InstanceLogEntry> {
        self.with(|i| {
            let from = match after {
                None => 0,
                Some(id) => match i.log.iter().position(|e| e.id == id) {
                    Some(p) => p + 1,
                    None => return Vec::new(),
                },
            };
            i.log
                .iter()
                .skip(from)
                .take(size as usize)
                .cloned()
                .collect()
        })
    }
}

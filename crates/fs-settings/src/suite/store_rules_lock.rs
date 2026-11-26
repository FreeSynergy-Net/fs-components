//! The rules of the suite `store` about locks and the instance log: the store refuses a locked key, the log only grows, unlocking needs a reason.
//!
//! Pattern: one rule per behaviour (macro `store_rule!`).

use super::check::{fail, ok, refused_as, std_key};
use super::codes::SuiteCode;
use super::store_check::{got, same, scope_of, store_rule, text, value};
use crate::SettingCode;
use crate::admin::{LockChange, Stampd};
use crate::declare::Place;
use crate::layer::Scope;
use crate::standard::{REFRESH, SCHEME, STYLE};
use crate::value::Raw;

fn who() -> Result<Stampd, fs_conformance::Coded<SuiteCode>> {
    let by = fs_types::BasicValue::parse("https://example.org/people/suite")
        .map_err(|_| fs_conformance::Coded::new(SuiteCode::LogWrong))?;
    Ok(Stampd {
        by,
        at: fs_types::Utc::from_millis(1_700_000_000_000),
    })
}

store_rule!(
    /// The store itself refuses a locked key (the second guard): for a value and for a removal, with nothing written.
    LockedRefused,
    "store-locked-refused",
    |f| {
        let key = std_key(STYLE)?;
        let s = f.fresh();
        let change = LockChange::Lock {
            value: text("calm"),
            reason: "house style".to_owned(),
        };
        let w = who()?;
        ok(s.write_lock(&key, &change, &w), &key, 1)?;
        let at = s.stamp();
        let by = w.by.canonical();
        let locked = [
            ("key", key.as_str()),
            ("reason", "house style"),
            ("by", by.as_str()),
        ];
        let is = SettingCode::Locked;
        refused_as(
            s.write(
                &Scope::General,
                Place::Profile,
                0,
                &key,
                Some(&text("craft")),
            ),
            &key,
            2,
            is,
            &locked,
        )?;
        refused_as(
            s.write(
                &scope_of("demo")?,
                Place::Profile,
                0,
                &key,
                Some(&text("craft")),
            ),
            &key,
            3,
            is,
            &locked,
        )?;
        refused_as(
            s.write(&Scope::General, Place::Profile, 0, &key, None),
            &key,
            4,
            is,
            &locked,
        )?;
        same(&s.stamp(), &at, SuiteCode::StampWrong, &key, 5)?;
        same(
            &value(&*s, &Scope::General, Place::Profile, 0, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            6,
        )?;
        same(
            &value(&*s, &scope_of("demo")?, Place::Profile, 0, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            7,
        )
    }
);

store_rule!(
    /// The instance log only grows: one line per change, oldest first, none for an unchanged value, paged without gaps.
    LogAppend,
    "instance-log-append",
    |f| {
        let key = std_key(SCHEME)?;
        let s = f.fresh();
        let w = who()?;
        same(
            &got(s.write_instance(&key, Some(&text("dark")), &w), &key, 0)?,
            &true,
            SuiteCode::LogWrong,
            &key,
            1,
        )?;
        same(
            &got(s.write_instance(&key, Some(&text("dark")), &w), &key, 0)?,
            &false,
            SuiteCode::LogWrong,
            &key,
            2,
        )?;
        same(
            &got(s.write_instance(&key, Some(&text("light")), &w), &key, 0)?,
            &true,
            SuiteCode::LogWrong,
            &key,
            3,
        )?;
        let all = s.instance_log(None, 100);
        same(&all.len(), &2, SuiteCode::LogWrong, &key, 4)?;
        if all[0].id >= all[1].id || all[1].old != Some(text("dark")) || all[1].action != "set" {
            return Err(fail(SuiteCode::LogWrong, &key, 5));
        }
        let rest = s.instance_log(Some(all[0].id), 100);
        same(
            &rest.iter().map(|e| e.id).collect::<Vec<_>>(),
            &vec![all[1].id],
            SuiteCode::LogWrong,
            &key,
            6,
        )
    }
);

store_rule!(
    /// Unlocking needs a reason: without one (empty after trimming) it stays locked; with one it is free, and the log says why.
    UnlockReason,
    "instance-unlock-reason",
    |f| {
        let key = std_key(REFRESH)?;
        let s = f.fresh();
        let w = who()?;
        let lock = LockChange::Lock {
            value: Raw::Int(500),
            reason: "calm".to_owned(),
        };
        ok(s.write_lock(&key, &lock, &w), &key, 1)?;
        for empty in ["", "  \t"] {
            let un = LockChange::Unlock {
                reason: empty.to_owned(),
            };
            refused_as(
                s.write_lock(&key, &un, &w),
                &key,
                2,
                SettingCode::ReasonMissing,
                &[("key", key.as_str())],
            )?;
        }
        same(
            &s.instance().locks.contains_key(&key),
            &true,
            SuiteCode::LockWrong,
            &key,
            3,
        )?;
        ok(
            s.write_lock(
                &key,
                &LockChange::Unlock {
                    reason: "Wartung vorbei".to_owned(),
                },
                &w,
            ),
            &key,
            4,
        )?;
        same(
            &s.instance().locks.contains_key(&key),
            &false,
            SuiteCode::LockWrong,
            &key,
            5,
        )?;
        let reasons: Vec<Option<String>> = s
            .instance_log(None, 10)
            .into_iter()
            .map(|e| e.reason)
            .collect();
        same(
            &reasons,
            &vec![Some("calm".to_owned()), Some("Wartung vorbei".to_owned())],
            SuiteCode::LogWrong,
            &key,
            6,
        )
    }
);

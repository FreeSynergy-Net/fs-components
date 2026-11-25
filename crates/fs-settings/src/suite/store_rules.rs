//! The rules of the suite `store` (all but the health rule): what every settings store keeps.
//!
//! Pattern: one rule per behaviour (macro `store_rule!`).

use super::check::{fail, ok, refused, refused_as, std_key};
use super::codes::SuiteCode;
use super::store_check::{
    drop_profile, got, make_profile, refuse, same, scope_of, store_rule, value,
};
use crate::SettingCode;
use crate::admin::{LockChange, Stampd};
use crate::declare::Place;
use crate::layer::Scope;
use crate::profile_name::{ProfileName, ProfileRef};
use crate::standard::{REFRESH, SCHEME, STYLE};
use crate::value::Raw;
use fs_types::BasicValue;

fn who() -> Result<Stampd, fs_conformance::Coded<SuiteCode>> {
    let by = fs_types::BasicValue::parse("https://example.org/people/suite")
        .map_err(|_| fs_conformance::Coded::new(SuiteCode::LogWrong))?;
    Ok(Stampd {
        by,
        at: fs_types::Utc::from_millis(1_700_000_000_000),
    })
}

fn text(x: &str) -> Raw {
    Raw::Str(x.to_owned())
}

store_rule!(
    /// A value of each primitive kind comes back as written, in general and program scope, at both places, also in a named profile.
    Roundtrip,
    "store-roundtrip",
    |f| {
        let (key, own) = (std_key(STYLE)?, scope_of("demo")?);
        let s = f.fresh();
        let id = got(make_profile(&*s, "Arbeit"), &key, 1)?;
        for (n, (scope, place, profile, raw)) in [
            (Scope::General, Place::Profile, 0, text("calm")),
            (own.clone(), Place::Profile, 0, Raw::Int(-7)),
            (Scope::General, Place::Device, 0, Raw::Bool(true)),
            (own.clone(), Place::Device, 0, Raw::Bool(false)),
            (own, Place::Profile, id, text("Zürich")),
        ]
        .into_iter()
        .enumerate()
        {
            let step = n as u64 * 2 + 1;
            ok(
                s.write(&scope, place, profile, &key, Some(&raw)),
                &key,
                step,
            )?;
            same(
                &value(&*s, &scope, place, profile, &key),
                &Some(raw),
                SuiteCode::WrongValue,
                &key,
                step + 1,
            )?;
        }
        Ok(())
    }
);

store_rule!(
    /// Removing a value removes it; removing what is not there is no error.
    Remove,
    "store-remove",
    |f| {
        let key = std_key(STYLE)?;
        let s = f.fresh();
        ok(
            s.write(&Scope::General, Place::Profile, 0, &key, None),
            &key,
            1,
        )?;
        ok(
            s.write(
                &Scope::General,
                Place::Profile,
                0,
                &key,
                Some(&text("calm")),
            ),
            &key,
            2,
        )?;
        ok(
            s.write(&Scope::General, Place::Profile, 0, &key, None),
            &key,
            3,
        )?;
        same(
            &value(&*s, &Scope::General, Place::Profile, 0, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            4,
        )
    }
);

store_rule!(
    /// A write moves the stamp (a change is seen), also a removal.
    StampMoves,
    "store-stamp-moves",
    |f| {
        let key = std_key(STYLE)?;
        let s = f.fresh();
        let first = s.stamp();
        ok(
            s.write(
                &Scope::General,
                Place::Profile,
                0,
                &key,
                Some(&text("calm")),
            ),
            &key,
            1,
        )?;
        let second = s.stamp();
        ok(
            s.write(&Scope::General, Place::Profile, 0, &key, None),
            &key,
            2,
        )?;
        if first == second || second == s.stamp() {
            return Err(fail(SuiteCode::StampWrong, &key, 3));
        }
        Ok(())
    }
);

store_rule!(
    /// Reading moves nothing: the stamp stays while nothing is written.
    StampStill,
    "store-stamp-still",
    |f| {
        let key = std_key(STYLE)?;
        let s = f.fresh();
        ok(
            s.write(
                &Scope::General,
                Place::Profile,
                0,
                &key,
                Some(&text("calm")),
            ),
            &key,
            1,
        )?;
        let at = s.stamp();
        let _ = (
            s.profile(&Scope::General, Place::Profile, 0),
            s.index(),
            s.instance(),
            s.health(),
        );
        same(&s.stamp(), &at, SuiteCode::StampWrong, &key, 2)?;
        same(&s.stamp(), &at, SuiteCode::StampWrong, &key, 3)
    }
);

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
    /// An index change is one step: when the change fails, nothing of it is kept.
    IndexAtomic,
    "store-index-atomic",
    |f| {
        let key = std_key(STYLE)?;
        let s = f.fresh();
        let half = s.update_index(&mut |i| {
            i.create(ProfileName::new("Half")?, None)?;
            Err(refuse())
        });
        refused(half, &key, 1)?;
        same(
            &s.index().0.names().count(),
            &0,
            SuiteCode::IndexWrong,
            &key,
            2,
        )?;
        ok(make_profile(&*s, "Whole"), &key, 3)?;
        same(
            &s.index().0.names().count(),
            &1,
            SuiteCode::IndexWrong,
            &key,
            4,
        )
    }
);

store_rule!(
    /// An id once issued is never issued again, also not by a change that asks for it.
    IdsNeverReused,
    "store-ids-never-reused",
    |f| {
        let key = std_key(STYLE)?;
        let s = f.fresh();
        let first = got(make_profile(&*s, "A"), &key, 2)?;
        let second = got(make_profile(&*s, "B"), &key, 3)?;
        ok(drop_profile(&*s, "B"), &key, 3)?;
        let third = got(make_profile(&*s, "C"), &key, 4)?;
        if !(first < second && second < third) {
            return Err(fail(SuiteCode::IndexWrong, &key, 5));
        }
        ok(drop_profile(&*s, "A"), &key, 6)?;
        let again = s.update_index(&mut |i| {
            let extra = ProfileName::new("D").map_err(|_| refuse())?;
            i.create(extra.clone(), None)?;
            if let Some(id) = i.get_mut(&ProfileRef::Named(extra)) {
                *id = first;
            }
            Ok(())
        });
        refused_as(
            again,
            &key,
            7,
            SettingCode::Profile,
            &[("cause", "id-invalid")],
        )?;
        same(
            &s.index().0.names().count(),
            &1,
            SuiteCode::IndexWrong,
            &key,
            8,
        )
    }
);

store_rule!(
    /// Deleting a profile clears its values in both scopes and leaves the base alone.
    DeleteClears,
    "store-delete-clears",
    |f| {
        let (key, own) = (std_key(STYLE)?, scope_of("demo")?);
        let s = f.fresh();
        let id = got(make_profile(&*s, "Arbeit"), &key, 5)?;
        ok(
            s.write(
                &Scope::General,
                Place::Profile,
                0,
                &key,
                Some(&text("base")),
            ),
            &key,
            2,
        )?;
        ok(
            s.write(
                &Scope::General,
                Place::Profile,
                id,
                &key,
                Some(&text("named")),
            ),
            &key,
            3,
        )?;
        ok(
            s.write(&own, Place::Profile, id, &key, Some(&text("named"))),
            &key,
            4,
        )?;
        ok(drop_profile(&*s, "Arbeit"), &key, 5)?;
        ok(s.clear(&Scope::General, id), &key, 6)?;
        ok(s.clear(&own, id), &key, 7)?;
        same(
            &value(&*s, &Scope::General, Place::Profile, id, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            8,
        )?;
        same(
            &value(&*s, &own, Place::Profile, id, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            9,
        )?;
        same(
            &value(&*s, &Scope::General, Place::Profile, 0, &key),
            &Some(text("base")),
            SuiteCode::WrongValue,
            &key,
            10,
        )
    }
);

store_rule!(
    /// A program's part is its own: another program and the general part do not see it, and the other way round.
    ScopeIsolated,
    "store-scope-isolated",
    |f| {
        let (key, a, b) = (std_key(STYLE)?, scope_of("demo")?, scope_of("other")?);
        let s = f.fresh();
        ok(
            s.write(&a, Place::Profile, 0, &key, Some(&text("craft"))),
            &key,
            1,
        )?;
        same(
            &value(&*s, &b, Place::Profile, 0, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            2,
        )?;
        same(
            &value(&*s, &Scope::General, Place::Profile, 0, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            3,
        )?;
        ok(
            s.write(
                &Scope::General,
                Place::Profile,
                0,
                &key,
                Some(&text("calm")),
            ),
            &key,
            4,
        )?;
        same(
            &value(&*s, &a, Place::Profile, 0, &key),
            &Some(text("craft")),
            SuiteCode::WrongValue,
            &key,
            5,
        )?;
        same(
            &value(&*s, &b, Place::Profile, 0, &key),
            &None,
            SuiteCode::WrongValue,
            &key,
            6,
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

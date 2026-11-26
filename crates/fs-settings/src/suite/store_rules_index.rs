//! The rules of the suite `store` about the stamp and the profile index: a change is seen, reading moves nothing, an index change is one step, ids are never reused.
//!
//! Pattern: one rule per behaviour (macro `store_rule!`).

use super::check::{fail, ok, refused, refused_as, std_key};
use super::codes::SuiteCode;
use super::store_check::{drop_profile, got, make_profile, refuse, same, store_rule, text};
use crate::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::profile_name::{ProfileName, ProfileRef};
use crate::standard::STYLE;

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

//! The rules of the suite `store` about values: what is written comes back, is removed, stays in its scope, goes with its profile.
//!
//! Pattern: one rule per behaviour (macro `store_rule!`).

use super::check::{ok, std_key};
use super::codes::SuiteCode;
use super::store_check::{
    drop_profile, got, make_profile, same, scope_of, store_rule, text, value,
};
use crate::declare::Place;
use crate::layer::Scope;
use crate::standard::STYLE;
use crate::value::Raw;

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

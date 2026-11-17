//! Helpers of the store rules: the rule macro, standard keys and scopes, the way a program makes and deletes a profile.
//!
//! Pattern: Specification (small predicates) + a macro that makes a rule of a body.

use fs_conformance::Coded;

use super::check::Checked;
use super::codes::{SuiteCode, keys};
use crate::codes::SettingCode;
use crate::declare::Place;
use crate::layer::Scope;
use crate::names::{ProgramName, SettingKey};
use crate::profile_name::{ProfileCode, ProfileName, ProfileRef};
use crate::store::SettingsStore;
use crate::value::Raw;

/// A rule over a store: `store_rule!(Type, "id", |fixture| { ...; Ok(()) })`. The body is a [`Checked`].
macro_rules! store_rule {
    ($(#[$doc:meta])* $t:ident, $id:literal, |$f:ident| $body:block) => {
        $(#[$doc])*
        pub(crate) struct $t;

        #[async_trait::async_trait(?Send)]
        impl<F: $crate::suite::StoreFixture + ?Sized> fs_conformance::Rule<F> for $t {
            fn id(&self) -> fs_conformance::RuleId {
                fs_conformance::RuleId::new($id)
            }

            async fn check(&self, $f: &mut F, _cx: &fs_conformance::Cx) -> fs_conformance::Verdict {
                fn body<F: $crate::suite::StoreFixture + ?Sized>($f: &mut F) -> $crate::suite::check::Checked $body
                match body($f) {
                    Ok(()) => fs_conformance::Verdict::Pass,
                    Err(code) => fs_conformance::Verdict::Fail(code.erase()),
                }
            }
        }
    };
}
pub(crate) use store_rule;

/// The scope of the program `name`.
pub(crate) fn scope_of(name: &str) -> Result<Scope, Coded<SuiteCode>> {
    ProgramName::new(name)
        .map(Scope::Program)
        .map_err(|_| Coded::new(SuiteCode::StandardMissing).with_text(keys::KEY, name))
}

/// The value of `key` in the profile `id` of `scope` at `place`.
pub(crate) fn value(
    s: &dyn SettingsStore,
    scope: &Scope,
    place: Place,
    id: u64,
    key: &SettingKey,
) -> Option<Raw> {
    s.profile(scope, place, id).values.get(key).cloned()
}

/// A step is wrong when `got != want`.
pub(crate) fn same<T: PartialEq>(got: &T, want: &T, code: SuiteCode, key: &SettingKey, step: u64) -> Checked {
    if got == want {
        Ok(())
    } else {
        Err(super::check::fail(code, key, step))
    }
}

/// Makes the profile `name` as `Settings::create_profile` does (the id above the base), and returns its id.
pub(crate) fn make_profile(
    s: &dyn SettingsStore,
    name: &str,
) -> Result<u64, Coded<SettingCode>> {
    let n = ProfileName::new(name).map_err(|p| crate::settings_profiles::mapped(&p))?;
    let index = s.update_index(&mut |i| {
        let next = i.get(&ProfileRef::Base).copied().unwrap_or(0) + 1;
        i.create(n.clone(), None)?;
        if let Some(id) = i.get_mut(&ProfileRef::Named(n.clone())) {
            *id = next;
        }
        if let Some(top) = i.get_mut(&ProfileRef::Base) {
            *top = next;
        }
        Ok(())
    })?;
    index
        .get(&ProfileRef::Named(n))
        .copied()
        .ok_or_else(|| Coded::new(SettingCode::Profile).with_text(crate::codes::keys::CAUSE, "unknown"))
}

/// Deletes the profile `name` (the index only; the caller clears the values).
pub(crate) fn drop_profile(s: &dyn SettingsStore, name: &str) -> Result<(), Coded<SettingCode>> {
    let n = ProfileName::new(name).map_err(|p| crate::settings_profiles::mapped(&p))?;
    s.update_index(&mut |i| i.delete(&n).map(|_| ())).map(|_| ())
}

/// The failure of a closure that must be refused: nothing in particular, only an error.
pub(crate) fn refuse() -> Coded<ProfileCode> {
    Coded::new(ProfileCode::Unknown).with_text(crate::profile_name::NAME, "refused")
}

/// A setup step of a rule that must go through: its code becomes `refused` with the code's name.
pub(crate) fn got<T>(
    r: Result<T, Coded<SettingCode>>,
    key: &SettingKey,
    step: u64,
) -> Result<T, Coded<SuiteCode>> {
    use crate::coded_stub::ErrorCode;
    r.map_err(|e| {
        super::check::fail(SuiteCode::Refused, key, step).with_text(keys::CODE, e.code().as_str())
    })
}

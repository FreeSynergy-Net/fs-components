//! Rules over named profiles (Nachzug 17.11., answer 16): switching live, inheritance, deleting the active profile,
//! the lock in every profile, unique names.
//!
//! Pattern: one rule per behaviour (macro `rule!`).

use std::sync::{Arc, Mutex, PoisonError};

use super::check::{expect, fail, ok, refused, rule, std_key, two_others};
use super::codes::{SuiteCode, keys};
use crate::layer::{Layer, Target};
use crate::profile_name::ProfileRef;
use crate::settings::{Change, OnChange};
use crate::standard::STYLE;

#[derive(Default)]
struct Told(Mutex<Vec<Change>>);

impl OnChange for Told {
    fn changed(&self, change: &Change) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(change.clone());
    }
}

rule!(
    /// Switching the profile changes the values live, each changed key told once.
    ProfileSwitchLive, "settings.profile-switch-live", |f| {
        let key = std_key(STYLE)?;
        let (a, _) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        let base = f.program().get(&key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        ok(f.program().create_profile("Vortrag"), &key, 1)?;
        ok(f.program().switch_profile(Some("Vortrag")), &key, 2)?;
        ok(f.program().save(&key, Some(&a), Target::ThisProgram), &key, 3)?;
        ok(f.program().switch_profile(None), &key, 4)?;
        expect(f.program(), &key, &base.value, base.from, false, 5)?;
        let told = Arc::new(Told::default());
        f.program().observe(told.clone());
        ok(f.program().switch_profile(Some("vortrag")), &key, 6)?;
        let n = told.0.lock().unwrap_or_else(PoisonError::into_inner).iter().filter(|c| c.key == key).count();
        if n != 1 {
            return Err(fail(SuiteCode::NotTold, &key, 7).with_number(keys::TOLD, n as u64));
        }
        expect(f.program(), &key, &a, Layer::NamedProgram, false, 8)
    }
);

rule!(
    /// A named profile inherits what it lacks from the base; its own values leave the base untouched.
    ProfileInherits, "settings.profile-inherits", |f| {
        let key = std_key(STYLE)?;
        let (a, b) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        ok(f.program().save(&key, Some(&a), Target::AllPrograms), &key, 1)?;
        ok(f.program().create_profile("Arbeit"), &key, 2)?;
        ok(f.program().switch_profile(Some("Arbeit")), &key, 3)?;
        expect(f.program(), &key, &a, Layer::Person, false, 4)?;
        ok(f.program().save(&key, Some(&b), Target::AllPrograms), &key, 5)?;
        f.other().refresh();
        expect(f.program(), &key, &b, Layer::NamedPerson, false, 6)?;
        expect(f.other(), &key, &b, Layer::NamedPerson, false, 7)?;
        ok(f.program().switch_profile(None), &key, 8)?;
        expect(f.program(), &key, &a, Layer::Person, false, 9)
    }
);

rule!(
    /// Deleting the active profile makes the base active, live; a new profile of the same name starts empty.
    ProfileDeleteActive, "settings.profile-delete-active", |f| {
        let key = std_key(STYLE)?;
        let (a, _) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        let base = f.program().get(&key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        ok(f.program().create_profile("Vortrag"), &key, 1)?;
        ok(f.program().switch_profile(Some("Vortrag")), &key, 2)?;
        ok(f.program().save(&key, Some(&a), Target::ThisProgram), &key, 3)?;
        let changes = f.program().delete_profile("Vortrag").map_err(|_| fail(SuiteCode::Refused, &key, 4))?;
        if !changes.iter().any(|c| c.key == key) || *f.program().profiles().active() != ProfileRef::Base {
            return Err(fail(SuiteCode::NotTold, &key, 5));
        }
        expect(f.program(), &key, &base.value, base.from, false, 6)?;
        ok(f.program().create_profile("Vortrag"), &key, 7)?;
        ok(f.program().switch_profile(Some("Vortrag")), &key, 8)?;
        expect(f.program(), &key, &base.value, base.from, false, 9)
    }
);

rule!(
    /// A lock holds in every profile: the named profile's own value does not count, saving there is refused.
    LockAllProfiles, "settings.lock-all-profiles", |f| {
        let key = std_key(STYLE)?;
        let (a, b) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        ok(f.program().create_profile("Arbeit"), &key, 1)?;
        ok(f.program().switch_profile(Some("Arbeit")), &key, 2)?;
        ok(f.program().save(&key, Some(&b), Target::ThisProgram), &key, 3)?;
        ok(f.admin(true).lock(&key, &a, "reason"), &key, 4)?;
        f.program().refresh();
        expect(f.program(), &key, &a, Layer::Instance, true, 5)?;
        refused(f.program().save(&key, Some(&b), Target::AllPrograms), &key, 6)?;
        ok(f.program().switch_profile(None), &key, 7)?;
        expect(f.program(), &key, &a, Layer::Instance, true, 8)
    }
);

rule!(
    /// Profile names are unique ignoring case; an empty name is refused.
    ProfileNameUnique, "settings.profile-name-unique", |f| {
        let key = std_key(STYLE)?;
        ok(f.program().create_profile("Arbeit"), &key, 1)?;
        refused(f.program().create_profile("arbeit"), &key, 2)?;
        refused(f.program().create_profile("  "), &key, 3)?;
        ok(f.program().create_profile("Vortrag"), &key, 4)?;
        refused(f.program().rename_profile("Vortrag", "ARBEIT"), &key, 5)
    }
);

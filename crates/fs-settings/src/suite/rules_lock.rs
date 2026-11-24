//! Rules over the lock, the lock button and the frontend choice.
//!
//! Pattern: one rule per behaviour (macro `rule!`).

use fs_conformance::Coded;

use super::check::{expect, fail, ok, refused, rule, std_key, two_others};
use super::codes::SuiteCode;
use crate::layer::{Layer, Target};
use crate::standard::{FRONTEND, SCHEME, STYLE};

rule!(
    /// A lock beats every layer, the program profile too; after the unlock the person's own value counts again.
    LockWins, "settings.lock-wins", |f| {
        let key = std_key(STYLE)?;
        let (a, b) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        ok(f.program().save(&key, Some(&b), Target::ThisProgram), &key, 1)?;
        let admin = f.admin(true);
        ok(admin.lock(&key, &a, "reason"), &key, 2)?;
        f.program().refresh();
        expect(f.program(), &key, &a, Layer::Instance, true, 3)?;
        ok(admin.unlock(&key, "done"), &key, 4)?;
        f.program().refresh();
        expect(f.program(), &key, &b, Layer::Program, false, 5)
    }
);

rule!(
    /// Locked = nobody changes it, the locker included; without the right nobody locks or unlocks.
    LockHolds, "settings.lock-holds", |f| {
        let key = std_key(STYLE)?;
        let (a, b) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        let admin = f.admin(true);
        let stranger = f.admin(false);
        ok(admin.lock(&key, &a, "reason"), &key, 1)?;
        f.program().refresh();
        refused(admin.set_default(&key, Some(&b)), &key, 2)?;
        refused(admin.lock(&key, &b, "again"), &key, 3)?;
        refused(f.program().save(&key, Some(&b), Target::ThisProgram), &key, 4)?;
        refused(f.program().save(&key, Some(&b), Target::AllPrograms), &key, 5)?;
        refused(stranger.unlock(&key, "done"), &key, 6)?;
        refused(stranger.lock(&std_key(SCHEME)?, &crate::Value::Choice("dark".to_owned()), "x"), &key, 7)?;
        ok(admin.unlock(&key, "done"), &key, 8)?;
        f.program().refresh();
        ok(f.program().save(&key, Some(&b), Target::ThisProgram), &key, 9)
    }
);

rule!(
    /// The lock button exists only with the right; the locked state is shown to everyone.
    LockButton, "settings.lock-button", |f| {
        let key = std_key(STYLE)?;
        let (a, _) = two_others(f.program(), &key).ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        let none = crate::NoLockRights;
        let admin = f.admin(true);
        if f.lock_view(&none, &key).button || !f.lock_view(admin.rights.as_ref(), &key).button {
            return Err(fail(SuiteCode::ButtonWrong, &key, 1));
        }
        ok(admin.lock(&key, &a, "reason"), &key, 2)?;
        f.program().refresh();
        let seen = f.lock_view(&none, &key);
        if seen.button || seen.locked.is_none() {
            return Err(fail(SuiteCode::LockWrong, &key, 3));
        }
        Ok(())
    }
);

rule!(
    /// The starter gets the person's frontend choice only: none at first, the saved one, none again after reset.
    FrontendChoice, "settings.frontend-choice", |f| {
        let key = std_key(FRONTEND)?;
        let step = |n: u64| Coded::new(SuiteCode::ChoiceWrong).with_number(super::codes::keys::STEP, n);
        if f.frontend_choice().is_some() {
            return Err(step(1));
        }
        let Some((a, _)) = two_others(f.program(), &key) else { return Ok(()) };
        ok(f.program().save(&key, Some(&a), Target::ThisProgram), &key, 2)?;
        if f.frontend_choice() != Some(match &a { crate::Value::Choice(id) => id.clone(), _ => String::new() }) {
            return Err(step(3));
        }
        ok(f.program().save(&key, None, Target::ThisProgram), &key, 4)?;
        if f.frontend_choice().is_some() { Err(step(5)) } else { Ok(()) }
    }
);

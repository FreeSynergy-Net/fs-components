//! Rules over declarations, layers, the two save buttons and live changes.
//!
//! Pattern: one rule per behaviour (macro `rule!`); every rule uses standard keys only, so it holds for every program.

use std::sync::{Arc, Mutex, PoisonError};

use fs_conformance::Coded;

use super::check::{expect, fail, ok, rule, std_key, two_others};
use super::codes::{SuiteCode, keys};
use crate::declare::Catalog;
use crate::layer::{Layer, Scope, Target};
use crate::names::SettingKey;
use crate::settings::{Change, OnChange};
use crate::standard::{FRONTEND, LANGUAGE, SCHEME, STYLE};
use crate::value::Raw;

fn values<F: super::SettingsFixture + ?Sized>(
    f: &F,
    key: &SettingKey,
) -> Result<(crate::Value, crate::Value), Coded<SuiteCode>> {
    two_others(f.program(), key).ok_or_else(|| fail(SuiteCode::WrongValue, key, 0))
}

rule!(
    /// The program's declarations form a catalog: no key twice, every default fits its kind.
    CatalogValid,
    "settings.catalog-valid",
    |f| {
        let refused = |e: fs_types::coded::Coded<crate::SettingCode>| {
            use fs_types::coded::ErrorCode;
            Coded::new(SuiteCode::CatalogRefused).with_text(keys::CODE, e.code().as_str())
        };
        Catalog::of(f.declarations().map_err(refused)?)
            .map(|_| ())
            .map_err(refused)
    }
);

rule!(
    /// Every declaration has its name and description in German and English (ADR 0057).
    TextsBoth,
    "settings.texts-both",
    |f| {
        let decls = f
            .declarations()
            .map_err(|_| Coded::new(SuiteCode::CatalogRefused))?;
        for d in &decls {
            for language in ["de", "en"] {
                for key in [d.label_key(), d.about_key()] {
                    if !f.has_text(language, &key) {
                        return Err(Coded::new(SuiteCode::TextMissing)
                            .with_text(keys::KEY, key)
                            .with_text(keys::LANGUAGE, language));
                    }
                }
            }
        }
        Ok(())
    }
);

rule!(
    /// Language, style, light/dark and the frontend choice are declared (first use: live in every program).
    StandardKeys,
    "settings.standard-keys",
    |f| {
        let decls = f
            .declarations()
            .map_err(|_| Coded::new(SuiteCode::CatalogRefused))?;
        for which in [LANGUAGE, STYLE, SCHEME, FRONTEND] {
            let key = std_key(which)?;
            if !decls.iter().any(|d| d.key == key) {
                return Err(
                    Coded::new(SuiteCode::StandardMissing).with_text(keys::KEY, key.as_str())
                );
            }
        }
        Ok(())
    }
);

rule!(
    /// The program profile beats the general profile; another program sees only the general one.
    ProgramWins,
    "settings.program-wins",
    |f| {
        let key = std_key(STYLE)?;
        let (a, b) = values(f, &key)?;
        ok(
            f.program().save(&key, Some(&a), Target::AllPrograms),
            &key,
            1,
        )?;
        ok(
            f.program().save(&key, Some(&b), Target::ThisProgram),
            &key,
            2,
        )?;
        f.other().refresh();
        expect(f.program(), &key, &b, Layer::Program, false, 1)?;
        expect(f.other(), &key, &a, Layer::Person, false, 2)
    }
);

rule!(
    /// "For this program" changes this program only.
    ThisProgramOnly,
    "settings.this-program-only",
    |f| {
        let key = std_key(STYLE)?;
        let (a, _) = values(f, &key)?;
        let before = f
            .other()
            .get(&key)
            .ok_or_else(|| fail(SuiteCode::WrongValue, &key, 0))?;
        ok(
            f.program().save(&key, Some(&a), Target::ThisProgram),
            &key,
            1,
        )?;
        f.other().refresh();
        expect(f.program(), &key, &a, Layer::Program, false, 2)?;
        expect(f.other(), &key, &before.value, before.from, false, 3)
    }
);

rule!(
    /// "For all" sets the general profile, this program follows it, another program's own value stays.
    AllFollows,
    "settings.all-follows",
    |f| {
        let key = std_key(STYLE)?;
        let (a, b) = values(f, &key)?;
        ok(f.other().save(&key, Some(&b), Target::ThisProgram), &key, 1)?;
        ok(
            f.program().save(&key, Some(&b), Target::ThisProgram),
            &key,
            2,
        )?;
        ok(
            f.program().save(&key, Some(&a), Target::AllPrograms),
            &key,
            3,
        )?;
        f.other().refresh();
        expect(f.program(), &key, &a, Layer::Person, false, 4)?;
        expect(f.other(), &key, &b, Layer::Program, false, 5)
    }
);

#[derive(Default)]
struct Count(Mutex<Vec<Change>>);

impl OnChange for Count {
    fn changed(&self, change: &Change) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(change.clone());
    }
}

rule!(
    /// A change by another program arrives live, exactly once, without a restart.
    Live,
    "settings.live",
    |f| {
        let key = std_key(STYLE)?;
        let (a, _) = values(f, &key)?;
        let count = Arc::new(Count::default());
        f.program().observe(count.clone());
        ok(f.other().save(&key, Some(&a), Target::AllPrograms), &key, 1)?;
        f.program().refresh();
        f.program().refresh();
        let told = count
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .filter(|c| c.key == key && c.after.value == a)
            .count();
        if told != 1 {
            return Err(fail(SuiteCode::NotTold, &key, 2).with_number(keys::TOLD, told as u64));
        }
        expect(f.program(), &key, &a, Layer::Person, false, 3)
    }
);

rule!(
    /// A stored value of the wrong type is a visible problem; a lower layer is used.
    BadValueVisible,
    "settings.bad-value-visible",
    |f| {
        let key = std_key(LANGUAGE)?;
        f.put(&Scope::General, &key, Raw::Int(5));
        f.program().refresh();
        let seen = f
            .program()
            .problems()
            .iter()
            .any(|p| p.code() == crate::SettingCode::WrongType);
        let from = f.program().get(&key).map(|e| e.from);
        if !seen || from == Some(Layer::Person) {
            return Err(Coded::new(SuiteCode::ProblemMissing).with_text(keys::KEY, key.as_str()));
        }
        Ok(())
    }
);

rule!(
    /// Reset ("follow the layer below") removes the program's own value.
    ResetFollows,
    "settings.reset-follows",
    |f| {
        let key = std_key(STYLE)?;
        let (a, b) = values(f, &key)?;
        f.put(&Scope::General, &key, a.to_raw());
        ok(
            f.program().save(&key, Some(&b), Target::ThisProgram),
            &key,
            1,
        )?;
        ok(f.program().save(&key, None, Target::ThisProgram), &key, 2)?;
        expect(f.program(), &key, &a, Layer::Person, false, 3)
    }
);

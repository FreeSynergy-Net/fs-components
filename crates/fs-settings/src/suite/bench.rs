//! The standard fixture: a program's declarations and texts, two programs (`demo`, `other`) over one faulty memory
//! store, and one switch per mutant.
//!
//! Pattern: Test double (fake store) + switch on the fake (the tasks form `mutation.rs:8`).

use std::sync::{Arc, PoisonError};

use crate::coded_stub::Coded;

use super::faulty::{Faulty, StoreFault};
use super::fixture::SettingsFixture;
use crate::codes::SettingCode;
use crate::declare::{Catalog, Declaration, Declares};
use crate::layer::Scope;
use crate::lock::{Admin, LockRights, LockView, lock_view};
use crate::names::{GroupId, Owner, OwnerKind, ProgramName, SettingKey};
use crate::settings::Settings;
use crate::standard::{FRONTEND, ShellSettings, person_choice};
use crate::store::NoSystem;
use crate::value::{Kind, Raw, Value};

/// Which MF2 texts a program has: `(language, key) -> present`.
pub type Texts = Box<dyn Fn(&str, &str) -> bool>;

/// One wrong behaviour of the bench (ids in `mutants.rs`).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BenchMutation {
    /// A key declared twice.
    DuplicateKey,
    /// A default outside its kind.
    BadDefault,
    /// The German texts are missing.
    TextMissing,
    /// `fs.shell.frontend` is not declared.
    StandardMissing,
    /// Program profile writes land in the general profile.
    ScopeFlattened,
    /// General writes also land in `other`'s profile.
    WritesEverywhere,
    /// Locks are not shown.
    LockDropped,
    /// The lock button shows for everyone.
    ButtonAlways,
    /// No change is ever seen.
    StampFrozen,
    /// Bad values vanish without a word.
    DropsBad,
    /// Reset does nothing.
    RemoveIgnored,
    /// The default frontend is handed over as the person's choice.
    ChoiceFromDefault,
    /// The profile index is never kept.
    IndexIgnored,
    /// A named profile does not inherit from the base.
    NamedIsolated,
}

struct Rights(bool);

impl LockRights for Rights {
    fn can_lock(&self, _key: &SettingKey) -> bool {
        self.0
    }
}

/// The bench.
pub struct Bench {
    decls: Vec<Declaration>,
    texts: Texts,
    store: Faulty,
    program: Settings,
    other: Settings,
    mutation: Option<BenchMutation>,
}

fn open(name: &str, decls: &[Declaration], store: &Faulty) -> Result<Settings, Coded<SettingCode>> {
    let catalog = Catalog::of(decls.to_vec())?;
    Ok(Settings::open(
        ProgramName::new(name)?,
        catalog,
        Arc::new(store.clone()),
        Arc::new(NoSystem),
    ))
}

impl Bench {
    /// For the program's declarations (they must form a catalog) and its texts (`(language, key) -> present`).
    ///
    /// # Errors
    /// The catalog's code when the declarations do not form one.
    pub fn new(decls: Vec<Declaration>, texts: Texts) -> Result<Self, Coded<SettingCode>> {
        let store = Faulty::default();
        let program = open("demo", &decls, &store)?;
        let other = open("other", &decls, &store)?;
        Ok(Self {
            decls,
            texts,
            store,
            program,
            other,
            mutation: None,
        })
    }

    /// The demo program: the shell's settings (styles `simple`, `craft`; frontends `simple`, `compact`) and one own
    /// setting `program.demo.week-start` (1 to 7, default 1); every text present.
    ///
    /// # Errors
    /// Never for these constants (a K-test proves it).
    pub fn demo() -> Result<Self, Coded<SettingCode>> {
        let s = |x: &str| x.to_owned();
        let mut decls = ShellSettings::new(
            vec![s("simple"), s("craft")],
            s("simple"),
            vec![s("simple"), s("compact")],
            s("simple"),
        )
        .declarations()?;
        let key = SettingKey::new(&Owner::new(OwnerKind::Program, "demo")?, "week-start")?;
        decls.push(Declaration::new(
            key,
            Kind::Whole { min: 1, max: 7 },
            Value::Whole(1),
            GroupId::new("calendar")?,
        ));
        Self::new(decls, Box::new(|_, _| true))
    }

    /// Switches `m` on (for this bench, also after `reset`).
    pub fn set_mutation(&mut self, m: BenchMutation) {
        use BenchMutation as B;
        let fault = match m {
            B::ScopeFlattened => Some(StoreFault::ScopeFlattened),
            B::WritesEverywhere => Some(StoreFault::WritesEverywhere),
            B::LockDropped => Some(StoreFault::LockDropped),
            B::StampFrozen => Some(StoreFault::StampFrozen),
            B::DropsBad => Some(StoreFault::DropsBad),
            B::RemoveIgnored => Some(StoreFault::RemoveIgnored),
            B::IndexIgnored => Some(StoreFault::IndexIgnored),
            B::NamedIsolated => Some(StoreFault::NamedIsolated),
            _ => None,
        };
        *self
            .store
            .fault
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = fault;
        if m == B::LockDropped {
            // the store forgets locks everywhere: also its own check at writing
            self.store.inner.set_lax("lock");
        }
        self.mutation = Some(m);
    }

    fn is(&self, m: BenchMutation) -> bool {
        self.mutation == Some(m)
    }
}

impl SettingsFixture for Bench {
    fn reset(&mut self) {
        let store = Faulty {
            inner: crate::memory::MemoryStore::new(),
            fault: self.store.fault.clone(),
        };
        if self.is(BenchMutation::LockDropped) {
            store.inner.set_lax("lock");
        }
        if let (Ok(p), Ok(o)) = (
            open("demo", &self.decls, &store),
            open("other", &self.decls, &store),
        ) {
            (self.store, self.program, self.other) = (store, p, o);
        }
    }

    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>> {
        let mut d = self.decls.clone();
        if self.is(BenchMutation::DuplicateKey)
            && let Some(first) = d.first().cloned()
        {
            d.push(first);
        }
        if self.is(BenchMutation::BadDefault) {
            let key = SettingKey::new(&Owner::new(OwnerKind::Program, "demo")?, "bad")?;
            d.push(Declaration::new(
                key,
                Kind::Whole { min: 0, max: 1 },
                Value::Whole(5),
                GroupId::new("calendar")?,
            ));
        }
        if self.is(BenchMutation::StandardMissing) {
            let frontend = ShellSettings::key(FRONTEND)?;
            d.retain(|x| x.key != frontend);
        }
        Ok(d)
    }

    fn has_text(&self, language: &str, key: &str) -> bool {
        !(self.is(BenchMutation::TextMissing) && language == "de") && (self.texts)(language, key)
    }

    fn program(&self) -> &Settings {
        &self.program
    }

    fn other(&self) -> &Settings {
        &self.other
    }

    fn admin(&self, rights: bool) -> Admin {
        let catalog = self.program.catalog().clone();
        Admin {
            store: Arc::new(self.store.clone()),
            rights: Arc::new(Rights(rights)),
            by: fs_types::BasicValue::parse("https://example.org/people/admin-test")
                .expect("a constant IRI"),
            catalog,
            clock: crate::system_clock,
        }
    }

    fn put(&mut self, scope: &Scope, key: &SettingKey, raw: Raw) {
        let _ = crate::store::SettingsStore::write(
            &self.store.inner,
            scope,
            crate::declare::Place::Profile,
            0,
            key,
            Some(&raw),
        );
    }

    fn lock_view(&self, rights: &dyn LockRights, key: &SettingKey) -> LockView {
        let view = match self.program.get(key) {
            Some(e) => lock_view(rights, key, &e),
            None => LockView {
                button: rights.can_lock(key),
                locked: None,
            },
        };
        if self.is(BenchMutation::ButtonAlways) {
            LockView {
                button: true,
                ..view
            }
        } else {
            view
        }
    }

    fn frontend_choice(&self) -> Option<String> {
        let e = self.program.get(&ShellSettings::key(FRONTEND).ok()?)?;
        match (&e.value, self.is(BenchMutation::ChoiceFromDefault)) {
            (Value::Choice(id), true) => Some(id.clone()),
            _ => person_choice(&e),
        }
    }
}

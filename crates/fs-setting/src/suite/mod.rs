//! The conformance suite `settings` (feature `conformance`; fs-conformance review condition C1: the suite lives in the
//! crate it checks; A4: the feature is in no `default`, fs-conformance never in a shipped program's normal graph).
//!
//! What it checks is a program's WIRING of settings: its declarations (with its blocks'), its texts in both languages,
//! the standard keys, the layers, the two save buttons, the lock and the lock button, live changes, the frontend
//! choice. Fixture: [`SettingsFixture`]; standard: [`Bench`] (two programs `demo` and `other` over one memory store, a
//! fault switch per mutant). Every rule is named by at least one mutant of [`bench_mutants`] (`fs_conformance::prove`).
//!
//! Pattern: Abstract factory (`SettingsSuite` makes the rules) + generic rules `impl<F: SettingsFixture + ?Sized> Rule<F>`
//! (review condition A1) + Decorator (`faulty.rs`, the mutants' store).

mod bench;
mod check;
mod codes;
mod faulty;
mod fixture;
mod group;
mod mutants;
mod rules_layers;
mod rules_lock;

pub use bench::{Bench, BenchMutation};
pub use codes::{SuiteCode, keys as suite_keys};
pub use fixture::SettingsFixture;
pub use group::SettingsSuite;
pub use mutants::{BenchMutant, bench_mutants};

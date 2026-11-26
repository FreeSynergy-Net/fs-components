//! The conformance suite `settings` (feature `conformance`; fs-conformance review condition C1: the suite lives in the
//! crate it checks; A4: the feature is in no `default`, fs-conformance never in a shipped program's normal graph).
//!
//! What it checks is a program's WIRING of settings: its declarations (with its blocks'), its texts in both languages,
//! the standard keys, the layers, the two save buttons, the lock and the lock button, live changes, the frontend
//! choice, the named profiles. Fixture: [`SettingsFixture`]; standard: [`Bench`] (two programs `demo` and `other` over one memory store, a
//! fault switch per mutant). Every rule is named by at least one mutant of [`bench_mutants`] (`fs_conformance::prove`).
//!
//! A second suite, `store` ([`StoreSuite`], 12 rules, fixture [`StoreFixture`], provider [`StoreBench`], 12 mutants), checks what
//! every settings store keeps, wherever it lives (memory, behind a socket, in a database).
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
mod rules_profiles;
mod store_bench;
mod store_check;
mod store_fixture;
mod store_group;
mod store_health;
mod store_mutants;
mod store_rules;
mod store_rules_index;
mod store_rules_lock;

pub use bench::{Bench, BenchMutation};
pub use codes::{SuiteCode, keys as suite_keys};
pub use fixture::SettingsFixture;
pub use group::SettingsSuite;
pub use mutants::{BenchMutant, bench_mutants};
pub use store_bench::{BenchStore, StoreBench, StoreFault};
pub use store_fixture::StoreFixture;
pub use store_group::StoreSuite;
pub use store_mutants::{StoreBenchMutant, store_bench_mutants};

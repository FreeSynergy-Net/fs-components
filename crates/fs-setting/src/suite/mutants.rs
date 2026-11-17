//! The bench mutants: one wrong behaviour each, and the rules that must catch it. `fs_conformance::prove` runs each on a
//! fresh bench; every rule of the suite is named by at least one.
//!
//! Pattern: Strategy (`Mutant<Bench>`; a switch on the fake).

use fs_conformance::{Applied, Mutant, MutantId, RuleId};

use super::bench::{Bench, BenchMutation};

/// A switch of the bench with its id and the rules it must break.
#[derive(Clone, Copy, Debug)]
pub struct BenchMutant {
    id: &'static str,
    mutation: BenchMutation,
    must_fail: &'static [&'static str],
}

impl Mutant<Bench> for BenchMutant {
    fn id(&self) -> MutantId {
        MutantId::new(self.id)
    }

    fn must_fail(&self) -> Vec<RuleId> {
        self.must_fail.iter().copied().map(RuleId::new).collect()
    }

    fn apply(&self, bench: &mut Bench) -> Applied {
        bench.set_mutation(self.mutation);
        Applied::Yes
    }
}

/// The fourteen mutants (ids `settings-bench.<name>`, well formed and unique: review condition A3).
#[must_use]
pub fn bench_mutants() -> Vec<Box<dyn Mutant<Bench>>> {
    use BenchMutation as M;
    let m = |id: &'static str, mutation: M, must_fail: &'static [&'static str]| BenchMutant {
        id,
        mutation,
        must_fail,
    };
    let all: [BenchMutant; 14] = [
        m(
            "settings-bench.duplicate-key",
            M::DuplicateKey,
            &["settings.catalog-valid"],
        ),
        m(
            "settings-bench.bad-default",
            M::BadDefault,
            &["settings.catalog-valid"],
        ),
        m(
            "settings-bench.text-missing",
            M::TextMissing,
            &["settings.texts-both"],
        ),
        m(
            "settings-bench.standard-missing",
            M::StandardMissing,
            &["settings.standard-keys"],
        ),
        m(
            "settings-bench.scope-flattened",
            M::ScopeFlattened,
            &["settings.program-wins", "settings.this-program-only"],
        ),
        m(
            "settings-bench.writes-everywhere",
            M::WritesEverywhere,
            &["settings.all-follows"],
        ),
        m(
            "settings-bench.lock-dropped",
            M::LockDropped,
            &[
                "settings.lock-wins",
                "settings.lock-holds",
                "settings.lock-all-profiles",
            ],
        ),
        m(
            "settings-bench.button-always",
            M::ButtonAlways,
            &["settings.lock-button"],
        ),
        m(
            "settings-bench.stamp-frozen",
            M::StampFrozen,
            &["settings.live"],
        ),
        m(
            "settings-bench.drops-bad",
            M::DropsBad,
            &["settings.bad-value-visible"],
        ),
        m(
            "settings-bench.remove-ignored",
            M::RemoveIgnored,
            &["settings.reset-follows", "settings.frontend-choice"],
        ),
        m(
            "settings-bench.choice-from-default",
            M::ChoiceFromDefault,
            &["settings.frontend-choice"],
        ),
        m(
            "settings-bench.index-ignored",
            M::IndexIgnored,
            &[
                "settings.profile-switch-live",
                "settings.profile-delete-active",
                "settings.profile-name-unique",
            ],
        ),
        m(
            "settings-bench.named-isolated",
            M::NamedIsolated,
            &["settings.profile-inherits"],
        ),
    ];
    all.into_iter()
        .map(|b| Box::new(b) as Box<dyn Mutant<Bench>>)
        .collect()
}

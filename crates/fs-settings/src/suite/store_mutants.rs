//! The store bench mutants: one wrong behaviour each, and the rule that must catch it (`fs_conformance::prove`).
//!
//! Pattern: Strategy (`Mutant<StoreBench>`; a switch on the fake).

use fs_conformance::{Applied, Mutant, MutantId, RuleId};

use super::store_bench::{StoreBench, StoreFault};

/// A switch of the store bench with its id and the rule it must break.
#[derive(Clone, Copy, Debug)]
pub struct StoreBenchMutant {
    id: &'static str,
    fault: StoreFault,
    must_fail: &'static str,
}

impl Mutant<StoreBench> for StoreBenchMutant {
    fn id(&self) -> MutantId {
        MutantId::new(self.id)
    }

    fn must_fail(&self) -> Vec<RuleId> {
        vec![RuleId::new(self.must_fail)]
    }

    fn apply(&self, bench: &mut StoreBench) -> Applied {
        bench.set_fault(self.fault);
        Applied::Yes
    }
}

/// The twelve mutants, one per rule (ids `store-bench.<name>`).
#[must_use]
pub fn store_bench_mutants() -> Vec<Box<dyn Mutant<StoreBench>>> {
    use StoreFault as F;
    let m = |id, fault, must_fail| StoreBenchMutant {
        id,
        fault,
        must_fail,
    };
    let all = [
        m(
            "store-bench.mangled-kind",
            F::MangledKind,
            "store-roundtrip",
        ),
        m(
            "store-bench.remove-ignored",
            F::RemoveIgnored,
            "store-remove",
        ),
        m(
            "store-bench.stamp-frozen",
            F::StampFrozen,
            "store-stamp-moves",
        ),
        m(
            "store-bench.stamp-jitter",
            F::StampJitter,
            "store-stamp-still",
        ),
        m(
            "store-bench.lock-ignored",
            F::LockIgnored,
            "store-locked-refused",
        ),
        m("store-bench.index-half", F::IndexHalf, "store-index-atomic"),
        m(
            "store-bench.id-reused",
            F::IdReused,
            "store-ids-never-reused",
        ),
        m(
            "store-bench.clear-ignored",
            F::ClearIgnored,
            "store-delete-clears",
        ),
        m(
            "store-bench.scope-shared",
            F::ScopeShared,
            "store-scope-isolated",
        ),
        m(
            "store-bench.health-hidden",
            F::HealthHidden,
            "store-health-said",
        ),
        m("store-bench.log-dup", F::LogDup, "instance-log-append"),
        m(
            "store-bench.reason-unchecked",
            F::ReasonUnchecked,
            "instance-unlock-reason",
        ),
    ];
    all.into_iter()
        .map(|b| Box::new(b) as Box<dyn Mutant<StoreBench>>)
        .collect()
}

//! KB7: the suite `store` on the store bench: 12 rules conformant; every one of the 12 bench mutants is caught by its rule.
#![cfg(feature = "conformance")]

use fs_conformance::{Need, Options, Report, RuleId, Suite, prove, run};
use fs_settings::suite::{StoreBench, StoreFixture, StoreSuite, store_bench_mutants};

fn rules_of(r: &Report) -> Vec<String> {
    r.failures
        .iter()
        .map(|f| f.rule.as_str().to_owned())
        .collect()
}

#[tokio::test]
async fn kb7_store_bench_is_conformant() {
    let mut b = StoreBench::new();
    let r = run(&StoreSuite, &mut b, &Options::default()).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
    assert_eq!((r.checked, r.skipped.len()), (12, 0));
}

#[tokio::test]
async fn kb7_prove_catches_all_twelve() {
    let fresh = || Box::new(StoreBench::new());
    let mutants = store_bench_mutants();
    assert_eq!(mutants.len(), 12);
    let r = prove(&StoreSuite, &fresh, &mutants, &Options::default()).await;
    assert!(r.is_proven(), "{:?}", r.failures);
    assert_eq!(r.caught.len(), 12);
}

/// Evidence per mutant: each one turns red exactly the rule it names (printed with `--nocapture`).
#[tokio::test]
async fn kb7_each_mutant_is_red() {
    for m in store_bench_mutants() {
        let mut b = StoreBench::new();
        m.apply(&mut b);
        let r = run(&StoreSuite, &mut b, &Options::default()).await;
        let red = rules_of(&r);
        println!(
            "STORE-MUTANT {} -> red: {}",
            m.id().as_str(),
            red.join(", ")
        );
        for need in m.must_fail() {
            assert!(
                red.iter().any(|x| x == need.as_str()),
                "{} did not turn {} red",
                m.id().as_str(),
                need.as_str()
            );
        }
    }
}

/// A provider that cannot break its storage skips the health rule (listed, never passed).
struct NoBreak(StoreBench);

impl StoreFixture for NoBreak {
    fn fresh(&mut self) -> std::sync::Arc<dyn fs_settings::InstanceStore> {
        self.0.fresh()
    }

    fn break_store(&mut self, _store: &dyn fs_settings::InstanceStore) -> bool {
        false
    }
}

#[tokio::test]
async fn kb7_health_rule_is_skipped_not_passed() {
    let mut f = NoBreak(StoreBench::new());
    let o = Options {
        partial: true,
        ..Options::default()
    };
    let r = run(&StoreSuite, &mut f, &o).await;
    assert!(r.is_conformant(), "{:?}", r.failures);
    assert_eq!(r.checked, 11);
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].rule, RuleId::new("store-health-said"));
    assert_eq!(r.skipped[0].need, Need::new("fixture.break-store"));
    // strict: the missing capability is a failure
    let strict = run(&StoreSuite, &mut f, &Options::default()).await;
    assert!(!strict.is_conformant());
    let _: &dyn Suite<NoBreak> = &StoreSuite;
}

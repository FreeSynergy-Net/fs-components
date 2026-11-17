//! The suite `store`: its id and its twelve rules in order.
//!
//! Pattern: Abstract factory (makes the rules).

use fs_conformance::{Rule, Suite, SuiteId};

/// The suite `store`: the behaviour every settings store keeps, whether it lives in memory, behind a socket or in a
/// database (run by `fs-settings-rpc` over the client, by the program `fs-settings` over the database).
#[derive(Clone, Copy, Debug, Default)]
pub struct StoreSuite;

impl<F: super::StoreFixture + ?Sized> Suite<F> for StoreSuite {
    fn id(&self) -> SuiteId {
        SuiteId::new("store")
    }

    fn rules(&self) -> Vec<Box<dyn Rule<F>>> {
        use super::store_rules as r;
        vec![
            Box::new(r::Roundtrip),
            Box::new(r::Remove),
            Box::new(r::StampMoves),
            Box::new(r::StampStill),
            Box::new(r::LockedRefused),
            Box::new(r::IndexAtomic),
            Box::new(r::IdsNeverReused),
            Box::new(r::DeleteClears),
            Box::new(r::ScopeIsolated),
            Box::new(super::store_health::HealthSaid),
            Box::new(r::LogAppend),
            Box::new(r::UnlockReason),
        ]
    }
}

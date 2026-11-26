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
        use super::store_rules_index as i;
        use super::store_rules_lock as l;
        vec![
            Box::new(r::Roundtrip),
            Box::new(r::Remove),
            Box::new(i::StampMoves),
            Box::new(i::StampStill),
            Box::new(l::LockedRefused),
            Box::new(i::IndexAtomic),
            Box::new(i::IdsNeverReused),
            Box::new(r::DeleteClears),
            Box::new(r::ScopeIsolated),
            Box::new(super::store_health::HealthSaid),
            Box::new(l::LogAppend),
            Box::new(l::UnlockReason),
        ]
    }
}

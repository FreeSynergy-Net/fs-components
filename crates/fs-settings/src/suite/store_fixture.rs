//! What the store rules need: a fresh store, and a way to break it.
//!
//! Pattern: Strategy (supplied by the provider under test: [`super::StoreBench`] for the memory store, the RPC client of
//! `fs-settings-rpc`, the database adapter of the program `fs-settings`).

use std::sync::Arc;

use crate::admin::InstanceStore;

/// A provider of stores for the rules of [`super::StoreSuite`].
pub trait StoreFixture {
    /// A fresh, empty store (each rule starts here).
    fn fresh(&mut self) -> Arc<dyn InstanceStore>;

    /// Damages the storage under `store` (a file turned to rubbish and reopened) so that it can only serve the built-in
    /// defaults. `false` = this provider cannot (a client over a socket): the rule `store-health-said` is then skipped,
    /// listed, never passed.
    fn break_store(&mut self, store: &dyn InstanceStore) -> bool;
}

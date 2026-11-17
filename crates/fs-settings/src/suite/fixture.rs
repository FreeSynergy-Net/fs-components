//! What the rules need from a program: its declarations and texts, two programs over one store, the admin, and the
//! program's own wiring of the lock button and of the frontend choice.
//!
//! Pattern: Strategy (the fixture is supplied: [`super::Bench`] for every program; a program may bring its own).

use crate::coded_stub::Coded;

use crate::admin::Admin;
use crate::codes::SettingCode;
use crate::declare::Declaration;
use crate::layer::Scope;
use crate::lock::{LockRights, LockView};
use crate::names::SettingKey;
use crate::settings::Settings;
use crate::value::Raw;

/// The program under test, `demo`, and a second program, `other`, on the same device.
pub trait SettingsFixture {
    /// Back to an empty store and fresh handles (each rule starts here).
    fn reset(&mut self);

    /// The declarations as the program makes them (its own and its blocks'), before any catalog checks them.
    ///
    /// # Errors
    /// The block's own `key-invalid`.
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>>;

    /// Whether the program's MF2 texts hold `key` in `language` (`de`, `en`).
    fn has_text(&self, language: &str, key: &str) -> bool;

    /// The settings of `demo`.
    fn program(&self) -> &Settings;

    /// The settings of `other`.
    fn other(&self) -> &Settings;

    /// An admin over the same store; `can_lock` answers `rights` for every key.
    fn admin(&self, rights: bool) -> Admin;

    /// A value put into a profile by hand (another process, an editor), bypassing every check.
    fn put(&mut self, scope: &Scope, key: &SettingKey, raw: Raw);

    /// What the program's lock component gets for `key` (the program's wiring of [`crate::lock_view`]).
    fn lock_view(&self, rights: &dyn LockRights, key: &SettingKey) -> LockView;

    /// What the program hands to fs-starter as the person's frontend choice (its wiring of [`crate::person_choice`]).
    fn frontend_choice(&self) -> Option<String>;
}

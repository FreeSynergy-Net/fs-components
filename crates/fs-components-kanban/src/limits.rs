//! The settings of the board: how many facts one card takes. A SETTING of the instance (default 4), not a constant of the code
//! (rule 57); the host reads the effective value ([`facts_limit`]) and builds its cards with [`crate::Card::limited`].
//!
//! Why 4: the facts are ONE row in the card (one row for all facts, the card keeps its height); four short values fit the
//! narrowest column of every style. That is a default, no ceiling: an admin or a person raises it, the card then asks the
//! column to be wider, it never drops a value.
//!
//! Pattern: Strategy (`Declares`: the block says which settings it has) + Value object (`Limit`).

use fs_components_fields::limit::declaration;
use fs_settings::{
    Coded, Declaration, Declares, GroupId, Owner, OwnerKind, SettingCode, SettingKey, Settings,
};

use fs_components_fields::Limit;

/// The default of [`FACTS_LOCAL`]: how many facts a card takes.
pub const DEFAULT_FACTS: Limit = Limit::of(4);

/// The local part of the key `fs.kanban.card-facts`.
pub const FACTS_LOCAL: &str = "card-facts";

/// The words of the settings (MessageFormat 2): English and German.
pub const TEXT_EN: &str = include_str!("texts/en.mf2");
/// See [`TEXT_EN`].
pub const TEXT_DE: &str = include_str!("texts/de.mf2");

/// The settings the kanban board declares.
#[derive(Clone, Copy, Debug, Default)]
pub struct KanbanSettings;

impl KanbanSettings {
    /// The key `fs.kanban.card-facts`.
    ///
    /// # Errors
    /// `key-invalid` (a bug of this crate, caught by its own test).
    pub fn facts_key() -> Result<SettingKey, Coded<SettingCode>> {
        SettingKey::new(&Owner::new(OwnerKind::Framework, "kanban")?, FACTS_LOCAL)
    }
}

impl Declares for KanbanSettings {
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>> {
        Ok(vec![declaration(
            Self::facts_key()?,
            DEFAULT_FACTS,
            GroupId::new("board")?,
        )])
    }
}

/// How many facts a card takes: the effective value of the setting, [`DEFAULT_FACTS`] when it is not declared.
pub fn facts_limit(settings: &Settings) -> Limit {
    match KanbanSettings::facts_key() {
        Ok(key) => Limit::read(settings, &key, DEFAULT_FACTS),
        Err(_) => DEFAULT_FACTS,
    }
}

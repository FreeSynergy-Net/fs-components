//! The settings of the board workbench: how many field values the detail frame takes. A SETTING of the instance, not a constant
//! of the code (rule 57); the host reads the effective value ([`elements_limit`]) and builds its frames with
//! [`DetailFrame::limited`](super::DetailFrame::limited).
//!
//! Why 32: the type that sets the need is `tasks`: one title and 16 fields, 17 values. The frame scrolls, so more values cost
//! no layout; 32 leaves room for the next types without a change. A default, no ceiling: the instance raises it, the frame then
//! shows more, it never drops a value. The words of the setting are in `texts/*.mf2` (`setting-label-…`, `setting-about-…`).
//!
//! Pattern: Strategy (`Declares`: the workbench says which settings it has) + Value object (`Limit`).

use fs_components_fields::Limit;
use fs_components_fields::limit::declaration;
use fs_settings::{
    Coded, Declaration, Declares, GroupId, Owner, OwnerKind, SettingCode, SettingKey, Settings,
};

/// The default of [`ELEMENTS_LOCAL`]: how many elements a detail frame takes.
pub const DEFAULT_ELEMENTS: Limit = Limit::of(32);

/// The local part of the key `fs.workbenches.board-detail-elements`.
pub const ELEMENTS_LOCAL: &str = "board-detail-elements";

/// The settings the board workbench declares.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoardSettings;

impl BoardSettings {
    /// The key `fs.workbenches.board-detail-elements`.
    ///
    /// # Errors
    /// `key-invalid` (a bug of this crate, caught by its own test).
    pub fn elements_key() -> Result<SettingKey, Coded<SettingCode>> {
        SettingKey::new(
            &Owner::new(OwnerKind::Framework, "workbenches")?,
            ELEMENTS_LOCAL,
        )
    }
}

impl Declares for BoardSettings {
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>> {
        Ok(vec![declaration(
            Self::elements_key()?,
            DEFAULT_ELEMENTS,
            GroupId::new("board")?,
        )])
    }
}

/// How many elements a frame takes: the effective value of the setting, [`DEFAULT_ELEMENTS`] when it is not declared.
pub fn elements_limit(settings: &Settings) -> Limit {
    match BoardSettings::elements_key() {
        Ok(key) => Limit::read(settings, &key, DEFAULT_ELEMENTS),
        Err(_) => DEFAULT_ELEMENTS,
    }
}

//! The limit of the facts of a card, as a PARAMETER and a DEFAULT, not as a read setting (rule 57). The block does not read
//! settings and does not depend on `fs-settings`: the host registers [`FACTS_SETTING`] (key `fs.kanban.card-facts`) with the
//! settings store and builds its cards with [`crate::Card::limited`] and the effective value.
//!
//! Why 4: the facts are ONE row in the card (one row for all facts, the card keeps its height); four short values fit the
//! narrowest column of every style. That is a default, no ceiling: an admin or a person raises it.
//!
//! Pattern: plain data (`LimitSetting`) + Value object (`Limit`).

use fs_components_fields::{Limit, limit::LimitSetting};

/// The default of the facts of a card.
pub const DEFAULT_FACTS: Limit = Limit::of(4);

/// The setting `fs.kanban.card-facts` as data; its words are [`TEXT_EN`] and [`TEXT_DE`].
pub const FACTS_SETTING: LimitSetting = LimitSetting {
    key: "fs.kanban.card-facts",
    default: DEFAULT_FACTS,
    group: "board",
    label_key: "setting-label-fs-kanban-card-facts",
    about_key: "setting-about-fs-kanban-card-facts",
};

/// The words of the setting (`MessageFormat` 2): English.
pub const TEXT_EN: &str = include_str!("texts/en.mf2");
/// The words of the setting: German.
pub const TEXT_DE: &str = include_str!("texts/de.mf2");

//! The limit of the elements of the detail frame, as a PARAMETER and a DEFAULT, not as a read setting (rule 57). The workbench
//! does not read settings and does not depend on `fs-settings`: the host registers [`ELEMENTS_SETTING`] (key
//! `fs.workbenches.board-detail-elements`) and builds its frames with [`DetailFrame::limited`](super::DetailFrame::limited).
//!
//! Why 32: the type that sets the need is `tasks`: one title and 16 fields, 17 values. The frame scrolls, so more values cost no
//! layout; 32 leaves room for the next types. A default, no ceiling: the instance raises it. The words of the setting are in
//! `texts/*.mf2` (`setting-label-...`, `setting-about-...`).
//!
//! Pattern: plain data (`LimitSetting`) + Value object (`Limit`).

use fs_components_fields::{Limit, limit::LimitSetting};

/// The default of the elements of a detail frame.
pub const DEFAULT_ELEMENTS: Limit = Limit::of(32);

/// The setting `fs.workbenches.board-detail-elements` as data.
pub const ELEMENTS_SETTING: LimitSetting = LimitSetting {
    key: "fs.workbenches.board-detail-elements",
    default: DEFAULT_ELEMENTS,
    group: "board",
    label_key: "setting-label-fs-workbenches-board-detail-elements",
    about_key: "setting-about-fs-workbenches-board-detail-elements",
};

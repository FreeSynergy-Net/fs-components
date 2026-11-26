//! The names of the command slots: a slot is a GESTURE of the person at the block, never a command name or a channel of a form.
//!
//! Pattern: Specification (one rule, `is_gesture_slot`) over a closed word list; the list is the same as `trigger::ALL` of
//! `fs-components-core` (a test of the kanban crate, which knows both, holds them together; this crate does not depend on core).

use super::slot::{Slot, SlotKind};

/// The trigger words: the names a command slot of a block may have (and the second half of `<part>-<word>`).
pub const TRIGGERS: [&str; 4] = ["activate", "menu", "drop", "quick-add"];

/// Whether `name` is a gesture slot of a block with these `parts`: a trigger word, or `<part>-<trigger word>` for a gesture at a
/// part that is no record of the block (for example a later `column-menu`).
#[must_use]
pub fn is_gesture_slot(name: &str, parts: &[&str]) -> bool {
    TRIGGERS.contains(&name)
        || TRIGGERS.iter().any(|word| {
            name.strip_suffix(word)
                .and_then(|rest| rest.strip_suffix('-'))
                .is_some_and(|part| parts.contains(&part))
        })
}

/// The command slots whose name is no gesture slot of the block (in file order).
pub(super) fn misnamed<'a>(slots: &'a [Slot], parts: &[&str]) -> Vec<&'a str> {
    slots
        .iter()
        .filter(|s| s.kind == SlotKind::Command && !is_gesture_slot(&s.name, parts))
        .map(|s| s.name.as_str())
        .collect()
}

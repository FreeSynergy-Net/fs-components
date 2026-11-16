//! Which mono face Craft gets on this machine.
//!
//! Pattern: none (two pure lookups over the system fonts and one warning once per process). No font is carried (ADR 0089): the
//! list is `styles::craft::MONO`; Blinc takes the first installed name of it, so this module only asks the same font database
//! which one that is, and says so once when there is none.

use std::sync::{Once, OnceLock};

use fontdb::{Database, Family, Query};
use fs_components_styles::styles::craft::MONO;

/// The first face of `styles::craft::MONO` that is installed (the generic `monospace` when only the system's own mono
/// face is there), `None` when the machine has no mono face at all. Reads the system fonts once per process.
pub fn installed_mono_face() -> Option<&'static str> {
    static FOUND: OnceLock<Option<&'static str>> = OnceLock::new();
    *FOUND.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        let has = |family: Family<'_>| {
            db.query(&Query {
                families: &[family],
                ..Query::default()
            })
            .is_some()
        };
        MONO.iter().copied().find(|name| {
            if *name == "monospace" {
                has(Family::Monospace)
            } else {
                has(Family::Name(name))
            }
        })
    })
}

/// One line on stderr, once per process, when no mono face is installed: Craft then draws its labels in the sans face. It
/// never fails.
pub(crate) fn warn_when_missing() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if installed_mono_face().is_none() {
            eprintln!(
                "fs-components-render: no monospace font is installed (looked for {}); Craft shows its labels in the default face. Install JetBrains Mono through fs-software or any mono font.",
                MONO.join(", ")
            );
        }
    });
}

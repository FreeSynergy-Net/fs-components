//! Which mono face Craft gets on this machine.
//!
//! Pattern: none (a pure lookup over the system fonts and one warning once per process). No font is carried (ADR 0089): the
//! list is `styles::craft::MONO`; Blinc takes the first installed name of it, then the system's generic mono face, then its
//! own fixed list ([`BLINC_MONO`], copied from `blinc_text` `registry.rs` of the pinned tag `fs-blinc-3.1`). This module asks
//! the same font database in the same order, so it names what is drawn and says so once when there is nothing.

use std::sync::{Once, OnceLock};

use fontdb::{Database, Family, Query};
use fs_components_styles::styles::craft::MONO;

/// Blinc's own list of mono faces for the generic `monospace` (`registry.rs`, `find_generic_font_id`, tag `fs-blinc-3.1`).
const BLINC_MONO: [&str; 8] = [
    "JetBrains Mono",
    "Fira Code",
    "Roboto Mono",
    "Droid Sans Mono",
    "SF Mono",
    "Menlo",
    "Consolas",
    "DejaVu Sans Mono",
];

/// The mono face found on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonoFace {
    /// The family name (`"monospace"` when the system's generic mono face was the one found).
    pub family: &'static str,
    /// Whether the face is monospaced: the font database marks it so (the `post` table), or the advances of `i` and `M` are
    /// equal (some mono fonts, Noto Sans Mono for one, do not set the flag).
    pub monospaced: bool,
}

/// Whether `i` and `M` have the same advance in the face `id`.
fn equal_advances(db: &Database, id: fontdb::ID) -> bool {
    db.with_face_data(id, |data, index| {
        let face = ttf_parser::Face::parse(data, index).ok()?;
        let advance = |c: char| face.glyph_index(c).and_then(|g| face.glyph_hor_advance(g));
        Some(advance('i')? == advance('M')?)
    })
    .flatten()
    .unwrap_or(false)
}

/// The face Craft's labels get: the first installed of `styles::craft::MONO`, else the system's generic mono face, else the
/// first installed of Blinc's own list; `None` when the machine has no mono face at all. Reads the system fonts once per
/// process.
pub fn installed_mono_face() -> Option<MonoFace> {
    static FOUND: OnceLock<Option<MonoFace>> = OnceLock::new();
    *FOUND.get_or_init(|| {
        let mut db = Database::new();
        db.load_system_fonts();
        let find = |name: &'static str| {
            let family = if name == "monospace" {
                Family::Monospace
            } else {
                Family::Name(name)
            };
            let id = db.query(&Query {
                families: &[family],
                ..Query::default()
            })?;
            Some(MonoFace {
                family: name,
                monospaced: db.face(id).is_some_and(|f| f.monospaced) || equal_advances(&db, id),
            })
        };
        MONO.iter().chain(BLINC_MONO.iter()).copied().find_map(find)
    })
}

/// One line on stderr, once per process, when no mono face is installed: Craft then draws its labels in the default face. It
/// never fails. Called when Craft is chosen and when fonts are registered, whichever comes first.
pub(crate) fn warn_when_missing() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if installed_mono_face().is_none() {
            eprintln!(
                "fs-components-render: no monospace font is installed (looked for {} (our list), then {} (Blinc's fixed list)); Craft shows its labels in the default face. Install JetBrains Mono through fs-software or any mono font.",
                MONO.join(", "),
                BLINC_MONO.join(", ")
            );
        }
    });
}

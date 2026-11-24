//! The port for the words of the forms adapter: which keys the board asks the host's `Say` for, and the sentences themselves
//! (English and German, `MessageFormat` 2). The host registers [`TEXTS`] and lists [`KEYS`]; it keeps no list of its own.
//!
//! Pattern: Ports and Adapters (a published port of the adapter: the element names the words it needs and brings them, the host
//! only passes them on). Deviation: it is data (a list and two files), not a trait, because the host needs nothing else than the ids.

use fs_components_mf2::Source;

/// The refusal codes of the host that have a sentence (`kanban.refused.<code>`); a code outside this list shows as its key.
pub const REFUSED_CODES: &[&str] = &[
    "locked",
    "unsupported",
    fs_components_core::form_write::ASK_UNSUPPORTED,
    "unreachable",
    "conflict",
    "rejected",
    "no-board",
    "failed",
];

/// Every key the forms adapter asks `Say` for (the refusal keys included).
pub const KEYS: &[&str] = &[
    "kanban.new_column",
    "kanban.column_name",
    "kanban.rename",
    "kanban.limit",
    "kanban.limit_placeholder",
    "kanban.limit_hint",
    "kanban.limit_invalid",
    "kanban.refused.locked",
    "kanban.refused.unsupported",
    "kanban.refused.ask-unsupported",
    "kanban.refused.unreachable",
    "kanban.refused.conflict",
    "kanban.refused.rejected",
    "kanban.refused.no-board",
    "kanban.refused.failed",
];

/// The two language files of these keys; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
pub const TEXTS: &[Source] = &[
    Source {
        language: "en",
        text: include_str!("../texts/form-en.mf2"),
    },
    Source {
        language: "de",
        text: include_str!("../texts/form-de.mf2"),
    },
];

/// The MF2 id of a key (the ids carry a hyphen where the key has a dot).
pub fn id_of(key: &str) -> String {
    key.replace('.', "-")
}

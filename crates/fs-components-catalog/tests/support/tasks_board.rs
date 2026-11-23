//! The bindings of the tasks board as a forms description would hold them (sketch `tasks-board.toml`, block `board`), built
//! by hand until the format of F2 reads them, and the provider fs-tasks would be after the `tasks_v2` follow-up. Test K-case
//! K19 checks them against the draft catalog: empty = the board can be drawn from the description alone.
//!
//! Pattern: Factory functions (pure).

use super::tasks_catalog::{TASK, task_shape, task_type};
use fs_components_catalog::{AcceptsCode, Binding, ContractRange, Provider, Role, Source};
use fs_records::path;
use fs_types::coded::Coded;
use semver::Version;

/// The contract range the description binds to: the tasks contract after the `tasks_v2` follow-up (assumed `0.2`, draft O6).
///
/// # Errors
/// None for this fixed text; `range-invalid` would mean the parser broke.
pub fn tasks_range() -> Result<ContractRange, Coded<AcceptsCode>> {
    ContractRange::parse("tasks@>=0.2.0, <0.3.0")
}

/// fs-tasks as a provider: capability `tasks`, version 0.2.0, its description and shape.
#[must_use]
pub fn tasks_provider() -> Provider {
    Provider {
        capability: "tasks".into(),
        version: Version::new(0, 2, 0),
        types: vec![task_type()],
        shapes: vec![task_shape()],
    }
}

/// The bindings of block `board`: columns from `board.columns` (grouping `board`), order by hand, the card's title and
/// facts, the lock, moving, quick add and the card menu. `lanes = true` adds the third dimension: lanes by priority (the
/// user's „Prio als 3. Dimension“, Gespräch 4 Z. 10); the default view has none (1:1 with today's board).
///
/// # Errors
/// Only if [`tasks_range`] fails.
pub fn board_bindings(lanes: bool) -> Result<Vec<Binding>, Coded<AcceptsCode>> {
    let range = tasks_range()?;
    let b = |slot: &str, source: Source| Binding::new(slot, &range, TASK, source);
    let fact = |name: &str| Source::Field {
        path: path(name),
        role: Role::Compact,
    };
    let mut out = vec![
        b("cards", Source::Records),
        b("columns", Source::Grouping("board".into())),
        b("order", Source::Order("rank".into())),
        b(
            "card-title",
            Source::Field {
                path: path("title"),
                role: Role::Display,
            },
        ),
        b("card-facts", fact("due")),
        b("card-facts", fact("priority")),
        b("card-facts", fact("participants")),
        b("card-facts", fact("keywords")),
        b("card-lock", Source::Lock),
        b("move", Source::Command("move".into())),
        b("quick-add", Source::Command("create".into())),
        b("card-menu", Source::Command("claim".into())),
        b("card-menu", Source::Command("release".into())),
        b("card-menu", Source::Command("note".into())),
        b("card-menu", Source::Command("delete".into())),
    ];
    if lanes {
        out.push(b("lanes", Source::Grouping("priority".into())));
    }
    Ok(out)
}

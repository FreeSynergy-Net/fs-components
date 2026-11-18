//! The snapshot the workbenches read: the groups and the rows of one moment, with every problem that came up while it was made.
//!
//! Pattern: Memento (a consistent picture of the data at one `generation`) built by one function, [`refresh`]. Nothing is left
//! out without a word: a refused page, an unknown group, a cut-off list are [`Problem`]s the host shows.

use super::mirror::{Fault, Group, RecordQuery, RecordRow, RecordsRead};
use super::shape::RecordsBinding;

/// Something that did not go as the description wants; the code is for tests and the host's `partial-notice`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// `page-refused` (a hidden field carried a value), `group-unknown`, `truncated`, or a code of the provider.
    pub code: String,
    /// A field, a group key or a count.
    pub detail: Option<String>,
}

impl Problem {
    pub(super) fn new(code: &str, detail: Option<String>) -> Self {
        Self {
            code: code.to_owned(),
            detail,
        }
    }
}

/// One moment of the data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    /// The generation of the provider when it was made.
    pub generation: u64,
    /// The groups, left to right.
    pub groups: Vec<Group>,
    /// The rows, in the order of the pages.
    pub rows: Vec<RecordRow>,
    /// How many records she may read, if the provider counts.
    pub total: Option<u64>,
    /// Everything that was refused, cut or not placed.
    pub problems: Vec<Problem>,
}

impl Snapshot {
    /// The rows of one group, in order.
    pub fn in_group<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a RecordRow> {
        self.rows
            .iter()
            .filter(move |r| r.group.as_deref() == Some(key))
    }

    /// One row by id.
    pub fn row(&self, id: &str) -> Option<&RecordRow> {
        self.rows.iter().find(|r| r.id == id)
    }
}

/// How the worker stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Nothing has arrived yet.
    Loading,
    /// The snapshot is current as of the last poll.
    Ready,
    /// The last call failed: the snapshot is the last known state and the board says so (never silently old).
    Stale(Fault),
}

/// The result of one refresh.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The generation is the one already known.
    Unchanged,
    /// A new snapshot.
    Fresh(Snapshot),
    /// The provider did not answer.
    Failed(Fault),
}

/// Ask the provider; with `known` the generation of the snapshot in hand, an unchanged generation costs one call.
pub async fn refresh(
    records: &dyn RecordsRead,
    binding: &RecordsBinding,
    known: Option<u64>,
) -> Outcome {
    match read(records, binding, known).await {
        Ok(Some(snapshot)) => Outcome::Fresh(snapshot),
        Ok(None) => Outcome::Unchanged,
        Err(fault) => Outcome::Failed(fault),
    }
}

async fn read(
    records: &dyn RecordsRead,
    b: &RecordsBinding,
    known: Option<u64>,
) -> Result<Option<Snapshot>, Fault> {
    let scope = b.scope.as_deref();
    let generation = records.generation(&b.data_type, scope).await?;
    if known == Some(generation) {
        return Ok(None);
    }
    let set = records.groups(&b.data_type, &b.grouping, scope).await?;
    let mut snap = Snapshot {
        generation,
        groups: set.groups,
        ..Snapshot::default()
    };
    let mut query = RecordQuery {
        data_type: b.data_type.clone(),
        scope: b.scope.clone(),
        grouping: Some(b.grouping.clone()),
        order: b.order.clone(),
        fields: b.fields(),
        after: None,
        size: b.page_size,
    };
    let mut lost: Vec<String> = Vec::new();
    for page_no in 0..b.max_pages.max(1) {
        let page = records.find(&query).await?;
        snap.total = page.total;
        if let Some(field) = page.items.iter().find_map(RecordRow::leaking) {
            // a hidden value came with the page: the whole page is refused, with a code the reader can see (F1 K30)
            snap.problems
                .push(Problem::new("page-refused", Some(field.to_owned())));
        } else {
            for row in page.items {
                let placed = row
                    .group
                    .as_deref()
                    .is_some_and(|g| snap.groups.iter().any(|x| x.key == g));
                if placed {
                    snap.rows.push(row);
                } else {
                    lost.push(row.id);
                }
            }
        }
        match page.next {
            Some(next) => {
                if page_no + 1 >= b.max_pages.max(1) {
                    snap.problems.push(Problem::new("truncated", Some(next)));
                    break;
                }
                query.after = Some(next);
            }
            None => break,
        }
    }
    if !lost.is_empty() {
        snap.problems
            .push(Problem::new("group-unknown", Some(lost.len().to_string())));
    }
    Ok(Some(snap))
}

//! The read side of `fs-records` as this crate needs it: the same names and meanings, narrower (no filter tree, no changes feed).
//!
//! Whether a hidden field may carry a value, and what the reader is told, is the plan's rule (`Judge`), not this mirror's.
//!
//! Pattern: Adapter port (a narrow mirror of the `Records` trait of the F1 draft, probe `3077cc53` in fs-types). The futures are
//! boxed and not `Send`: a capnp client is `Rc`. A hidden field carries no value; a lock is the program's decision.

use std::future::Future;
use std::pin::Pin;

/// Why a call failed: a code of the records interface (`unavailable`, `unknown-grouping`, ...), plus an optional detail text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    /// The code.
    pub code: String,
    /// What the provider added (a path, a limit); not a sentence for the reader.
    pub detail: Option<String>,
}

impl Fault {
    /// A fault with a code.
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: None,
        }
    }

    /// With a detail.
    #[must_use]
    pub fn with(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// A pending answer (boxed, not `Send`).
pub type Pending<'a, T> = Pin<Box<dyn Future<Output = Result<T, Fault>> + 'a>>;

/// The values of one field (canonical texts of its basic type; empty = no value).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldValue {
    /// The top-level field.
    pub field: String,
    /// Canonical texts.
    pub values: Vec<String>,
}

/// Why the person does not see a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HiddenReason {
    /// No right to read it.
    NoRight,
    /// Marked sensitive and not shared with her.
    Sensitive,
}

/// A field the person may not see.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hidden {
    /// The top-level field.
    pub field: String,
    /// Why.
    pub reason: HiddenReason,
}

/// What kind of lock a record carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LockKind {
    /// Someone took it over until a time.
    Claimed,
    /// She may read, not change.
    ReadOnly,
    /// An admin locked it.
    Admin,
}

/// The lock of a record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordLock {
    /// The kind.
    pub kind: LockKind,
    /// The reason as the locker wrote it (user text).
    pub reason: Option<String>,
}

/// One record as the asking person may see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordRow {
    /// The identity (an IRI the host never makes up).
    pub id: String,
    /// The readable fields asked for.
    pub values: Vec<FieldValue>,
    /// The fields asked for that she may not read.
    pub hidden: Vec<Hidden>,
    /// The lock, if any.
    pub lock: Option<RecordLock>,
    /// The record commands she may run on it now (rights only).
    pub may: Vec<String>,
    /// The group key in the grouping of the query.
    pub group: Option<String>,
}

impl RecordRow {
    /// The values of a field; `None` when the field has none or is hidden.
    pub fn values_of(&self, field: &str) -> Option<&[String]> {
        self.values
            .iter()
            .find(|v| v.field == field && !v.values.is_empty())
            .map(|v| v.values.as_slice())
    }

    /// Why a field is hidden, if it is.
    pub fn hidden_reason(&self, field: &str) -> Option<HiddenReason> {
        self.hidden
            .iter()
            .find(|h| h.field == field)
            .map(|h| h.reason)
    }
}

/// One page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    /// At most `size` records.
    pub items: Vec<RecordRow>,
    /// Where the next page starts; `None` = the last.
    pub next: Option<String>,
    /// The number of matching records she may read; `None` = the provider does not count.
    pub total: Option<u64>,
}

/// One group now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// The stable key.
    pub key: String,
    /// The title as the owner wrote it (user data); `None` = the key is shown.
    pub title: Option<String>,
    /// A limit that is a hint, never a lock.
    pub hint_limit: Option<u32>,
}

/// The groups of one grouping, left to right.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupSet {
    /// Changes when the set changes.
    pub revision: u64,
    /// The groups.
    pub groups: Vec<Group>,
}

/// What is asked of `find`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordQuery {
    /// The data type.
    pub data_type: String,
    /// The container record; `None` = all she may see.
    pub scope: Option<String>,
    /// The grouping; every record then carries its group key.
    pub grouping: Option<String>,
    /// A named order of the shape.
    pub order: Option<String>,
    /// The top-level fields to answer.
    pub fields: Vec<String>,
    /// The `next` of the page before.
    pub after: Option<String>,
    /// At most this many.
    pub size: u32,
}

/// The read side of the records interface.
pub trait RecordsRead {
    /// One page.
    fn find(&self, query: &RecordQuery) -> Pending<'_, Page>;

    /// The groups of a grouping now.
    fn groups(&self, data_type: &str, grouping: &str, scope: Option<&str>)
    -> Pending<'_, GroupSet>;

    /// A number that changes whenever a record or the group set changed.
    fn generation(&self, data_type: &str, scope: Option<&str>) -> Pending<'_, u64>;
}

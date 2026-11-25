//! A slot of a block: its kind, the basic types and the role it takes, how many values, whether it must be bound.
//!
//! Pattern: Value object. The keys are the words of the TOML files (closed lists; an unknown word is a problem, never ignored).

use super::forms::FormRef;
use fs_types::BasicType;

/// What a slot takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotKind {
    /// The value of one field, in a [`Role`].
    Value,
    /// The records of one data type: a list ([`Cardinality::Multiple`]) or one record ([`Cardinality::Single`]).
    Records,
    /// A grouping of the records of the slot it lies in (board columns, lanes).
    Grouping,
    /// An order of those records (by hand, by due date, by priority).
    Order,
    /// A command of the type (a button, a menu entry).
    Command,
    /// The lock state of a record (shown by the lock component; the rights come from the program).
    Lock,
}

/// The four components of a basic type (ADR 0066 §4; the same four as `fs_components_fields::Mode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// To type into.
    Input,
    /// To read, every part with its label.
    Display,
    /// In one line of running text.
    Compact,
    /// In a list cell.
    Cell,
}

/// How many values a slot shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cardinality {
    /// At most one: a field with `multiple` does not fit.
    Single,
    /// Any number: every field fits.
    Multiple,
}

/// The basic types a value slot takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BasicSet {
    /// Every basic type (`basic = ["any"]`): a slot that shows any field in its compact or cell component.
    Any,
    /// Only these.
    Only(Vec<BasicType>),
}

impl BasicSet {
    /// Whether `basic` is in the set. `Unknown` is in no set: a field of a newer type is plain text to an old reader, and
    /// plain text is bound as `text`, never as `unknown`.
    #[must_use]
    pub fn takes(&self, basic: BasicType) -> bool {
        match self {
            Self::Any => basic != BasicType::Unknown,
            Self::Only(list) => basic != BasicType::Unknown && list.contains(&basic),
        }
    }
}

/// One slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    /// Kebab-case name, unique in its block.
    pub name: String,
    /// What it takes.
    pub kind: SlotKind,
    /// For [`SlotKind::Value`] only; otherwise `Only(vec![])`.
    pub basic: BasicSet,
    /// For [`SlotKind::Value`] only.
    pub role: Option<Role>,
    /// How many values or records.
    pub cardinality: Cardinality,
    /// A required slot without a binding locks the block with a reason.
    pub required: bool,
    /// The records slot of the same block whose type this slot's field, grouping, order, command or lock belongs to.
    pub within: Option<String>,
    /// How many bindings the slot takes, shown in binding order (1 or more, no ceiling of ours: the number is a `u32`). It is
    /// what ONE description may list. How many a card or a frame takes at run time is a setting of the instance (kanban
    /// `fs.kanban.card-facts`, workbenches `fs.workbenches.board-detail-elements`), not this number.
    pub repeat: u32,
    /// For [`SlotKind::Grouping`] only: the forms whose rows are the HEADS of the groups in their own order (the lanes of a board
    /// come from a second source, `heads = ["rows@^1.0.0"]`); empty = the groups are the values of the field. Closed: a binding
    /// names its head source, the form is checked against these.
    pub heads: Vec<FormRef>,
    /// For [`SlotKind::Command`] only: the names of the arguments the component hands the command, in order (kebab case, each
    /// once; empty = none). It is the stable argument schema of a gesture: the host checks `[[action]].trigger` (= the slot name)
    /// and the arguments of the form against it. The record the gesture is about is no argument, it is the call's record.
    pub args: Vec<String>,
}

/// The word of a basic type in the contract files: the enumerant name of `types.capnp` (lower camel case).
#[must_use]
// With fs-types 0.0.2 only `Unknown` is left for the wildcard; the wildcard is wanted for the basic types of 0.0.3.
#[allow(clippy::match_wildcard_for_single_variants)]
pub fn basic_key(basic: BasicType) -> &'static str {
    match basic {
        BasicType::Text => "text",
        BasicType::MultilineText => "multilineText",
        BasicType::Email => "email",
        BasicType::Phone => "phone",
        BasicType::Name => "name",
        BasicType::Address => "address",
        BasicType::Place => "place",
        BasicType::Date => "date",
        BasicType::DateTime => "dateTime",
        BasicType::Language => "language",
        BasicType::Link => "link",
        BasicType::Image => "image",
        BasicType::Key => "key",
        BasicType::Choice => "choice",
        BasicType::Keywords => "keywords",
        BasicType::Reference => "reference",
        BasicType::Integer => "integer",
        BasicType::Whole => "whole",
        BasicType::Flag => "flag",
        BasicType::Duration => "duration",
        // `Unknown` and a basic type added after the pin have no word in the
        // contract files: they read as `unknown`, which no file may use. Consequences: a slot with `basic = [..]` (`Only`) refuses
        // such a field as `basic-mismatch` and then says `basic=unknown` although the type is a real one (the report is too
        // coarse until the pin is lifted); a slot with `basic = ["any"]` takes it (`BasicSet::takes` only refuses `Unknown`).
        // Open: a K-case for the new types after the pin lift; G7 (`renders`, all words) settles it for good.
        _ => "unknown",
    }
}

/// The basic types this reader has a word for (every one but `Unknown`, and but the ones added after the pin).
pub const BASICS: [BasicType; 20] = [
    BasicType::Text,
    BasicType::MultilineText,
    BasicType::Email,
    BasicType::Phone,
    BasicType::Name,
    BasicType::Address,
    BasicType::Place,
    BasicType::Date,
    BasicType::DateTime,
    BasicType::Language,
    BasicType::Link,
    BasicType::Image,
    BasicType::Key,
    BasicType::Choice,
    BasicType::Keywords,
    BasicType::Reference,
    BasicType::Integer,
    BasicType::Whole,
    BasicType::Flag,
    BasicType::Duration,
];

/// The basic type of a word; `unknown` is not a word a file may use.
#[must_use]
pub fn basic_from_key(key: &str) -> Option<BasicType> {
    BASICS.into_iter().find(|b| basic_key(*b) == key)
}

impl SlotKind {
    /// The word in the files.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Records => "records",
            Self::Grouping => "grouping",
            Self::Order => "order",
            Self::Command => "command",
            Self::Lock => "lock",
        }
    }

    /// The kind of a word.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        [
            Self::Value,
            Self::Records,
            Self::Grouping,
            Self::Order,
            Self::Command,
            Self::Lock,
        ]
        .into_iter()
        .find(|k| k.key() == key)
    }
}

impl Role {
    /// The word in the files.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Display => "display",
            Self::Compact => "compact",
            Self::Cell => "cell",
        }
    }

    /// The role of a word.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        [Self::Input, Self::Display, Self::Compact, Self::Cell]
            .into_iter()
            .find(|r| r.key() == key)
    }
}

/// Whether `name` is a slot name: `[a-z][a-z0-9-]{0,31}`, no `--`, not ending in `-`.
///
/// The 32 is a FORMAT limit of the contract file (`by=technical`, rule 57), not a runtime limit: a slot name becomes a part of
/// element ids and of message keys, which stay short and fixed; the format is written down in `contract/contract.toml`
/// (comment "slot names"). Nothing a person or a provider could want to raise: names are written by the block author.
#[must_use]
pub(crate) fn name_ok(name: &str) -> bool {
    let b = name.as_bytes();
    (1..=32).contains(&b.len())
        && b[0].is_ascii_lowercase()
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
        && !name.contains("--")
        && !name.ends_with('-')
}

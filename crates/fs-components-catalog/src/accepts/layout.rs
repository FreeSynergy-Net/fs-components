//! The layout keys of a block (block contract 0.4, A5 + A5b): `places`, `user`, `sizing`, `reads`, `sets`, `defaults`.
//!
//! Pattern: Value object. The schema is closed like the one of a slot (see `layout_read`): an unknown key or word, `min > max`, a
//! place that is no `part` of the block, or an attribute that is not in the vocabulary of `contract.toml` is a problem.

use std::collections::BTreeMap;

/// What a place takes (the classes of ENTSCHEID-zeichnen K5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Takes {
    /// A command (a button, a menu entry).
    Command,
    /// A whole form of a module.
    Form,
    /// Another frame (a place that holds a place tree).
    Frame,
    /// A single value.
    Value,
    /// Anything the catalog knows.
    Any,
}

/// How many a place takes at most.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Max {
    /// This many (1 or more).
    Count(u32),
    /// No ceiling of ours (`"n"`).
    Many,
}

/// Who orders what a place holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceOrder {
    /// The layout (file order).
    Layout,
    /// The person (reorder).
    User,
}

/// What the person may do with a frame or block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UserAct {
    /// Change its size.
    Resize,
    /// Change the order of what it holds.
    Reorder,
    /// Move it to another place.
    Move,
    /// Close it.
    Close,
    /// Let it float.
    Float,
}

/// How a block gets its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sizing {
    /// It has a natural size.
    Content,
    /// It fills what its frame gives.
    Fill,
}

/// One place of a frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceSpec {
    /// The place, one of the `parts` of the block.
    pub place: String,
    /// What it takes, in file order.
    pub takes: Vec<Takes>,
    /// At least this many (0 = may stay empty).
    pub min: u32,
    /// At most this many.
    pub max: Max,
    /// Who orders its content; `None` = not said.
    pub order: Option<PlaceOrder>,
}

/// The layout keys of one block; all empty for a block that has none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layout {
    /// Places (frames only).
    pub places: Vec<PlaceSpec>,
    /// What the person may do.
    pub user: Vec<UserAct>,
    /// How it is sized; `None` = not said.
    pub sizing: Option<Sizing>,
    /// Attributes it reports (a subset of the vocabulary `reads`).
    pub reads: Vec<String>,
    /// Attributes a layout may set (a subset of the vocabulary `sets`).
    pub sets: Vec<String>,
    /// Starting values of attributes in `sets`.
    pub defaults: BTreeMap<String, String>,
}

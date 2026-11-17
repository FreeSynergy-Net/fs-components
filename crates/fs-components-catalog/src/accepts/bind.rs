//! A binding: which data goes into which slot. It is typed (ADR 0087 Nachtrag point 4: capability, contract version, data
//! type, field path, role) and names no program, no record and no expression.
//!
//! Pattern: Value objects. What an installation offers is a [`Provider`]: its capability with version, the type catalog it
//! answers (`TypeCatalog.describe`) and the shapes it answers (`Records.shape`).

use fs_types::TypeDescription;

use super::range::{ContractRange, Version2};
use super::slot::{Role, SlotKind};
use fs_records::Shape;

/// What a binding takes from the data type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    /// The records of the type (a list, or one record for a single slot).
    Records,
    /// One field; v1 binds only a top-level field (`path` has one step), deeper steps are refused as `field-unknown`.
    Field {
        /// The field path (`["title"]`).
        path: Vec<String>,
        /// The role the description asks for; it must be the slot's role.
        role: Role,
    },
    /// A grouping of the shape (`board`, `progress`, `priority`).
    Grouping(String),
    /// An order of the shape (`rank`, `due`, `priority`).
    Order(String),
    /// A command of the type description.
    Command(String),
    /// The lock state the records carry.
    Lock,
}

impl Source {
    /// The slot kind this source fits.
    #[must_use]
    pub fn kind(&self) -> SlotKind {
        match self {
            Self::Records => SlotKind::Records,
            Self::Field { .. } => SlotKind::Value,
            Self::Grouping(_) => SlotKind::Grouping,
            Self::Order(_) => SlotKind::Order,
            Self::Command(_) => SlotKind::Command,
            Self::Lock => SlotKind::Lock,
        }
    }
}

/// One binding of one slot of one block in a description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    /// The slot name.
    pub slot: String,
    /// The contract with its range.
    pub contract: ContractRange,
    /// The data type (`Task`).
    pub data_type: String,
    /// What is taken.
    pub source: Source,
}

impl Binding {
    /// A binding.
    #[must_use]
    pub fn new(slot: &str, contract: &ContractRange, data_type: &str, source: Source) -> Self {
        Self {
            slot: slot.to_owned(),
            contract: contract.clone(),
            data_type: data_type.to_owned(),
            source,
        }
    }
}

/// What one installed provider offers for one capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provider {
    /// The capability key.
    pub capability: String,
    /// Its contract version (`SupportReport.contractVersion`).
    pub version: Version2,
    /// Its type catalog.
    pub types: Vec<TypeDescription>,
    /// Its shapes, one per listable type.
    pub shapes: Vec<Shape>,
}

impl Provider {
    /// The type with this name.
    #[must_use]
    pub fn type_of(&self, name: &str) -> Option<&TypeDescription> {
        self.types.iter().find(|t| t.name == name)
    }

    /// The shape of this type.
    #[must_use]
    pub fn shape_of(&self, name: &str) -> Option<&Shape> {
        self.shapes.iter().find(|s| s.data_type == name)
    }
}

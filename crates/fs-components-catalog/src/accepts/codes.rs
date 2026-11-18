//! The codes of the catalog and of the binding check, with their parameter keys. Message key: `accepts-error-` and the name.
//!
//! Pattern: none, an enum with total `match`es. A code is never renamed or reused once released; a new one is appended to the
//! enum AND to `ALL` (K-case K1).

use fs_types::coded::ErrorCode;

/// What is wrong with the catalog or with a binding, without a sentence.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AcceptsCode {
    /// A contract file is not TOML or breaks the closed schema of `accepts` (parameters `file`, `field`).
    FileInvalid,
    /// `contract.toml` names another contract version than this reader knows (parameters `found`, `want`).
    ContractVersion,
    /// A block has no `accepts` key (parameter `block`); since 0.3 every block names it, `[]` for a frame.
    AcceptsMissing,
    /// A block is defined twice across the files (parameters `block`, `file`).
    BlockTwice,
    /// A slot is named twice in one block (parameters `block`, `slot`).
    SlotTwice,
    /// A slot breaks a rule of its kind (parameters `block`, `slot`, `field`).
    SlotInvalid,
    /// A description names a block that is not in the catalog (parameter `block`).
    BlockUnknown,
    /// A binding names a slot the block does not have (parameters `block`, `slot`).
    SlotUnknown,
    /// A required slot has no binding (parameters `block`, `slot`).
    SlotUnbound,
    /// A slot is bound twice (parameters `block`, `slot`).
    BoundTwice,
    /// The kind of the binding is not the kind of the slot (parameters `block`, `slot`, `kind`).
    KindMismatch,
    /// A version range cannot be read (parameter `text`).
    RangeInvalid,
    /// No installed provider of the capability is in the range (parameters `capability`, `range`; `have` = the versions found).
    ContractMissing,
    /// The data type is not in the provider's type catalog (parameters `capability`, `type`).
    TypeUnknown,
    /// The field path is not a field of the type (parameters `type`, `path`).
    FieldUnknown,
    /// The slot does not accept the basic type of the field (parameters `block`, `slot`, `basic`).
    BasicMismatch,
    /// The role of the binding is not the role of the slot (parameters `block`, `slot`, `role`).
    RoleMismatch,
    /// A field with several values on a slot for one value (parameters `block`, `slot`, `path`).
    CardinalityMismatch,
    /// An input slot bound to a field the program sets itself (parameters `type`, `path`).
    FieldReadOnly,
    /// The type has no grouping, order or command of this name (parameters `type`, `name`).
    NameUnknown,
    /// The type has no `find` command, so it cannot be listed (parameter `type`).
    NotListable,
    /// A slot inside another slot is bound to another source than that slot (parameters `block`, `slot`, `within`).
    WithinMismatch,
    /// A `renders` entry breaks the closed schema (parameters `block`, `field`).
    RendersInvalid,
    /// A `renders` entry names a basic type this reader has no word for (parameters `block`, `word`).
    RendersUnknownBasic,
    /// Two blocks render the same basic type (and target) in the same role (parameters `basic`, `target`, `role`, `first`, `second`).
    RendersTwice,
    /// No block renders this basic type in this role (parameters `basic`, `role`; `target` when the field has one).
    RendersNone,
}

impl ErrorCode for AcceptsCode {
    const PREFIX: &'static str = "accepts-error-";
    const ALL: &'static [Self] = &[
        Self::FileInvalid,
        Self::ContractVersion,
        Self::AcceptsMissing,
        Self::BlockTwice,
        Self::SlotTwice,
        Self::SlotInvalid,
        Self::BlockUnknown,
        Self::SlotUnknown,
        Self::SlotUnbound,
        Self::BoundTwice,
        Self::KindMismatch,
        Self::RangeInvalid,
        Self::ContractMissing,
        Self::TypeUnknown,
        Self::FieldUnknown,
        Self::BasicMismatch,
        Self::RoleMismatch,
        Self::CardinalityMismatch,
        Self::FieldReadOnly,
        Self::NameUnknown,
        Self::NotListable,
        Self::WithinMismatch,
        Self::RendersInvalid,
        Self::RendersUnknownBasic,
        Self::RendersTwice,
        Self::RendersNone,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::FileInvalid => "file-invalid",
            Self::ContractVersion => "contract-version",
            Self::AcceptsMissing => "accepts-missing",
            Self::BlockTwice => "block-twice",
            Self::SlotTwice => "slot-twice",
            Self::SlotInvalid => "slot-invalid",
            Self::BlockUnknown => "block-unknown",
            Self::SlotUnknown => "slot-unknown",
            Self::SlotUnbound => "slot-unbound",
            Self::BoundTwice => "bound-twice",
            Self::KindMismatch => "kind-mismatch",
            Self::RangeInvalid => "range-invalid",
            Self::ContractMissing => "contract-missing",
            Self::TypeUnknown => "type-unknown",
            Self::FieldUnknown => "field-unknown",
            Self::BasicMismatch => "basic-mismatch",
            Self::RoleMismatch => "role-mismatch",
            Self::CardinalityMismatch => "cardinality-mismatch",
            Self::FieldReadOnly => "field-read-only",
            Self::NameUnknown => "name-unknown",
            Self::NotListable => "not-listable",
            Self::WithinMismatch => "within-mismatch",
            Self::RendersInvalid => "renders-invalid",
            Self::RendersUnknownBasic => "renders-unknown-basic",
            Self::RendersTwice => "renders-twice",
            Self::RendersNone => "renders-none",
        }
    }
}

/// The parameter keys of [`AcceptsCode`].
pub mod keys {
    /// A contract file name.
    pub const FILE: &str = "file";
    /// A schema key of a file or of a slot (`kind`, `basic`, `role`, `cardinality`, `within`, `slot`, `accepts`).
    pub const FIELD: &str = "field";
    /// The version found.
    pub const FOUND: &str = "found";
    /// The version wanted.
    pub const WANT: &str = "want";
    /// A block name.
    pub const BLOCK: &str = "block";
    /// A slot name.
    pub const SLOT: &str = "slot";
    /// A slot kind key.
    pub const KIND: &str = "kind";
    /// A role key.
    pub const ROLE: &str = "role";
    /// A basic type key.
    pub const BASIC: &str = "basic";
    /// A text that could not be read.
    pub const TEXT: &str = "text";
    /// A capability key.
    pub const CAPABILITY: &str = "capability";
    /// A version range as text.
    pub const RANGE: &str = "range";
    /// The versions an installation has, joined by `,`.
    pub const HAVE: &str = "have";
    /// A data type name.
    pub const TYPE: &str = "type";
    /// A field path, its steps joined by `/`.
    pub const PATH: &str = "path";
    /// A grouping, order or command name.
    pub const NAME: &str = "name";
    /// The slot a slot lies in.
    pub const WITHIN: &str = "within";
    /// A word of a file that is not known (`renders.basic`).
    pub const WORD: &str = "word";
    /// The first of two blocks.
    pub const FIRST: &str = "first";
    /// The second of two blocks.
    pub const SECOND: &str = "second";
    /// The target data type of a reference field.
    pub const TARGET: &str = "target";
}

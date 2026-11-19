//! The registry of the basic types that have components.
//!
//! Pattern: Registry — [`Kind`] names a type, [`Kind::field`] gives its [`BasicField`].

use super::{
    Address, Choice, Date, Email, Flag, Integer, Keywords, Link, Multiline, Name, Phone, Reference,
    Whole,
};
use crate::field::BasicField;

/// The basic types done so far (the first five of `docs/en/programs/map.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// E-mail address (RFC 5322, JSContact `EmailAddress`).
    Email,
    /// Name of a person (JSContact `Name`).
    Name,
    /// Phone number (E.164).
    Phone,
    /// Postal address (JSContact `Address`).
    Address,
    /// Calendar date (RFC 3339 `full-date`).
    Date,
    /// One entry of a fixed list (`FieldDescription.choices`).
    Choice,
    /// A set of short words.
    Keywords,
    /// The identity of another object (an absolute IRI).
    Reference,
    /// An address a person can open (`http`, `https`, `mailto`, `tel`).
    Link,
    /// Text with line breaks.
    Multiline,
    /// Yes or no (a switch).
    Flag,
    /// A whole number with a sign (64 bit).
    Integer,
    /// A whole number from 0 up.
    Whole,
}

impl Kind {
    /// All kinds in a fixed order.
    pub const ALL: [Kind; 13] = [
        Kind::Email,
        Kind::Name,
        Kind::Phone,
        Kind::Address,
        Kind::Date,
        Kind::Choice,
        Kind::Keywords,
        Kind::Reference,
        Kind::Link,
        Kind::Multiline,
        Kind::Flag,
        Kind::Integer,
        Kind::Whole,
    ];

    /// The type's rules and structure.
    pub fn field(self) -> &'static dyn BasicField {
        match self {
            Kind::Email => &Email,
            Kind::Name => &Name,
            Kind::Phone => &Phone,
            Kind::Address => &Address,
            Kind::Date => &Date,
            Kind::Choice => &Choice,
            Kind::Keywords => &Keywords,
            Kind::Reference => &Reference,
            Kind::Link => &Link,
            Kind::Multiline => &Multiline,
            Kind::Flag => &Flag,
            Kind::Integer => &Integer,
            Kind::Whole => &Whole,
        }
    }

    /// The name of the building block of the catalog that draws this kind (`contract/blocks`, key `renders`).
    pub fn block(self) -> &'static str {
        match self {
            Kind::Email => "field-email",
            Kind::Name => "field-name",
            Kind::Phone => "field-phone",
            Kind::Address => "field-address",
            Kind::Date => "field-date",
            Kind::Choice => "field-choice",
            Kind::Keywords => "field-keywords",
            Kind::Reference => "field-reference",
            Kind::Link => "field-link",
            Kind::Multiline => "field-multiline",
            Kind::Flag => "field-flag",
            Kind::Integer => "field-integer",
            Kind::Whole => "field-whole",
        }
    }

    /// The kind whose building block has this name (`None`: no component yet; the caller shows the canonical text).
    pub fn from_block(block: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.block() == block)
    }

    /// The kind with this key.
    pub fn from_key(key: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.field().key() == key)
    }
}

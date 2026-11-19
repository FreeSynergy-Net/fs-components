//! The registry of the basic types that have components.
//!
//! Pattern: Registry — [`Kind`] names a type, [`Kind::field`] gives its [`BasicField`].

use super::{Address, Date, DateTime, Duration, Email, Name, Phone};
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
    /// Date and clock time with an offset (RFC 3339 `date-time`).
    DateTime,
    /// A span of time (RFC 8984 duration, ISO 8601 subset).
    Duration,
}

impl Kind {
    /// All kinds in a fixed order.
    pub const ALL: [Kind; 7] = [
        Kind::Email,
        Kind::Name,
        Kind::Phone,
        Kind::Address,
        Kind::Date,
        Kind::DateTime,
        Kind::Duration,
    ];

    /// The type's rules and structure.
    pub fn field(self) -> &'static dyn BasicField {
        match self {
            Kind::Email => &Email,
            Kind::Name => &Name,
            Kind::Phone => &Phone,
            Kind::Address => &Address,
            Kind::Date => &Date,
            Kind::DateTime => &DateTime,
            Kind::Duration => &Duration,
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
            Kind::DateTime => "field-date-time",
            Kind::Duration => "field-duration",
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

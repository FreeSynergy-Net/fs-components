//! The basic types that have components.
//!
//! Pattern: none — declares the modules and re-exports; [`Kind`] (the registry) is in `kind.rs`.

mod address;
mod choice;
mod date;
mod date_time;
mod duration;
mod email;
mod flag;
mod keywords;
mod kind;
mod link;
mod multiline;
mod name;
mod number;
mod phone;
mod reference;

pub use address::Address;
pub use choice::{Choice, check as check_choice};
pub use date::Date;
pub use date_time::{DateTime, utc_text};
pub use duration::Duration;
pub use email::Email;
pub use flag::Flag;
pub use keywords::{Keywords, canonical as canonical_keywords};
pub use kind::Kind;
pub use link::{Link, SCHEMES};
pub use multiline::{MORE, Multiline};
pub use name::Name;
pub use number::{Integer, Whole, stepped};
pub use phone::Phone;
pub use reference::Reference;

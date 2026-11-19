//! The basic types that have components.
//!
//! Pattern: none — declares the modules and re-exports; [`Kind`] (the registry) is in `kind.rs`.

mod address;
mod choice;
mod date;
mod date_time;
mod duration;
mod email;
mod keywords;
mod kind;
mod name;
mod phone;
mod reference;

pub use address::Address;
pub use choice::{Choice, check as check_choice};
pub use date::Date;
pub use date_time::{DateTime, utc_text};
pub use duration::Duration;
pub use email::Email;
pub use keywords::{Keywords, canonical as canonical_keywords};
pub use kind::Kind;
pub use name::Name;
pub use phone::Phone;
pub use reference::Reference;

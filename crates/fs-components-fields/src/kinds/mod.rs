//! The basic types that have components.
//!
//! Pattern: none — declares the modules and re-exports; [`Kind`] (the registry) is in `kind.rs`.

mod address;
mod date;
mod email;
mod flag;
mod kind;
mod link;
mod multiline;
mod name;
mod number;
mod phone;

pub use address::Address;
pub use date::Date;
pub use email::Email;
pub use flag::Flag;
pub use kind::Kind;
pub use link::{Link, SCHEMES};
pub use multiline::{MORE, Multiline};
pub use name::Name;
pub use number::{Integer, Whole, stepped};
pub use phone::Phone;

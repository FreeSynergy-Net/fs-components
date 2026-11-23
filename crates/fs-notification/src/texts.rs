//! The words of this crate as MessageFormat 2 files (ADR 0057; the pattern of `fs-settings`): the name and the description of
//! the three settings and the sentence of every code `notification-error-<code>`. A test checks them with
//! `fs_components_mf2::check_pairs`; this crate itself does not depend on the MF2 library.
//!
//! Pattern: constants (the files are data).

/// The English file.
pub const TEXT_EN: &str = include_str!("texts/en.mf2");

/// The German file.
pub const TEXT_DE: &str = include_str!("texts/de.mf2");

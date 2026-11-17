//! The words of this crate as MessageFormat 2 files (ADR 0057; the pattern of `fs-components-filepicker/src/labels.rs`: one
//! `en.mf2` and one `de.mf2` next to the code, embedded with `include_str!`). They hold the name and the description of
//! every standard setting (`setting-label-…`, `setting-about-…`, see [`crate::Declaration::label_key`]) and the sentence
//! of the code `reason-missing`. A test checks them with `fs_components_mf2::check_pairs`; this crate itself does not
//! depend on the MF2 library.
//!
//! Pattern: constants (the files are data).

/// The English file.
pub const TEXT_EN: &str = include_str!("texts/en.mf2");

/// The German file.
pub const TEXT_DE: &str = include_str!("texts/de.mf2");

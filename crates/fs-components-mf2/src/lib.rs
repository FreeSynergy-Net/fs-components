//! `fs-components-mf2`: the texts of FreeSynergy as Unicode `MessageFormat` 2 (ADR 0057), behind the `Words` trait.
//!
//! Two ways to get texts. The RUNTIME way: [`Mf2Texts::from_packed`] loads the catalogs the packer wrote (`fs-release
//! pack-texts`: `manifest.mf2m`, `<language>.<hash>.mf2b`, and one date table per language `dates/<tag>/icu.table`); no parser
//! and no CLDR data crates are linked, `:datetime` formats by the language when the date table is there, else neutral. The OLD
//! way (feature `compile`, on by default until wave C of the texts draft): one text file per language next to the code
//! (`texts/en.mf2`, `texts/de.mf2`), embedded with `include_str!`, compiled when a message is first asked for, and
//! [`check_pairs`], the build-time check every crate with texts runs as a test. [`Mf2Texts`] is a
//! [`Words`](fs_components_words::Words) over one locale and its fallback chain (`de-AT`, `de`, English). The library is `mf2`
//! (pinned to `=3.0.0`; see `DESIGN` for the licence and the reservations of ADR 0057). It is a separate crate so that
//! `fs-components-words` stays std only.
//!
//! Pattern: Adapter (`mf2` behind `Words`) + Chain (the languages) — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod backend;
#[cfg(feature = "compile")]
mod check;
mod date;
mod date_data;
mod icu_dates;
mod packed;
#[cfg(feature = "compile")]
mod source;
#[cfg(feature = "compile")]
mod sources;
mod texts;

#[cfg(feature = "compile")]
pub use check::{check_counts, check_pairs};
pub use date_data::{DateData, DateDir, NoDates};
pub use packed::LoadError;
#[cfg(feature = "compile")]
pub use source::Source;
pub use texts::Mf2Texts;

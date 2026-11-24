//! The runtime way: `Mf2Texts` from catalogs the packer wrote (`fs-release pack-texts`), no `compile`.
//!
//! Pattern: Strategy/Provider — the directory of a texts package (the `Messages` strategy `Packed`) plus a `DateData` provider
//! for the date tables. A package is a directory with `manifest.mf2m` and one `<tag>.<hash>.mf2b` per language; the date
//! tables are not in it but in the date packages (one per language, for all programs): `dates/<tag>/icu.table`.
//! There is no list of languages: the chain of the locale (`de-AT`, `de`, then English) is looked up by file name.
//!
//! Failure modes are fixed: a missing or damaged manifest or catalog is an `Err` (the package is wrong, the caller decides);
//! a missing or damaged DATE table is no error: that language formats dates neutrally, the message stays.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use fs_components_words::Locale;
use mf2_catalog::{Catalog, Manifest};

use crate::DateData;
use crate::backend::Packed;
use crate::date_data::registry_for;
use crate::icu_dates::with_table;
use crate::texts::{Language, Mf2Texts};

/// A texts package that cannot be loaded.
#[derive(Debug)]
pub enum LoadError {
    /// A file or the directory could not be read.
    Read(String),
    /// `manifest.mf2m` is damaged.
    Manifest(String),
    /// The catalog of a language is damaged, or belongs to another manifest.
    Catalog(String),
    /// The package has no catalog for any language of the chain (not even English).
    NoLanguage,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(s) => write!(f, "cannot read {s}"),
            Self::Manifest(s) => write!(f, "damaged manifest: {s}"),
            Self::Catalog(s) => write!(f, "damaged catalog: {s}"),
            Self::NoLanguage => {
                f.write_str("the package has no catalog for the language or English")
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// The file `<tag>.<hash>.mf2b` of `tag` in `dir`, if there is one (the hash part has no dot, so `de` does not take `de-AT`).
fn catalog_file(dir: &Path, tag: &Locale) -> Result<Option<std::path::PathBuf>, LoadError> {
    let list =
        std::fs::read_dir(dir).map_err(|e| LoadError::Read(format!("{}: {e}", dir.display())))?;
    let prefix = format!("{}.", tag.as_str());
    let mut found: Vec<_> = list
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix(&prefix)
                .and_then(|rest| rest.strip_suffix(".mf2b"))
                .is_some_and(|hash| !hash.is_empty() && !hash.contains('.'))
        })
        .map(|e| e.path())
        .collect();
    found.sort();
    Ok(found.pop())
}

impl Mf2Texts {
    /// The texts of `locale` from the package in `dir`: for each tag of the chain (`de-AT`, `de`) its catalog if there is one,
    /// then English. `dates` gives the date table of each language; where it has none (or a damaged one) the message stays
    /// and only the date or time is neutral. Loading leaks the date table of a language once (about 1.4 KB, see
    /// `icu_dates.rs`).
    ///
    /// # Errors
    /// `LoadError` when the manifest or a catalog of the chain cannot be read, or no language of the chain is in the package.
    pub fn from_packed(
        dir: &Path,
        locale: &Locale,
        dates: &dyn DateData,
    ) -> Result<Self, LoadError> {
        let path = dir.join("manifest.mf2m");
        let bytes = std::fs::read(&path)
            .map_err(|e| LoadError::Read(format!("{}: {e}", path.display())))?;
        let manifest =
            Arc::new(Manifest::read(&bytes).map_err(|e| LoadError::Manifest(format!("{e:?}")))?);
        let mut wanted = locale.chain();
        wanted.push(Locale::english());
        let mut chain: Vec<Language> = Vec::new();
        for tag in wanted {
            if chain.iter().any(|l| l.tag == tag.as_str()) {
                continue;
            }
            let Some(file) = catalog_file(dir, &tag)? else {
                continue;
            };
            let bytes = std::fs::read(&file)
                .map_err(|e| LoadError::Read(format!("{}: {e}", file.display())))?;
            let plain = Catalog::new(bytes.clone(), manifest.hash())
                .map_err(|e| LoadError::Catalog(format!("{}: {e:?}", file.display())))?;
            let table = dates.blob(&tag);
            let with = with_table(&bytes, manifest.hash(), table.as_deref());
            chain.push(Language {
                tag: tag.to_string(),
                registry: registry_for(with.is_some()),
                messages: Box::new(Packed {
                    catalog: with.unwrap_or(plain),
                    manifest: Arc::clone(&manifest),
                }),
            });
        }
        if chain.is_empty() {
            return Err(LoadError::NoLanguage);
        }
        Ok(Self { chain })
    }
}

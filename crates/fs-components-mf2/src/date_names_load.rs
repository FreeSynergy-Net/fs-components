//! Which `DateNames` a locale gets: the language package when it is there, else the neutral names.
//!
//! Pattern: Factory Method – `date_names` hands out an `Arc<dyn DateNames>`; the caller does not know whether ICU4X data or the
//! Null Object is behind it. The packages are looked up like the texts (`de-AT`, then `de`); inside the blob the rule is:
//! the full locale first (`en-GB`, `en-US` have their own week), then the language alone. The person's choice of the first
//! day (`-u-fw-sun`) is an extension of the locale and stays in both tries.

use std::sync::Arc;

use fs_components_words::{DateNames, Locale};
use icu_locale::Locale as IcuLocale;
use icu_provider_blob::BlobDataProvider;

use crate::date_data::DateData;
use crate::date_names::IcuDateNames;
use crate::date_names_neutral::NeutralDateNames;

/// The locales to try inside a blob: the full one, then the language alone (extensions kept).
fn candidates(tag: &Locale) -> Vec<IcuLocale> {
    let Ok(full) = tag.as_str().parse::<IcuLocale>() else {
        return Vec::new();
    };
    let mut alone = full.clone();
    alone.id.script = None;
    alone.id.region = None;
    alone.id.variants.clear();
    if alone == full {
        vec![full]
    } else {
        vec![full, alone]
    }
}

/// The names from the blob of `package`, read for `tag`.
fn from_package(dates: &dyn DateData, package: &Locale, tag: &Locale) -> Option<IcuDateNames> {
    let blob = dates.names(package)?;
    // The provider owns its copy of the bytes; the formatters own their data after loading, so nothing is leaked.
    let provider = BlobDataProvider::try_new_from_blob(blob.to_vec().into_boxed_slice()).ok()?;
    candidates(tag)
        .iter()
        .find_map(|loc| IcuDateNames::load(&provider, loc))
}

/// The calendar names of `locale`: from the first package of its chain (`de-AT`, `de`) that has a usable `names.blob`,
/// otherwise `NeutralDateNames`. Never fails: a missing or damaged package shows the neutral form.
pub fn date_names(dates: &dyn DateData, locale: &Locale) -> Arc<dyn DateNames> {
    // The package of a language is found by the tag without its extensions (`de-u-fw-sun` reads package `de`).
    let base = locale.as_str().split("-u-").next().unwrap_or_default();
    let base = Locale::parse(base).unwrap_or_else(|_| locale.clone());
    for package in base.chain() {
        if let Some(names) = from_package(dates, &package, locale) {
            return Arc::new(names);
        }
    }
    Arc::new(NeutralDateNames)
}

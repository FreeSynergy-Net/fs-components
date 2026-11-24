//! The old way: `Mf2Texts` from embedded `.mf2` sources, compiled on first use (feature `compile`).
//!
//! Pattern: Adapter (sources behind `Messages`). Dates here are always neutral: date tables belong to packed catalogs (the
//! runtime way, `packed.rs`); wave C moves the callers to those and this file goes away with the feature.

use std::collections::HashMap;
use std::sync::Mutex;

use fs_components_words::Locale;

use crate::backend::Compiled;
use crate::date_data::registry_for;
use crate::source::entries;
use crate::texts::{Language, Mf2Texts};
use crate::{DateData, NoDates, Source};

impl Mf2Texts {
    /// The texts of `locale` from `sources`, dates neutral. See `with_dates`.
    pub fn new(sources: &[Source], locale: &Locale) -> Self {
        Self::with_dates(sources, locale, &NoDates)
    }

    /// The texts of `locale` from `sources`: for each tag of the chain (`de-AT`, `de`) the source of that language if there
    /// is one, then English. A malformed message is left out (it comes in English); `check_pairs` reports it. Sources are
    /// compiled one by one without a date table, so `dates` does not change them: dates stay neutral and the message stays.
    /// Date tables are used by `Mf2Texts::from_packed`.
    pub fn with_dates(sources: &[Source], locale: &Locale, _dates: &dyn DateData) -> Self {
        let mut wanted = locale.chain();
        wanted.push(Locale::english());
        let mut chain: Vec<Language> = Vec::new();
        for tag in wanted {
            if chain.iter().any(|l| l.tag == tag.as_str()) {
                continue;
            }
            if let Some(s) = sources
                .iter()
                .find(|s| Locale::parse(s.language).is_ok_and(|l| l == tag))
            {
                let messages = entries(s.text)
                    .0
                    .into_iter()
                    .map(|e| (e.id, e.message))
                    .collect();
                chain.push(Language {
                    tag: tag.to_string(),
                    messages: Box::new(Compiled {
                        tag: tag.to_string(),
                        messages,
                        cache: Mutex::new(HashMap::new()),
                    }),
                    registry: registry_for(false),
                });
            }
        }
        Self { chain }
    }
}

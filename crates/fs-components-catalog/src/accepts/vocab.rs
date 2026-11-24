//! The closed word lists of `contract.toml` `[vocabulary]`: the standard states and the attributes a block may read or set.
//!
//! Pattern: Value object. The lists are data of the contract, not of this code: an attribute a block names that is not in the
//! list is a problem (`attr-unknown`), never ignored. A contract without `[vocabulary]` has empty lists, so every attribute is
//! unknown (closed, not open).

use toml::Table;

/// The words of `[vocabulary]`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Vocabulary {
    /// The standard states (a block may name its own besides these).
    pub states: Vec<String>,
    /// The attributes a block may only read (measured: `bounds`, `baseline` ...).
    pub reads: Vec<String>,
    /// The attributes a layout or the person may set (`min-width`, `opacity`, a frame's `axis` ...).
    pub sets: Vec<String>,
}

impl Vocabulary {
    /// Reads `[vocabulary]` of a `contract.toml` text; a missing table or key is an empty list. A list that is not a list of
    /// words is `None` (the catalog then reports `file-invalid` with the field `vocabulary`).
    #[must_use]
    pub fn read(contract_toml: &str) -> Option<Self> {
        let table: Table = contract_toml.parse().ok()?;
        let Some(v) = table.get("vocabulary") else {
            return Some(Self::default());
        };
        let v = v.as_table()?;
        let words = |key: &str| -> Option<Vec<String>> {
            let Some(list) = v.get(key) else {
                return Some(Vec::new());
            };
            list.as_array()?
                .iter()
                .map(|w| w.as_str().map(str::to_owned))
                .collect()
        };
        Some(Self {
            states: words("states")?,
            reads: words("reads")?,
            sets: words("sets")?,
        })
    }

    /// Whether `word` may be read.
    #[must_use]
    pub fn has_read(&self, word: &str) -> bool {
        self.reads.iter().any(|w| w == word)
    }

    /// Whether `word` may be set.
    #[must_use]
    pub fn has_set(&self, word: &str) -> bool {
        self.sets.iter().any(|w| w == word)
    }
}

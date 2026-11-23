//! The catalog: every block of the contract with its slots, from `contract.toml` and the block files.
//!
//! Target: `fs-components-catalog` embeds the files of its own tag (`include_str!` of `contract/blocks/*.toml`), so the host
//! and the contract can never disagree about the version; [`BlockCatalog::read`] takes the texts so that tests and the probe
//! can feed their own.
//!
//! Pattern: Builder + Registry (name → [`BlockSpec`]).

use std::collections::BTreeMap;

use toml::Table;

use super::codes::{AcceptsCode, keys};
use super::forms::FormRef;
use super::parse::read_blocks;
use super::renders::Renders;
use super::slot::Slot;
use fs_types::coded::Coded;
use semver::Version;

/// The block contract version that carries `accepts` and `renders` (0.2 has neither, 0.3 only `accepts`): three parts, read as
/// a `semver::Version` (VERSION-EINE-QUELLE), never compared as text.
pub const CONTRACT_VERSION: &str = "0.4.0";

/// The same version as a `semver::Version`; `CONTRACT_VERSION` is its text (a test holds them together).
const WANT: Version = Version::new(0, 4, 0);

/// One block and the slots it accepts (`[]` for a frame without data).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSpec {
    /// The block name of the contract (`board`, `form-field` …).
    pub name: String,
    /// Its slots, in file order.
    pub accepts: Vec<Slot>,
    /// What it draws (block contract 0.4, key `renders`); `None` for a block that is no field component.
    pub renders: Option<Renders>,
    /// The forms it draws (block contract 0.4, key `forms`); empty for a block that draws no whole form.
    pub forms: Vec<FormRef>,
}

impl BlockSpec {
    /// The slot with this name.
    #[must_use]
    pub fn slot(&self, name: &str) -> Option<&Slot> {
        self.accepts.iter().find(|s| s.name == name)
    }
}

/// All blocks of one contract version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockCatalog {
    blocks: BTreeMap<String, BlockSpec>,
}

impl BlockCatalog {
    /// Reads `contract.toml` (only `[contract] version`) and the block files `(name, text)`, in the order given (the
    /// contract reads them in file name order).
    ///
    /// # Errors
    /// Every problem: `contract-version`, `file-invalid`, `block-twice` and those of [`read_blocks`].
    pub fn read(
        contract_toml: &str,
        files: &[(&str, &str)],
    ) -> Result<Self, Vec<Coded<AcceptsCode>>> {
        let mut problems = Vec::new();
        match contract_version(contract_toml) {
            Some(v) if Version::parse(&v).is_ok_and(|found| found == WANT) => {}
            Some(v) => problems.push(
                Coded::new(AcceptsCode::ContractVersion)
                    .with_text(keys::FOUND, v)
                    .with_text(keys::WANT, CONTRACT_VERSION),
            ),
            None => problems.push(
                Coded::new(AcceptsCode::FileInvalid)
                    .with_text(keys::FILE, "contract.toml")
                    .with_text(keys::FIELD, "version"),
            ),
        }
        let mut blocks: BTreeMap<String, BlockSpec> = BTreeMap::new();
        for (file, text) in files {
            match read_blocks(file, text) {
                Err(mut list) => problems.append(&mut list),
                Ok(specs) => {
                    for spec in specs {
                        if blocks.contains_key(&spec.name) {
                            problems.push(
                                Coded::new(AcceptsCode::BlockTwice)
                                    .with_text(keys::BLOCK, spec.name.as_str())
                                    .with_text(keys::FILE, *file),
                            );
                        } else {
                            blocks.insert(spec.name.clone(), spec);
                        }
                    }
                }
            }
        }
        problems.extend(super::choose::twice(blocks.values()));
        if problems.is_empty() {
            Ok(Self { blocks })
        } else {
            Err(problems)
        }
    }

    /// The block with this name.
    #[must_use]
    pub fn block(&self, name: &str) -> Option<&BlockSpec> {
        self.blocks.get(name)
    }

    /// The block, or `block-unknown`.
    ///
    /// # Errors
    /// `block-unknown` with the name.
    pub fn need(&self, name: &str) -> Result<&BlockSpec, Coded<AcceptsCode>> {
        self.block(name)
            .ok_or_else(|| Coded::new(AcceptsCode::BlockUnknown).with_text(keys::BLOCK, name))
    }

    /// How many blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether there is none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// The blocks in name order.
    pub fn iter(&self) -> impl Iterator<Item = &BlockSpec> {
        self.blocks.values()
    }
}

/// `[contract] version` of `contract.toml`.
fn contract_version(text: &str) -> Option<String> {
    let table: Table = text.parse().ok()?;
    Some(
        table
            .get("contract")?
            .as_table()?
            .get("version")?
            .as_str()?
            .to_owned(),
    )
}

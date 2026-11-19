//! What a building block declares: one [`Declaration`] per setting, gathered in a [`Catalog`].
//!
//! Pattern: Builder (`Declaration::new(..).per_device()`) + Registry (`Catalog`, refuses duplicates and defaults
//! that do not fit) + Strategy (`Declares`, implemented by every block). The words (name and description, German and
//! English) are NOT in the declaration: they are MF2 texts of the block under [`Declaration::label_key`] and
//! [`Declaration::about_key`] (ADR 0057; the suite rule `settings.texts-both` checks both languages).

use std::collections::BTreeMap;

use fs_types::coded::Coded;

use crate::codes::{SettingCode, keys};
use crate::names::{GroupId, SettingKey};
use crate::value::{Kind, Value};

/// Where a key keeps its profile values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Place {
    /// In the profile files (`$XDG_CONFIG_HOME`), the same on every device of the person once profiles travel.
    #[default]
    Profile,
    /// In the device files (`$XDG_STATE_HOME`): this machine only (scale, a screen-dependent size).
    Device,
}

/// One setting of a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    /// The key.
    pub key: SettingKey,
    /// The kind and its limits.
    pub kind: Kind,
    /// The block's default (the lowest layer). Checked against `kind` by [`Catalog::add`].
    pub default: Value,
    /// The group it is shown in.
    pub group: GroupId,
    /// Where its profile values are kept.
    pub place: Place,
}

impl Declaration {
    /// A setting in `group`, kept in the profile. Every setting can be locked; who may, say the rights (`lock.rs`).
    #[must_use]
    pub fn new(key: SettingKey, kind: Kind, default: Value, group: GroupId) -> Self {
        Self {
            key,
            kind,
            default,
            group,
            place: Place::Profile,
        }
    }

    /// Kept per device.
    #[must_use]
    pub fn per_device(mut self) -> Self {
        self.place = Place::Device;
        self
    }

    /// The MF2 key of its name: `setting-label-` and the key with `.` as `-` (`setting-label-fs-shell-language`).
    #[must_use]
    pub fn label_key(&self) -> String {
        format!("setting-label-{}", self.key.as_str().replace('.', "-"))
    }

    /// The MF2 key of its description: `setting-about-` and the key with `.` as `-`.
    #[must_use]
    pub fn about_key(&self) -> String {
        format!("setting-about-{}", self.key.as_str().replace('.', "-"))
    }
}

/// All settings a program knows: its own and those of every block it is made of.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    all: BTreeMap<SettingKey, Declaration>,
}

impl Catalog {
    /// An empty catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `d`.
    ///
    /// # Errors
    /// `duplicate-key` when the key is there already; `default-invalid` when the default does not fit the kind.
    pub fn add(&mut self, d: Declaration) -> Result<&mut Self, Coded<SettingCode>> {
        let with_key = |code: SettingCode| Coded::new(code).with_text(keys::KEY, d.key.as_str());
        if self.all.contains_key(&d.key) {
            return Err(with_key(SettingCode::DuplicateKey));
        }
        if !d.kind.fits(&d.key, &d.default) {
            return Err(with_key(SettingCode::DefaultInvalid));
        }
        self.all.insert(d.key.clone(), d);
        Ok(self)
    }

    /// A catalog of `list`, in order; the first failure stops.
    ///
    /// # Errors
    /// As [`Catalog::add`].
    pub fn of(list: Vec<Declaration>) -> Result<Self, Coded<SettingCode>> {
        let mut c = Self::new();
        for d in list {
            c.add(d)?;
        }
        Ok(c)
    }

    /// The declaration of `key`.
    #[must_use]
    pub fn get(&self, key: &SettingKey) -> Option<&Declaration> {
        self.all.get(key)
    }

    /// Every declaration, ordered by key.
    pub fn iter(&self) -> impl Iterator<Item = &Declaration> {
        self.all.values()
    }
}

/// A building block that has settings. Each block declares only its own keys (owner = itself).
pub trait Declares {
    /// The block's declarations, in the order its settings page shows them.
    ///
    /// # Errors
    /// `key-invalid` when the block builds a bad key (a bug of the block, caught by its own K-test).
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>>;
}

//! The settings of a package as data: `catalog/settings.json`, read into the [`Declaration`]s of the existing [`Catalog`].
//!
//! Pattern: Registry + Adapter. The file is the adapter onto [`Declares`]; [`Catalog`] stays the registry. It is NOT a
//! free plugin registry: the reader knows only the closed list of [`Kind`]s; what it does not know it refuses (serde
//! `deny_unknown_fields` on every level), it never takes it silently. The `schema` of an entry is a real subset of JSON
//! Schema 2020-12 (`type`, `enum`, `default`, `minimum`, `maximum`, `maxLength`, plus the own `format: "bcp47"`).
//!
//! The file can only DESCRIBE a setting. The order of the layers stays in [`crate::resolve`], and who may lock stays a
//! right of fs-access: the file has no field for either, so a package cannot give itself one (such a field is refused).
//! The words are not in the file: [`Declaration::label_key`], [`Declaration::about_key`], [`Declaration::choice_key`].

use fs_types::coded::Coded;
use fs_types::{BasicValue, Language};
use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Value as Json, error::Category};

use crate::codes::{SettingCode, keys};
use crate::declare::{Catalog, Declaration, Declares};
use crate::names::{GroupId, Owner, OwnerKind, SettingKey};
use crate::value::{Kind, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRaw {
    version: String,
    settings: Vec<EntryRaw>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryRaw {
    owner: String,
    key: String,
    schema: SchemaRaw,
    group: String,
    #[serde(default)]
    place: PlaceRaw,
    #[serde(default, rename = "human-only")]
    human_only: bool,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum PlaceRaw {
    #[default]
    Profile,
    Device,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SchemaRaw {
    #[serde(rename = "type")]
    ty: String,
    #[serde(rename = "enum")]
    choices: Option<Vec<String>>,
    format: Option<String>,
    minimum: Option<i64>,
    maximum: Option<i64>,
    #[serde(rename = "maxLength")]
    max_length: Option<u32>,
    default: Json,
}

/// The declarations of one package file, checked as a [`Catalog`] is (no duplicate key, every default fits its kind).
/// Every entry names its own owner (area): one package may declare several (`fs.open`, `fs.rpc`, `fs.mcp`).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogFile {
    file: String,
    owners: Vec<Owner>,
    list: Vec<Declaration>,
}

/// Keys that are safety switches: a file may declare them only as a flag, off by default, and `human-only`. A file can
/// therefore never switch one on (`fs.mcp.enabled`, "MCP only a person", ADR 0054).
const PROTECTED: &[&str] = &["fs.mcp.enabled"];

impl CatalogFile {
    /// Reads `text`; `file` names the file (the package) in a failure and in [`CatalogFile::gather`].
    ///
    /// # Errors
    /// `catalog-invalid` (`file`, `cause` = `json`: no JSON; `field`: unknown, missing or mistyped field; `owner`: an
    /// entry has no owner, or a key outside its own; `schema`: no `schema` of the closed list; `protected`: a safety
    /// switch that is not off, flag, human-only and `place: device`), `key-invalid`, `default-invalid`, `duplicate-key`.
    pub fn parse(text: &str, file: &str) -> Result<Self, Coded<SettingCode>> {
        let bad = |cause: &str| {
            Coded::new(SettingCode::CatalogInvalid)
                .with_text(keys::FILE, file)
                .with_text(keys::CAUSE, cause)
        };
        let raw: FileRaw = serde_json::from_str(text).map_err(|e| match e.classify() {
            Category::Data => bad("field"),
            _ => bad("json"),
        })?;
        if raw.version != "0.1" {
            return Err(bad("field"));
        }
        let mut owners = Vec::new();
        let mut list = Vec::with_capacity(raw.settings.len());
        for e in raw.settings {
            let owner = read_owner(&e.owner).ok_or_else(|| bad("owner"))?;
            let key = SettingKey::parse(&e.key)?;
            if !e
                .key
                .starts_with(&format!("{}.{}.", owner.kind().prefix(), owner.name()))
            {
                return Err(bad("owner").with_text(keys::KEY, e.key));
            }
            let (kind, default) = read_schema(&e.schema).map_err(|c| match c {
                SchemaProblem::Schema => bad("schema").with_text(keys::KEY, key.as_str()),
                SchemaProblem::Default => {
                    Coded::new(SettingCode::DefaultInvalid).with_text(keys::KEY, key.as_str())
                }
            })?;
            if PROTECTED.contains(&key.as_str())
                && !(kind == Kind::Flag
                    && default == Value::Flag(false)
                    && e.human_only
                    && matches!(e.place, PlaceRaw::Device))
            {
                return Err(bad("protected").with_text(keys::KEY, key.as_str()));
            }
            let mut d = Declaration::new(key, kind, default, GroupId::new(&e.group)?);
            if matches!(e.place, PlaceRaw::Device) {
                d = d.per_device();
            }
            list.push(if e.human_only { d.human_only() } else { d });
            if !owners.contains(&owner) {
                owners.push(owner);
            }
        }
        Catalog::of(list.clone())?;
        owners.sort();
        Ok(Self {
            file: file.to_owned(),
            owners,
            list,
        })
    }

    /// The owners (areas) named by the entries, sorted, each once.
    #[must_use]
    pub fn owners(&self) -> &[Owner] {
        &self.owners
    }

    /// The name of the file (package) it was read under.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// One [`Catalog`] of the files of several packages. The same key in two of them is `catalog-clash` with the names
    /// of BOTH files (`key`, `first`, `second`): nothing is overwritten, no file wins.
    ///
    /// # Errors
    /// `catalog-clash`; `default-invalid` as [`Catalog::add`].
    pub fn gather(files: &[&CatalogFile]) -> Result<Catalog, Coded<SettingCode>> {
        let mut all = Catalog::new();
        let mut from: BTreeMap<SettingKey, &str> = BTreeMap::new();
        for f in files {
            for d in &f.list {
                if let Some(first) = from.get(&d.key) {
                    return Err(Coded::new(SettingCode::CatalogClash)
                        .with_text(keys::KEY, d.key.as_str())
                        .with_text(keys::FIRST, *first)
                        .with_text(keys::SECOND, f.file.as_str()));
                }
                from.insert(d.key.clone(), f.file.as_str());
                all.add(d.clone())?;
            }
        }
        Ok(all)
    }
}

/// `fs.open` as an [`Owner`]; anything that is no `<prefix>.<name>` of the closed list is `None`.
fn read_owner(text: &str) -> Option<Owner> {
    let (prefix, name) = text.split_once('.')?;
    let kind = OwnerKind::ALL
        .iter()
        .copied()
        .find(|k| k.prefix() == prefix)?;
    Owner::new(kind, name).ok()
}

impl Declares for CatalogFile {
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>> {
        Ok(self.list.clone())
    }
}

impl Declaration {
    /// The MF2 key of the name of one choice: `setting-choice-`, the key with `.` as `-`, `-`, the choice
    /// (`setting-choice-fs-open-when-open-tab`).
    #[must_use]
    pub fn choice_key(&self, choice: &str) -> String {
        format!(
            "setting-choice-{}-{}",
            self.key.as_str().replace('.', "-"),
            choice.replace('.', "-")
        )
    }
}

enum SchemaProblem {
    Schema,
    Default,
}

/// The closed mapping of a `schema` onto a [`Kind`] and its default; any other shape is `Schema`.
fn read_schema(s: &SchemaRaw) -> Result<(Kind, Value), SchemaProblem> {
    use SchemaProblem::{Default as BadDefault, Schema};
    let only = |enum_: bool, format: bool, range: bool, len: bool| {
        (s.choices.is_some(), s.format.is_some()) == (enum_, format)
            && (s.minimum.is_some() || s.maximum.is_some()) == range
            && s.max_length.is_some() == len
    };
    let (kind, default) = match s.ty.as_str() {
        "boolean" if only(false, false, false, false) => {
            (Kind::Flag, s.default.as_bool().map(Value::Flag))
        }
        "integer" if only(false, false, true, false) => {
            let (Some(min), Some(max)) = (s.minimum, s.maximum) else {
                return Err(Schema);
            };
            (
                Kind::Whole { min, max },
                s.default.as_i64().map(Value::Whole),
            )
        }
        "string" if only(true, false, false, false) => {
            let list = s.choices.clone().ok_or(Schema)?;
            let d = s.default.as_str().map(|t| Value::Choice(t.to_owned()));
            (Kind::Choice { keys: list }, d)
        }
        "string" if only(false, true, false, false) && s.format.as_deref() == Some("bcp47") => {
            let d = s.default.as_str().and_then(|t| Language::parse(t).ok());
            (Kind::Language, d.map(Value::Language))
        }
        "string" if only(false, false, false, true) => {
            let max_chars = s.max_length.ok_or(Schema)?;
            let d = s.default.as_str().map(|t| Value::Text(t.to_owned()));
            (Kind::Text { max_chars }, d)
        }
        _ => return Err(Schema),
    };
    default.map(|d| (kind, d)).ok_or(BadDefault)
}

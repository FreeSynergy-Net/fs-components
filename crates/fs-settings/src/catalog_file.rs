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
use serde::Deserialize;
use serde_json::{Value as Json, error::Category};

use crate::codes::{SettingCode, keys};
use crate::declare::{Catalog, Declaration, Declares};
use crate::names::{GroupId, Owner, OwnerKind, SettingKey};
use crate::value::{Kind, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRaw {
    owner: String,
    version: String,
    settings: Vec<EntryRaw>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryRaw {
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
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogFile {
    owner: Owner,
    list: Vec<Declaration>,
}

impl CatalogFile {
    /// Reads `text`; `file` only names the file in a failure.
    ///
    /// # Errors
    /// `catalog-invalid` (`file`, `cause` = `json`: no JSON; `field`: unknown, missing or mistyped field; `owner`: no
    /// owner, or a key outside it; `schema`: no `schema` of the closed list), `key-invalid`, `default-invalid`,
    /// `duplicate-key`.
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
        let (prefix, name) = raw.owner.split_once('.').ok_or_else(|| bad("owner"))?;
        let kind = OwnerKind::ALL
            .iter()
            .copied()
            .find(|k| k.prefix() == prefix);
        let owner = kind
            .ok_or_else(|| bad("owner"))
            .and_then(|k| Owner::new(k, name).map_err(|_| bad("owner")))?;
        let mut list = Vec::with_capacity(raw.settings.len());
        for e in raw.settings {
            let key = SettingKey::parse(&e.key)?;
            if !e.key.starts_with(&format!("{}.{}.", prefix, owner.name())) {
                return Err(bad("owner").with_text(keys::KEY, e.key));
            }
            let (kind, default) = read_schema(&e.schema).map_err(|c| match c {
                SchemaProblem::Schema => bad("schema").with_text(keys::KEY, key.as_str()),
                SchemaProblem::Default => {
                    Coded::new(SettingCode::DefaultInvalid).with_text(keys::KEY, key.as_str())
                }
            })?;
            let mut d = Declaration::new(key, kind, default, GroupId::new(&e.group)?);
            if matches!(e.place, PlaceRaw::Device) {
                d = d.per_device();
            }
            list.push(if e.human_only { d.human_only() } else { d });
        }
        Catalog::of(list.clone())?;
        Ok(Self { owner, list })
    }

    /// The owner named in the file.
    #[must_use]
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
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

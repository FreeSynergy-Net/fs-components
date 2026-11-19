//! What a setting may hold: its kind (declared), its value (checked), and the primitive form a store keeps.
//!
//! Pattern: Value objects + one checking function per kind ([`Kind::read`]). The kinds are a closed list; a new kind is a
//! new variant (draft O12: fs-types `BasicType` has no yes/no and no whole number yet, so this list is the crate's own).

use fs_types::coded::Coded;
use fs_types::{BasicValue, Language};

use crate::codes::{SettingCode, keys};
use crate::layer::Layer;
use crate::names::SettingKey;

/// The kind of a setting, with its limits.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Yes or no.
    Flag,
    /// A whole number in `min..=max`.
    Whole {
        /// The smallest allowed.
        min: i64,
        /// The largest allowed.
        max: i64,
    },
    /// One key of a list (style ids, `light`/`dark`). The list belongs to the declaring block.
    Choice {
        /// The allowed keys, each a word.
        keys: Vec<String>,
    },
    /// A BCP 47 language tag (RFC 5646), normalised by fs-types `Language`.
    Language,
    /// One line of text of at most `max_chars` characters. Never a secret (lib rules).
    Text {
        /// The most characters allowed.
        max_chars: u32,
    },
}

/// A checked value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// Yes or no.
    Flag(bool),
    /// A whole number.
    Whole(i64),
    /// A key of the list.
    Choice(String),
    /// A language.
    Language(Language),
    /// A text.
    Text(String),
}

/// The primitive form a store keeps (ADR 0086: primitive types, no JSON, no blob). TOML: boolean, integer, string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Raw {
    /// A boolean.
    Bool(bool),
    /// An integer.
    Int(i64),
    /// A string.
    Str(String),
}

impl Value {
    /// The primitive form.
    #[must_use]
    pub fn to_raw(&self) -> Raw {
        match self {
            Self::Flag(b) => Raw::Bool(*b),
            Self::Whole(n) => Raw::Int(*n),
            Self::Choice(s) | Self::Text(s) => Raw::Str(s.clone()),
            Self::Language(l) => Raw::Str(l.canonical()),
        }
    }
}

impl Kind {
    /// Reads `raw` as a value of this kind. `layer` only names where it came from in a failure.
    ///
    /// # Errors
    /// `wrong-type`, `out-of-range`, `unknown-choice`, `bad-language`, `text-too-long` with `key` and `layer`.
    pub fn read(
        &self,
        key: &SettingKey,
        layer: Layer,
        raw: &Raw,
    ) -> Result<Value, Coded<SettingCode>> {
        let fail = |code: SettingCode| {
            Coded::new(code)
                .with_text(keys::KEY, key.as_str())
                .with_text(keys::LAYER, layer.as_str())
        };
        match (self, raw) {
            (Self::Flag, Raw::Bool(b)) => Ok(Value::Flag(*b)),
            (Self::Whole { min, max }, Raw::Int(n)) => {
                if n < min || n > max {
                    Err(fail(SettingCode::OutOfRange))
                } else {
                    Ok(Value::Whole(*n))
                }
            }
            (Self::Choice { keys: list }, Raw::Str(s)) => {
                if list.iter().any(|k| k == s) {
                    Ok(Value::Choice(s.clone()))
                } else {
                    Err(fail(SettingCode::UnknownChoice))
                }
            }
            (Self::Language, Raw::Str(s)) => Language::parse(s)
                .map(Value::Language)
                .map_err(|_| fail(SettingCode::BadLanguage)),
            (Self::Text { max_chars }, Raw::Str(s)) => {
                let fits = u32::try_from(s.chars().count()).is_ok_and(|n| n <= *max_chars);
                if fits && !s.contains(['\n', '\r']) {
                    Ok(Value::Text(s.clone()))
                } else {
                    Err(fail(SettingCode::TextTooLong))
                }
            }
            _ => Err(fail(SettingCode::WrongType)),
        }
    }

    /// Whether `value` fits this kind (the same rules as [`Kind::read`] of its raw form).
    #[must_use]
    pub fn fits(&self, key: &SettingKey, value: &Value) -> bool {
        self.read(key, Layer::Default, &value.to_raw())
            .is_ok_and(|v| &v == value)
    }
}

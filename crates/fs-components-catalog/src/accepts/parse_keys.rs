//! The keys of one slot whose rules depend on its kind: `basic`, `cardinality`, `within`.
//!
//! Pattern: small parsers; `None` = the key is wrong for this kind.

use toml::Value;

use super::slot::{BasicSet, Cardinality, SlotKind, basic_from_key, name_ok};

/// `basic`: a value slot needs a non-empty list of basic type words or exactly `["any"]`; other kinds have none.
pub(super) fn read_basic(value: Option<&Value>, is_value: bool) -> Option<BasicSet> {
    let words: Vec<&str> = match value {
        None => Vec::new(),
        Some(v) => v
            .as_array()?
            .iter()
            .map(Value::as_str)
            .collect::<Option<_>>()?,
    };
    match (is_value, words.as_slice()) {
        (false, []) => Some(BasicSet::Only(Vec::new())),
        (false, _) | (true, []) => None,
        (true, ["any"]) => Some(BasicSet::Any),
        (true, list) => {
            let mut out = Vec::new();
            for w in list {
                let b = basic_from_key(w)?;
                if out.contains(&b) {
                    return None;
                }
                out.push(b);
            }
            Some(BasicSet::Only(out))
        }
    }
}

/// `cardinality`: default `multiple` for records, `single` otherwise; only value and records slots may say `multiple`.
pub(super) fn read_cardinality(
    value: Option<&Value>,
    kind: Option<SlotKind>,
) -> Option<Cardinality> {
    let may_many = matches!(kind, Some(SlotKind::Value | SlotKind::Records));
    match value.map(Value::as_str) {
        None if kind == Some(SlotKind::Records) => Some(Cardinality::Multiple),
        None | Some(Some("single")) => Some(Cardinality::Single),
        Some(Some("multiple")) if may_many => Some(Cardinality::Multiple),
        _ => None,
    }
}

/// `within`: required for value, grouping, order and lock slots (they belong to records), optional for a command, not allowed
/// for records (no list inside a list in v1). `Some(None)` = rightly absent.
// Three answers on purpose: refused (`None`), no `within` (`Some(None)`), a `within` (`Some(Some(name))`).
#[allow(clippy::option_option)]
pub(super) fn read_within(value: Option<&Value>, kind: Option<SlotKind>) -> Option<Option<String>> {
    let text = match value {
        None => None,
        Some(v) => Some(v.as_str().filter(|s| name_ok(s))?.to_owned()),
    };
    match (kind?, text) {
        (SlotKind::Records | SlotKind::Command, None) => Some(None),
        (SlotKind::Records, Some(_)) | (_, None) => None,
        (_, Some(t)) => Some(Some(t)),
    }
}

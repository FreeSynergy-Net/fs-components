//! The keys of one slot whose rules depend on its kind: `basic`, `cardinality`, `within`, `heads`.
//!
//! Pattern: small parsers; `None` = the key is wrong for this kind.

use toml::Value;

use super::forms::FormRef;
use super::slot::{basic_from_key, name_ok, BasicSet, Cardinality, SlotKind};

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

/// `heads`: only a grouping slot may have it, a non-empty list of full form needs (`rows@^1.0.0`), each form once. Absent = no
/// heads (`Some(vec![])`); `None` = the key is wrong here (other kind, empty list, bad entry, a form twice).
pub(super) fn read_heads(value: Option<&Value>, kind: Option<SlotKind>) -> Option<Vec<FormRef>> {
    let Some(value) = value else {
        return Some(Vec::new());
    };
    if kind? != SlotKind::Grouping {
        return None;
    }
    let mut out: Vec<FormRef> = Vec::new();
    for entry in value.as_array()? {
        let form = FormRef::parse(entry.as_str()?)?;
        if out.iter().any(|o| o.name() == form.name()) {
            return None;
        }
        out.push(form);
    }
    (!out.is_empty()).then_some(out)
}

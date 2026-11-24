//! The reader of the layout keys of a block (`places`, `user`, `sizing`, `reads`, `sets`, `defaults`); the types are in `layout`.
//!
//! Pattern: Builder (TOML table in, [`Layout`] out), by hand so that every problem names its key.

use std::collections::BTreeMap;

use toml::{Table, Value};

use super::codes::{AcceptsCode, keys};
use super::layout::{Layout, Max, PlaceOrder, PlaceSpec, Sizing, Takes, UserAct};
use super::slot::name_ok;
use super::vocab::Vocabulary;
use fs_types::coded::Coded;

const PLACE_KEYS: [&str; 5] = ["place", "takes", "min", "max", "order"];

fn word<T: Copy>(list: &[(&str, T)], w: &str) -> Option<T> {
    list.iter().find(|(k, _)| *k == w).map(|(_, v)| *v)
}

const TAKES: [(&str, Takes); 5] = [
    ("command", Takes::Command),
    ("form", Takes::Form),
    ("frame", Takes::Frame),
    ("value", Takes::Value),
    ("any", Takes::Any),
];
const ORDERS: [(&str, PlaceOrder); 2] =
    [("layout", PlaceOrder::Layout), ("user", PlaceOrder::User)];
const ACTS: [(&str, UserAct); 5] = [
    ("resize", UserAct::Resize),
    ("reorder", UserAct::Reorder),
    ("move", UserAct::Move),
    ("close", UserAct::Close),
    ("float", UserAct::Float),
];
const SIZINGS: [(&str, Sizing); 2] = [("content", Sizing::Content), ("fill", Sizing::Fill)];

fn layout_bad(block: &str, field: &str) -> Coded<AcceptsCode> {
    Coded::new(AcceptsCode::LayoutInvalid)
        .with_text(keys::BLOCK, block)
        .with_text(keys::FIELD, field)
}

fn place_bad(block: &str, place: &str, field: &str) -> Coded<AcceptsCode> {
    Coded::new(AcceptsCode::PlaceInvalid)
        .with_text(keys::BLOCK, block)
        .with_text(keys::PLACE, place)
        .with_text(keys::FIELD, field)
}

fn attr_bad(block: &str, field: &str, name: &str) -> Coded<AcceptsCode> {
    Coded::new(AcceptsCode::AttrUnknown)
        .with_text(keys::BLOCK, block)
        .with_text(keys::FIELD, field)
        .with_text(keys::NAME, name)
}

/// The layout keys of the block `name` (`body` = its table); every problem is pushed.
pub(super) fn read_layout(
    name: &str,
    body: &Table,
    vocab: &Vocabulary,
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Layout {
    let parts = strings(body.get("parts")).unwrap_or_default();
    let sets = attrs(name, "sets", body, |w| vocab.has_set(w), problems);
    let mut out = Layout {
        places: read_places(name, body.get("places"), &parts, problems),
        reads: attrs(name, "reads", body, |w| vocab.has_read(w), problems),
        defaults: BTreeMap::new(),
        sets,
        ..Layout::default()
    };
    if let Some(v) = body.get("user") {
        match strings(Some(v)).and_then(|l| l.iter().map(|w| word(&ACTS, w)).collect()) {
            Some(list) => out.user = list,
            None => problems.push(layout_bad(name, "user")),
        }
    }
    if let Some(v) = body.get("sizing") {
        match v.as_str().and_then(|w| word(&SIZINGS, w)) {
            Some(s) => out.sizing = Some(s),
            None => problems.push(layout_bad(name, "sizing")),
        }
    }
    if let Some(v) = body.get("defaults") {
        read_defaults(name, v, &mut out, problems);
    }
    out
}

fn strings(v: Option<&Value>) -> Option<Vec<String>> {
    v?.as_array()?
        .iter()
        .map(|w| w.as_str().map(str::to_owned))
        .collect()
}

/// `reads` or `sets`: a list of words, each of them in the vocabulary, none twice.
fn attrs(
    block: &str,
    field: &str,
    body: &Table,
    known: impl Fn(&str) -> bool,
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Vec<String> {
    let Some(v) = body.get(field) else {
        return Vec::new();
    };
    let Some(list) = strings(Some(v)) else {
        problems.push(layout_bad(block, field));
        return Vec::new();
    };
    for (i, w) in list.iter().enumerate() {
        if !known(w) || list[..i].contains(w) {
            problems.push(attr_bad(block, field, w));
        }
    }
    list
}

fn read_defaults(block: &str, v: &Value, out: &mut Layout, problems: &mut Vec<Coded<AcceptsCode>>) {
    let Some(t) = v.as_table() else {
        problems.push(layout_bad(block, "defaults"));
        return;
    };
    for (k, val) in t {
        // A default belongs to an attribute the block sets.
        if !out.sets.contains(k) {
            problems.push(attr_bad(block, "defaults", k));
        }
        match val.as_str() {
            Some(s) => {
                out.defaults.insert(k.clone(), s.to_owned());
            }
            None => problems.push(layout_bad(block, "defaults")),
        }
    }
}

fn read_places(
    block: &str,
    v: Option<&Value>,
    parts: &[String],
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Vec<PlaceSpec> {
    let Some(v) = v else { return Vec::new() };
    let Some(list) = v.as_array() else {
        problems.push(layout_bad(block, "places"));
        return Vec::new();
    };
    let mut out: Vec<PlaceSpec> = Vec::new();
    for entry in list {
        let Some(t) = entry.as_table() else {
            problems.push(layout_bad(block, "places"));
            continue;
        };
        if let Some(p) = read_place(block, t, parts, problems) {
            if out.iter().any(|o| o.place == p.place) {
                problems.push(place_bad(block, &p.place, "place"));
            } else {
                out.push(p);
            }
        }
    }
    out
}

fn read_place(
    block: &str,
    t: &Table,
    parts: &[String],
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Option<PlaceSpec> {
    let place = t.get("place").and_then(Value::as_str).unwrap_or("");
    let mut wrong: Vec<&str> = t
        .keys()
        .filter(|k| !PLACE_KEYS.contains(&k.as_str()))
        .map(String::as_str)
        .collect();
    if !name_ok(place) {
        wrong.push("place");
    } else if !parts.iter().any(|p| p == place) {
        problems.push(
            Coded::new(AcceptsCode::PlaceNotPart)
                .with_text(keys::BLOCK, block)
                .with_text(keys::PLACE, place),
        );
    }
    let takes: Option<Vec<Takes>> = strings(t.get("takes"))
        .filter(|l| !l.is_empty())
        .and_then(|l| l.iter().map(|w| word(&TAKES, w)).collect());
    if takes.is_none() {
        wrong.push("takes");
    }
    let min = t
        .get("min")
        .and_then(Value::as_integer)
        .and_then(|n| u32::try_from(n).ok());
    let max = match t.get("max") {
        Some(Value::String(s)) if s == "n" => Some(Max::Many),
        Some(Value::Integer(n)) => u32::try_from(*n).ok().filter(|n| *n >= 1).map(Max::Count),
        _ => None,
    };
    if min.is_none() {
        wrong.push("min");
    }
    if max.is_none() {
        wrong.push("max");
    }
    if let (Some(min), Some(Max::Count(max))) = (min, max)
        && min > max
    {
        wrong.push("min");
    }
    let order = match t.get("order") {
        None => Some(None),
        Some(o) => o.as_str().and_then(|w| word(&ORDERS, w)).map(Some),
    };
    if order.is_none() {
        wrong.push("order");
    }
    if !wrong.is_empty() {
        problems.extend(wrong.iter().map(|f| place_bad(block, place, f)));
        return None;
    }
    Some(PlaceSpec {
        place: place.to_owned(),
        takes: takes?,
        min: min?,
        max: max?,
        order: order?,
    })
}

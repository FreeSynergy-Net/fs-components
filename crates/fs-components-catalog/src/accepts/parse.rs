//! Reads the key `accepts` of every block in one contract file. Every other key of a block (names, parts, states) belongs to
//! the Python check `contract/check` and is not read here.
//!
//! The schema of one slot is closed (draft §4.1): `slot`, `kind`, `basic`, `role`, `cardinality`, `required`, `within`,
//! `repeat`; an
//! unknown key, an unknown word or a key that does not fit the kind is a problem with the key as parameter `field`. All problems
//! of a file are returned, never only the first.
//!
//! Pattern: Builder (TOML table in, [`BlockSpec`]s out); walked by hand over `toml::Table` so that every problem names its key.

use toml::{Table, Value};

use super::catalog::BlockSpec;
use super::codes::{AcceptsCode, keys};
use super::parse_keys::{read_basic, read_cardinality, read_within};
use super::slot::{Role, Slot, SlotKind, name_ok};
use fs_types::coded::Coded;

/// The keys a slot may have.
const SLOT_KEYS: [&str; 8] = [
    "slot",
    "kind",
    "basic",
    "role",
    "cardinality",
    "required",
    "within",
    "repeat",
];

/// The blocks of one file (`file` = its name for the problems), each with its slots.
///
/// # Errors
/// Every problem of the file: `file-invalid`, `accepts-missing`, `slot-invalid`, `slot-twice`.
pub fn read_blocks(file: &str, text: &str) -> Result<Vec<BlockSpec>, Vec<Coded<AcceptsCode>>> {
    let table: Table = text.parse().map_err(|_| vec![file_bad(file, "toml")])?;
    let Some(blocks) = table.get("block") else {
        return Ok(Vec::new());
    };
    let Some(blocks) = blocks.as_table() else {
        return Err(vec![file_bad(file, "block")]);
    };
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for (name, body) in blocks {
        let Some(body) = body.as_table() else {
            problems.push(file_bad(file, name));
            continue;
        };
        let Some(accepts) = body.get("accepts") else {
            problems.push(
                Coded::new(AcceptsCode::AcceptsMissing).with_text(keys::BLOCK, name.as_str()),
            );
            continue;
        };
        if let Some(slots) = read_accepts(name, accepts, &mut problems) {
            out.push(BlockSpec {
                name: name.clone(),
                accepts: slots,
            });
        }
    }
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems)
    }
}

fn file_bad(file: &str, field: &str) -> Coded<AcceptsCode> {
    Coded::new(AcceptsCode::FileInvalid)
        .with_text(keys::FILE, file)
        .with_text(keys::FIELD, field)
}

fn slot_bad(block: &str, slot: &str, field: &str) -> Coded<AcceptsCode> {
    Coded::new(AcceptsCode::SlotInvalid)
        .with_text(keys::BLOCK, block)
        .with_text(keys::SLOT, slot)
        .with_text(keys::FIELD, field)
}

/// The slots of one block, or `None` when any of them is wrong (the problems are pushed).
fn read_accepts(
    block: &str,
    value: &Value,
    problems: &mut Vec<Coded<AcceptsCode>>,
) -> Option<Vec<Slot>> {
    let before = problems.len();
    let Some(list) = value.as_array() else {
        problems.push(slot_bad(block, "", "accepts"));
        return None;
    };
    let mut slots: Vec<Slot> = Vec::new();
    for entry in list {
        let Some(table) = entry.as_table() else {
            problems.push(slot_bad(block, "", "accepts"));
            continue;
        };
        let Some(slot) = read_slot(block, table, problems) else {
            continue;
        };
        if slots.iter().any(|s| s.name == slot.name) {
            problems.push(
                Coded::new(AcceptsCode::SlotTwice)
                    .with_text(keys::BLOCK, block)
                    .with_text(keys::SLOT, slot.name),
            );
        } else {
            slots.push(slot);
        }
    }
    for slot in &slots {
        let Some(parent) = &slot.within else { continue };
        let fits = slots
            .iter()
            .any(|s| &s.name == parent && s.kind == SlotKind::Records);
        if !fits {
            problems.push(slot_bad(block, &slot.name, "within"));
        }
    }
    (problems.len() == before).then_some(slots)
}

/// One slot; every wrong key is pushed as its own problem.
fn read_slot(block: &str, t: &Table, problems: &mut Vec<Coded<AcceptsCode>>) -> Option<Slot> {
    let name = t.get("slot").and_then(Value::as_str).unwrap_or("");
    let mut wrong: Vec<String> = t
        .keys()
        .filter(|k| !SLOT_KEYS.contains(&k.as_str()))
        .cloned()
        .collect();
    if !name_ok(name) {
        wrong.push("slot".into());
    }
    let kind = t
        .get("kind")
        .and_then(Value::as_str)
        .and_then(SlotKind::from_key);
    if kind.is_none() {
        // The other keys depend on the kind: without it they are not judged (one problem, not five).
        wrong.push("kind".into());
        problems.extend(wrong.iter().map(|f| slot_bad(block, name, f)));
        return None;
    }
    let is_value = kind == Some(SlotKind::Value);
    let basic = read_basic(t.get("basic"), is_value);
    if basic.is_none() {
        wrong.push("basic".into());
    }
    let role = match (t.get("role"), is_value) {
        (None, false) => Some(None),
        (Some(r), true) => r.as_str().and_then(Role::from_key).map(Some),
        _ => None,
    };
    if role.is_none() {
        wrong.push("role".into());
    }
    let cardinality = read_cardinality(t.get("cardinality"), kind);
    if cardinality.is_none() {
        wrong.push("cardinality".into());
    }
    let required = match t.get("required") {
        None => Some(false),
        Some(v) => v.as_bool(),
    };
    if required.is_none() {
        wrong.push("required".into());
    }
    let within = read_within(t.get("within"), kind);
    if within.is_none() {
        wrong.push("within".into());
    }
    let repeat = match t.get("repeat") {
        None => Some(1),
        Some(v) => v
            .as_integer()
            .filter(|n| (1..=16).contains(n))
            .and_then(|n| u8::try_from(n).ok()),
    };
    if repeat.is_none() {
        wrong.push("repeat".into());
    }
    if !wrong.is_empty() {
        problems.extend(wrong.iter().map(|f| slot_bad(block, name, f)));
        return None;
    }
    Some(Slot {
        name: name.to_owned(),
        kind: kind?,
        basic: basic?,
        role: role?,
        cardinality: cardinality?,
        required: required?,
        within: within?,
        repeat: repeat?,
    })
}

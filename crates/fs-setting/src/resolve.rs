//! The effective value of one setting: the only place that walks the layers.
//!
//! Order (draft §4.3): a lock of the instance wins over every layer ("gesperrt schlägt jede Schicht"); else the highest
//! layer that holds a value that fits the declaration: active named profile (program part > general part; it inherits
//! what it lacks; Nachzug 17.11.) > program profile > general profile > system > instance default > block default. A
//! value that does not fit is a problem and the walk goes on downwards; a lock without a value that fits is the
//! problem `lock-refused` and holds nothing. A lock holds in EVERY profile. Values of the person under a lock stay stored and count again after the unlock.
//!
//! Pattern: pure function (no store, no clock); [`crate::Settings`] calls it for every key after each read.

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::declare::Declaration;
use crate::layer::Layer;
use crate::lock::Lock;
use crate::store::{Instance, Snapshot};
use crate::value::{Raw, Value};

/// What the walk sees, already read (the profiles at the key's [`crate::Place`]).
#[derive(Clone, Copy, Debug)]
pub struct Sources<'a> {
    /// The instance layer.
    pub instance: &'a Instance,
    /// The system's value for this key.
    pub system: Option<&'a Raw>,
    /// The general profile.
    pub person: &'a Snapshot,
    /// This program's profile.
    pub program: &'a Snapshot,
    /// The active named profile (general part, part of this program); `None` while the base is active.
    pub named: Option<(&'a Snapshot, &'a Snapshot)>,
}

/// The value a program uses, where it came from, and the lock if one holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effective {
    /// The value.
    pub value: Value,
    /// Its layer.
    pub from: Layer,
    /// The admin's lock, shown with its reason; `Some` only when `from == Layer::Instance`.
    pub lock: Option<Lock>,
}

/// The effective value of `d`, and the problems met on the way (each with `key` and `layer`).
#[must_use]
pub fn resolve(d: &Declaration, s: Sources<'_>) -> (Effective, Vec<Coded<SettingCode>>) {
    let mut problems = Vec::new();
    let mut instance = s.instance.values.values.get(&d.key);
    if let Some(lock) = s.instance.locks.get(&d.key) {
        if let Some(v) = take(d, Layer::Instance, instance, &mut problems) {
            let e = Effective {
                value: v,
                from: Layer::Instance,
                lock: Some(lock.clone()),
            };
            return (e, problems);
        }
        instance = None; // read once: its problem is listed already
        // Locked without a value that fits (cannot happen through `Admin::lock`; a file edited by hand): says so.
        problems.push(Coded::new(SettingCode::LockRefused).with_text(keys::KEY, d.key.as_str()));
    }
    let (named_person, named_program) = match s.named {
        Some((p, q)) => (p.values.get(&d.key), q.values.get(&d.key)),
        None => (None, None),
    };
    let walk: [(Layer, Option<&Raw>); 6] = [
        (Layer::NamedProgram, named_program),
        (Layer::NamedPerson, named_person),
        (Layer::Program, s.program.values.get(&d.key)),
        (Layer::Person, s.person.values.get(&d.key)),
        (Layer::System, s.system),
        (Layer::Instance, instance),
    ];
    for (layer, raw) in walk {
        if let Some(v) = take(d, layer, raw, &mut problems) {
            return (
                Effective {
                    value: v,
                    from: layer,
                    lock: None,
                },
                problems,
            );
        }
    }
    (
        Effective {
            value: d.default.clone(),
            from: Layer::Default,
            lock: None,
        },
        problems,
    )
}

/// Reads `raw` at `layer`; a value that does not fit goes to `problems`.
fn take(
    d: &Declaration,
    layer: Layer,
    raw: Option<&Raw>,
    problems: &mut Vec<Coded<SettingCode>>,
) -> Option<Value> {
    match d.kind.read(&d.key, layer, raw?) {
        Ok(v) => Some(v),
        Err(p) => {
            problems.push(p);
            None
        }
    }
}

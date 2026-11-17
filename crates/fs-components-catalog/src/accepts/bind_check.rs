//! The check of a block's bindings against its slots and against what is installed.
//!
//! Returns ALL problems in binding order, then the unbound required slots. Empty = the block can be drawn. The host
//! (`fs-forms-run`, run F2) draws a block with problems locked, with the codes as the reason; it never drops it silently.
//!
//! Pattern: Specification (pure function over values; no I/O, no clock).

use fs_types::{CommandKind, TypeDescription};

use super::bind::{Binding, Provider, Source};
use super::catalog::BlockSpec;
use super::codes::{AcceptsCode, keys};
use super::range::ContractRange;
use super::slot::{Cardinality, Role, Slot, basic_key};
use fs_types::coded::Coded;

/// Checks `bindings` of `block` against `providers` (one entry per installed provider and capability).
#[must_use]
pub fn check(
    block: &BlockSpec,
    bindings: &[Binding],
    providers: &[Provider],
) -> Vec<Coded<AcceptsCode>> {
    let mut out = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    let at = |code: AcceptsCode, slot: &str| {
        Coded::new(code)
            .with_text(keys::BLOCK, block.name.as_str())
            .with_text(keys::SLOT, slot)
    };
    for b in bindings {
        let Some(slot) = block.slot(&b.slot) else {
            out.push(at(AcceptsCode::SlotUnknown, &b.slot));
            continue;
        };
        if seen.iter().filter(|s| **s == b.slot).count() >= usize::from(slot.repeat) {
            out.push(at(AcceptsCode::BoundTwice, &b.slot));
            continue;
        }
        seen.push(&b.slot);
        if b.source.kind() != slot.kind {
            out.push(
                at(AcceptsCode::KindMismatch, &b.slot).with_text(keys::KIND, b.source.kind().key()),
            );
            continue;
        }
        if let Some(parent) = &slot.within {
            let fits = bindings.iter().any(|p| {
                &p.slot == parent && p.contract == b.contract && p.data_type == b.data_type
            });
            if !fits {
                out.push(
                    at(AcceptsCode::WithinMismatch, &b.slot)
                        .with_text(keys::WITHIN, parent.as_str()),
                );
            }
        }
        let Some(provider) = pick(providers, &b.contract) else {
            out.push(missing(providers, &b.contract));
            continue;
        };
        let Some(ty) = provider.type_of(&b.data_type) else {
            out.push(
                Coded::new(AcceptsCode::TypeUnknown)
                    .with_text(keys::CAPABILITY, b.contract.capability())
                    .with_text(keys::TYPE, b.data_type.as_str()),
            );
            continue;
        };
        if let Some(problem) = source_problem(block, slot, b, ty, provider) {
            out.push(problem);
        }
    }
    for slot in &block.accepts {
        if slot.required && !seen.contains(&slot.name.as_str()) {
            out.push(at(AcceptsCode::SlotUnbound, &slot.name));
        }
    }
    out
}

/// The provider in the range with the highest version.
fn pick<'a>(providers: &'a [Provider], range: &ContractRange) -> Option<&'a Provider> {
    providers
        .iter()
        .filter(|p| p.capability == range.capability() && range.contains(p.version))
        .max_by_key(|p| p.version)
}

fn missing(providers: &[Provider], range: &ContractRange) -> Coded<AcceptsCode> {
    let have: Vec<String> = providers
        .iter()
        .filter(|p| p.capability == range.capability())
        .map(|p| p.version.to_string())
        .collect();
    Coded::new(AcceptsCode::ContractMissing)
        .with_text(keys::CAPABILITY, range.capability())
        .with_text(keys::RANGE, range.to_string())
        .with_text(keys::HAVE, have.join(","))
}

/// The problem of one source against the type, if any.
fn source_problem(
    block: &BlockSpec,
    slot: &Slot,
    b: &Binding,
    ty: &TypeDescription,
    provider: &Provider,
) -> Option<Coded<AcceptsCode>> {
    let named = |name: &str| {
        Coded::new(AcceptsCode::NameUnknown)
            .with_text(keys::TYPE, ty.name.as_str())
            .with_text(keys::NAME, name)
    };
    let shape = provider.shape_of(&ty.name);
    match &b.source {
        Source::Records | Source::Lock => {
            let listable =
                ty.commands.iter().any(|c| c.kind == CommandKind::Find) && shape.is_some();
            (!listable).then(|| {
                Coded::new(AcceptsCode::NotListable).with_text(keys::TYPE, ty.name.as_str())
            })
        }
        Source::Grouping(name) => {
            let found = shape.is_some_and(|s| s.groupings.iter().any(|g| &g.name == name));
            (!found).then(|| named(name))
        }
        Source::Order(name) => {
            let found = shape.is_some_and(|s| s.orders.iter().any(|o| &o.name == name));
            (!found).then(|| named(name))
        }
        Source::Command(name) => ty.find_command(name).is_none().then(|| named(name)),
        Source::Field { path, role } => field_problem(block, slot, ty, path, *role),
    }
}

fn field_problem(
    block: &BlockSpec,
    slot: &Slot,
    ty: &TypeDescription,
    path: &[String],
    role: Role,
) -> Option<Coded<AcceptsCode>> {
    let at = |code: AcceptsCode| {
        Coded::new(code)
            .with_text(keys::BLOCK, block.name.as_str())
            .with_text(keys::SLOT, slot.name.as_str())
    };
    let joined = path.join("/");
    let field = match path {
        [one] => ty.find_field(one),
        _ => None,
    };
    let Some(field) = field else {
        return Some(
            Coded::new(AcceptsCode::FieldUnknown)
                .with_text(keys::TYPE, ty.name.as_str())
                .with_text(keys::PATH, joined),
        );
    };
    if !slot.basic.takes(field.basic) {
        return Some(at(AcceptsCode::BasicMismatch).with_text(keys::BASIC, basic_key(field.basic)));
    }
    if slot.role != Some(role) {
        return Some(at(AcceptsCode::RoleMismatch).with_text(keys::ROLE, role.key()));
    }
    if slot.cardinality == Cardinality::Single && field.multiple {
        return Some(at(AcceptsCode::CardinalityMismatch).with_text(keys::PATH, joined));
    }
    if role == Role::Input && field.read_only {
        return Some(
            Coded::new(AcceptsCode::FieldReadOnly)
                .with_text(keys::TYPE, ty.name.as_str())
                .with_text(keys::PATH, joined),
        );
    }
    None
}

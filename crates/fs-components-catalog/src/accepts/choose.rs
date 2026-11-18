//! The catalog chooses the building block of a field: one block for a basic type (and target) in a role (G7, block contract 0.4).
//!
//! The choice is a function of the TYPE and the ROLE only. A style (Fiori, Apple, Windows, Simple) never enters it: a style
//! changes how the block looks, not which block is drawn. No program name, no table in the host.
//!
//! * A block that names the target of a reference (`target = ["Person"]`) wins over the general block of the type.
//! * Two blocks for the same field and role are a reading error of the catalog (`renders-twice`), found when it is read.
//! * No block for a type and role: `renders-none`; never a silent fall back. The one exception is `Unknown` (a type newer than
//!   this reader): it is drawn by [`FALLBACK_BLOCK`] and the answer says `locked`, so the host shows the reason.
//!
//! Pattern: Abstract Factory over the catalog (the host asks the catalog, then calls only the factory of the block named).

use fs_types::FieldDescription;

use super::catalog::{BlockCatalog, BlockSpec};
use super::codes::{AcceptsCode, keys};
use super::slot::{BASICS, Role, basic_key};
use fs_types::BasicType;
use fs_types::coded::Coded;

/// The block that draws a field whose basic type this reader does not know (plain text, input locked with the reason).
pub const FALLBACK_BLOCK: &str = "field-text";

/// The answer of [`BlockCatalog::choose`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chosen<'a> {
    /// The block to draw.
    pub block: &'a BlockSpec,
    /// The field is of a type this reader does not know: show the block with the input locked and say why.
    pub locked: bool,
}

impl BlockCatalog {
    /// The one block that draws `field` in `role`.
    ///
    /// # Errors
    /// `renders-none` (no block for the type and role), `renders-twice` (the catalog was built with two; only possible for a
    /// catalog not read from files), `block-unknown` (the fallback block of an unknown type is not in the catalog).
    pub fn choose(
        &self,
        field: &FieldDescription,
        role: Role,
    ) -> Result<Chosen<'_>, Coded<AcceptsCode>> {
        if basic_key(field.basic) == "unknown" {
            return Ok(Chosen {
                block: self.need(FALLBACK_BLOCK)?,
                locked: true,
            });
        }
        let mine = |b: &&BlockSpec| {
            b.renders
                .as_ref()
                .is_some_and(|r| r.basic == field.basic && r.has_role(role))
        };
        let named: Vec<&BlockSpec> = self
            .iter()
            .filter(mine)
            .filter(|b| {
                b.renders
                    .as_ref()
                    .is_some_and(|r| r.names_target(&field.target))
            })
            .collect();
        let pool: Vec<&BlockSpec> = if named.is_empty() {
            self.iter()
                .filter(mine)
                .filter(|b| b.renders.as_ref().is_some_and(|r| r.target.is_empty()))
                .collect()
        } else {
            named
        };
        match pool.as_slice() {
            [one] => Ok(Chosen {
                block: one,
                locked: false,
            }),
            [] => Err(Coded::new(AcceptsCode::RendersNone)
                .with_text(keys::BASIC, basic_key(field.basic))
                .with_text(keys::ROLE, role.key())
                .with_text(keys::TARGET, field.target.as_str())),
            [first, second, ..] => Err(twice_code(first, second, role)),
        }
    }

    /// Every basic type and role that no block renders: `renders-none` for each (the catalog is complete when empty). The host
    /// refuses to start with a gap in a type a program binds; a gap in a type nobody binds is only reported.
    #[must_use]
    pub fn gaps(&self) -> Vec<Coded<AcceptsCode>> {
        let roles = [Role::Input, Role::Display, Role::Compact, Role::Cell];
        let mut out = Vec::new();
        for basic in BASICS {
            for role in roles {
                let drawn = self.iter().any(|b| {
                    b.renders
                        .as_ref()
                        .is_some_and(|r| r.basic == basic && r.has_role(role))
                });
                if !drawn {
                    out.push(
                        Coded::new(AcceptsCode::RendersNone)
                            .with_text(keys::BASIC, basic_key(basic))
                            .with_text(keys::ROLE, role.key())
                            .with_text(keys::TARGET, ""),
                    );
                }
            }
        }
        out
    }
}

/// Every pair of blocks that draw the same field in the same role (`renders-twice`), in block name order.
pub(super) fn twice<'a>(blocks: impl Iterator<Item = &'a BlockSpec>) -> Vec<Coded<AcceptsCode>> {
    let list: Vec<&BlockSpec> = blocks.filter(|b| b.renders.is_some()).collect();
    let mut out = Vec::new();
    for (i, a) in list.iter().enumerate() {
        for b in &list[i + 1..] {
            for role in [Role::Input, Role::Display, Role::Compact, Role::Cell] {
                let clash = match (&a.renders, &b.renders) {
                    (Some(x), Some(y)) => x.clash(y, role),
                    _ => false,
                };
                if clash {
                    out.push(twice_code(a, b, role));
                }
            }
        }
    }
    out
}

fn twice_code(a: &BlockSpec, b: &BlockSpec, role: Role) -> Coded<AcceptsCode> {
    let basic = a.renders.as_ref().map_or(BasicType::Unknown, |r| r.basic);
    let target = a
        .renders
        .as_ref()
        .map_or(String::new(), |r| r.target.join(","));
    Coded::new(AcceptsCode::RendersTwice)
        .with_text(keys::BASIC, basic_key(basic))
        .with_text(keys::TARGET, target)
        .with_text(keys::ROLE, role.key())
        .with_text(keys::FIRST, a.name.as_str())
        .with_text(keys::SECOND, b.name.as_str())
}

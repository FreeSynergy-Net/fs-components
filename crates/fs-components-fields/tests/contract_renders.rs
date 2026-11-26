//! The building blocks of the fields and the block contract say the same: every `Kind` has exactly one block in the contract
//! that `renders` its basic type in all four roles, and every contract block that `renders` a field is either a `Kind` or on the
//! short list of blocks that have no `Kind` yet. "Whenever it is an e-mail, the e-mail field draws it" (user rule).
//!
//! Mutants: a `Kind::block` name that the contract does not have -> `every_kind_has_exactly_one_renders_block`; a contract block
//! that `renders` a type but has no `Kind` and is not listed -> `every_renders_block_is_a_kind_or_listed`; the catalog chooses
//! another block than `Kind::block` -> `choose_on_the_real_catalog_gives_the_kind_block`.
//! Pattern: none; tests over the real contract files.

#![allow(clippy::pedantic)]

use fs_components_catalog::{BlockCatalog, FALLBACK_BLOCK, Role, basic_from_key, basic_key};
use fs_components_fields::Kind;
use fs_types::coded::Param;
use fs_types::{BasicType, FieldDescription};

const ROLES: [Role; 4] = [Role::Input, Role::Display, Role::Compact, Role::Cell];

/// Blocks of the contract that draw a field but have no `Kind` (component) in this crate yet: the plain text field (it is the
/// textbox), the person picker (a reference to a person) and the person face (an image, in the roles display and compact; the
/// initials stand in when there is none); the widgets of the last two live in `fs-components-widgets`.
const NO_KIND: [&str; 3] = ["field-text", "person-picker", "person-face"];

/// The basic type of a kind: the key of the kind, except the many-line text, whose basic type is `multilineText`.
fn basic_of(kind: Kind) -> Option<BasicType> {
    let key = match kind {
        Kind::Multiline => "multilineText",
        other => other.field().key(),
    };
    basic_from_key(key)
}

fn real() -> BlockCatalog {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../contract");
    let read = |p: &str| std::fs::read_to_string(format!("{dir}/{p}")).unwrap();
    let mut names: Vec<String> = std::fs::read_dir(format!("{dir}/blocks"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".toml"))
        .collect();
    names.sort();
    let files: Vec<(String, String)> = names
        .iter()
        .map(|n| (n.clone(), read(&format!("blocks/{n}"))))
        .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    BlockCatalog::read(&read("contract.toml"), &refs).expect("the contract reads")
}

#[test]
fn every_kind_has_exactly_one_renders_block() {
    let cat = real();
    for kind in Kind::ALL {
        let basic = basic_of(kind).expect("a Kind is a basic type");
        let mine: Vec<&str> = cat
            .iter()
            // a block with a target (the person picker) is the more exact rule for the same basic type, not a second block
            .filter(|b| {
                b.renders
                    .as_ref()
                    .is_some_and(|r| r.basic == basic && r.target.is_empty())
            })
            .map(|b| b.name.as_str())
            .collect();
        assert_eq!(
            mine,
            [kind.block()],
            "{kind:?}: one block, the one of `Kind::block`"
        );
        let r = cat.need(kind.block()).unwrap().renders.as_ref().unwrap();
        assert!(
            r.target.is_empty(),
            "{kind:?}: the field block names no target"
        );
        for role in ROLES {
            assert!(r.has_role(role), "{kind:?} {role:?}");
        }
    }
}

#[test]
fn every_renders_block_is_a_kind_or_listed() {
    let cat = real();
    for b in cat.iter().filter(|b| b.renders.is_some()) {
        let known = Kind::from_block(&b.name).is_some() || NO_KIND.contains(&b.name.as_str());
        assert!(
            known,
            "`{}` renders a field but has no Kind and is not on the list",
            b.name
        );
    }
    for name in NO_KIND {
        assert!(
            cat.block(name).is_some_and(|b| b.renders.is_some()),
            "{name}"
        );
        assert!(
            Kind::from_block(name).is_none(),
            "{name} got a Kind: take it off the list"
        );
    }
}

#[test]
fn choose_on_the_real_catalog_gives_the_kind_block() {
    let cat = real();
    for kind in Kind::ALL {
        let basic = basic_of(kind).unwrap();
        for role in ROLES {
            let got = cat
                .choose(&FieldDescription::new("x", basic), role)
                .unwrap();
            assert_eq!(
                (got.block.name.as_str(), got.locked),
                (kind.block(), false),
                "{kind:?} {role:?}"
            );
        }
    }
    for role in ROLES {
        let got = cat
            .choose(&FieldDescription::new("x", BasicType::Text), role)
            .unwrap();
        assert_eq!(got.block.name, "field-text");
    }
}

#[test]
fn the_real_catalog_is_no_longer_none_everywhere() {
    let cat = real();
    // thirteen types are drawn (text, multiline text, email, name, phone, address, date, link, choice, keywords, reference, dateTime,
    // duration), the person picker draws a reference in the role input and the person face an image in the roles display and
    // compact: the other types stay gaps, nothing is dropped silently
    let gaps = cat.gaps();
    let basics: Vec<String> = gaps
        .iter()
        .map(
            |g| match g.params().find(|(k, _)| *k == "basic").map(|(_, v)| v) {
                Some(Param::Text(t)) => t.clone(),
                _ => String::new(),
            },
        )
        .collect();
    assert_eq!(gaps.len(), 14, "{basics:?}");
    for drawn in [
        "text",
        "multilineText",
        "email",
        "name",
        "phone",
        "address",
        "date",
        "link",
        "choice",
        "keywords",
        "reference",
        "dateTime",
        "duration",
    ] {
        assert!(
            !basics.iter().any(|b| b == drawn),
            "{drawn} still has a gap"
        );
    }
    // a type nobody draws yet stays visible as `renders-none`, never as text
    let e = cat
        .choose(&FieldDescription::new("x", BasicType::Place), Role::Cell)
        .unwrap_err();
    assert_eq!(e.code(), fs_components_catalog::AcceptsCode::RendersNone);
    assert_eq!(basic_key(BasicType::Place), "place");
}

#[test]
fn an_unknown_type_is_drawn_by_the_fallback_block_locked() {
    let cat = real();
    let got = cat
        .choose(&FieldDescription::new("x", BasicType::Unknown), Role::Input)
        .unwrap();
    assert_eq!(
        (got.block.name.as_str(), got.locked),
        (FALLBACK_BLOCK, true)
    );
}

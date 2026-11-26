//! G7: the catalog chooses the block of a field (`renders`, `BlockCatalog::choose`); K-cases of PLAN-grundtyp-felder 7.4.
//! The table `WANT` is written out here, apart from the code and from the data file.
mod support;

use fs_components_catalog::{AcceptsCode, BlockCatalog, FALLBACK_BLOCK, Role};
use fs_types::{BasicType, FieldDescription};
use support::util::*;

const RENDERS: &str = include_str!("data/blocks-renders.toml");
const ROLES: [Role; 4] = [Role::Input, Role::Display, Role::Compact, Role::Cell];

/// The block of every basic type, independent of the data file.
const WANT: [(BasicType, &str); 20] = [
    (BasicType::Text, "field-text"),
    (BasicType::MultilineText, "field-multiline"),
    (BasicType::Email, "field-email"),
    (BasicType::Phone, "field-phone"),
    (BasicType::Name, "field-name"),
    (BasicType::Address, "field-address"),
    (BasicType::Place, "field-place"),
    (BasicType::Date, "field-date"),
    (BasicType::DateTime, "field-date-time"),
    (BasicType::Language, "field-language"),
    (BasicType::Link, "field-link"),
    (BasicType::Image, "field-image"),
    (BasicType::Key, "field-key"),
    (BasicType::Choice, "field-choice"),
    (BasicType::Keywords, "field-keywords"),
    (BasicType::Reference, "field-reference"),
    (BasicType::Integer, "field-integer"),
    (BasicType::Whole, "field-whole"),
    (BasicType::Flag, "field-flag"),
    (BasicType::Duration, "field-duration"),
];

fn cat() -> BlockCatalog {
    catalog_of(RENDERS)
}

fn reference(target: &str) -> FieldDescription {
    let mut f = FieldDescription::new("who", BasicType::Reference);
    target.clone_into(&mut f.target);
    f
}

/// C1: every basic type in every role gets exactly the block of the table (mutant M1: email -> field-text).
#[test]
fn c1_choose_table() {
    let c = cat();
    for (basic, block) in WANT {
        for role in ROLES {
            let got = c.choose(&FieldDescription::new("x", basic), role).unwrap();
            assert_eq!(got.block.name, block, "{basic:?} {role:?}");
            assert!(!got.locked);
        }
    }
}

/// C2: a reference to a person, profile, group or organization is drawn by the person picker, any other target (or none) by
/// the general reference block (mutant M4: general before specific).
#[test]
fn c2_target_wins() {
    let c = cat();
    for t in ["Person", "Profile", "Group", "Organization"] {
        for role in ROLES {
            assert_eq!(
                c.choose(&reference(t), role).unwrap().block.name,
                "person-picker"
            );
        }
    }
    for t in ["Task", ""] {
        assert_eq!(
            c.choose(&reference(t), Role::Cell).unwrap().block.name,
            "field-reference"
        );
    }
}

/// C3: an unknown type is drawn by the plain text block and is LOCKED (mutant M5: `None` or a panic).
#[test]
fn c3_unknown_is_locked_text() {
    let c = cat();
    let got = c
        .choose(&FieldDescription::new("x", BasicType::Unknown), Role::Input)
        .unwrap();
    assert_eq!(
        (got.block.name.as_str(), got.locked),
        (FALLBACK_BLOCK, true)
    );
}

/// C4: the unknown type without the fallback block is `block-unknown`, never a made-up block.
#[test]
fn c4_unknown_without_fallback() {
    let c = catalog_of(
        "[block.field-email]\naccepts = []\nrenders = { basic = \"email\", roles = [\"input\"] }\n",
    );
    let e = c
        .choose(&FieldDescription::new("x", BasicType::Unknown), Role::Input)
        .unwrap_err();
    assert_eq!(e.code(), AcceptsCode::BlockUnknown);
}

/// C5: a type or role nobody draws is `renders-none` with its parameters, not a silent text field.
#[test]
fn c5_none_is_visible() {
    let c = catalog_of(
        "[block.field-email]\naccepts = []\nrenders = { basic = \"email\", roles = [\"input\"] }\n",
    );
    let e = c
        .choose(&FieldDescription::new("x", BasicType::Email), Role::Cell)
        .unwrap_err();
    assert_eq!(show(&[e]), ["RendersNone[basic=email,role=cell,target=]"]);
    let e = c
        .choose(&FieldDescription::new("x", BasicType::Phone), Role::Input)
        .unwrap_err();
    assert_eq!(e.code(), AcceptsCode::RendersNone);
    let e = c.choose(&reference("Task"), Role::Input).unwrap_err();
    assert_eq!(param(&e, "target").as_deref(), Some("Task"));
}

/// C6: two blocks for one basic type and role are a reading error with both names (mutant M2); different roles are fine.
#[test]
fn c6_twice() {
    let two = "[block.a]\naccepts = []\nrenders = { basic = \"email\", roles = [\"input\", \"cell\"] }\n\
               [block.b]\naccepts = []\nrenders = { basic = \"email\", roles = [\"cell\"] }\n";
    let err = BlockCatalog::read(CONTRACT, &[("t.toml", two)]).unwrap_err();
    assert_eq!(
        show(&err),
        ["RendersTwice[basic=email,target=,role=cell,first=a,second=b]"]
    );
    let apart = "[block.a]\naccepts = []\nrenders = { basic = \"email\", roles = [\"input\"] }\n\
                 [block.b]\naccepts = []\nrenders = { basic = \"email\", roles = [\"cell\"] }\n";
    assert!(BlockCatalog::read(CONTRACT, &[("t.toml", apart)]).is_ok());
    let targets = "[block.a]\naccepts = []\nrenders = { basic = \"reference\", target = [\"Person\"], roles = [\"cell\"] }\n\
                   [block.b]\naccepts = []\nrenders = { basic = \"reference\", target = [\"Person\", \"Group\"], roles = [\"cell\"] }\n";
    let err = BlockCatalog::read(CONTRACT, &[("t.toml", targets)]).unwrap_err();
    assert_eq!(codes(&err), [AcceptsCode::RendersTwice]);
    let general_and_named = "[block.a]\naccepts = []\nrenders = { basic = \"reference\", roles = [\"cell\"] }\n\
                             [block.b]\naccepts = []\nrenders = { basic = \"reference\", target = [\"Person\"], roles = [\"cell\"] }\n";
    assert!(BlockCatalog::read(CONTRACT, &[("t.toml", general_and_named)]).is_ok());
}

/// C7: the closed schema of `renders`: unknown key, unknown basic word, empty or duplicate roles, a role word that is none.
#[test]
fn c7_schema() {
    let bad = |renders: &str| {
        let text = format!("[block.a]\naccepts = []\nrenders = {renders}\n");
        BlockCatalog::read(CONTRACT, &[("t.toml", &text)]).unwrap_err()
    };
    assert_eq!(
        show(&bad("{ basic = \"email\", roles = [\"cell\"], x = 1 }")),
        ["RendersInvalid[block=a,field=x]"]
    );
    assert_eq!(
        show(&bad("{ basic = \"webAddress\", roles = [\"cell\"] }")),
        ["RendersUnknownBasic[block=a,word=webAddress]"]
    );
    assert_eq!(
        show(&bad("{ basic = \"email\", roles = [] }")),
        ["RendersInvalid[block=a,field=roles]"]
    );
    assert_eq!(
        show(&bad("{ basic = \"email\", roles = [\"cell\", \"cell\"] }")),
        ["RendersInvalid[block=a,field=roles]"]
    );
    assert_eq!(
        show(&bad("{ basic = \"email\", roles = [\"big\"] }")),
        ["RendersInvalid[block=a,field=roles]"]
    );
    assert_eq!(
        show(&bad("{ roles = [\"cell\"] }")),
        ["RendersInvalid[block=a,field=basic]"]
    );
    assert_eq!(
        show(&bad("\"email\"")),
        ["RendersInvalid[block=a,field=renders]"]
    );
    assert_eq!(
        codes(&bad(
            "{ basic = \"email\", roles = [\"cell\"], target = [] }"
        )),
        [AcceptsCode::RendersInvalid]
    );
}

/// C8: the completeness report: the test data has no gap; a block without `cell` shows exactly that gap (mutant M3).
#[test]
fn c8_gaps() {
    assert_eq!(cat().gaps().len(), 0);
    let no_cell = RENDERS.replacen(
        "{ basic = \"email\", roles = [\"input\", \"display\", \"compact\", \"cell\"] }",
        "{ basic = \"email\", roles = [\"input\", \"display\", \"compact\"] }",
        1,
    );
    let gaps = BlockCatalog::read(CONTRACT, &[("t.toml", &no_cell)])
        .unwrap()
        .gaps();
    assert_eq!(show(&gaps), ["RendersNone[basic=email,role=cell,target=]"]);
}

/// C9: the choice has no style: `choose` takes a field and a role and nothing else, so the same answer for every style (M8).
#[test]
fn c9_no_style_in_the_choice() {
    let c = cat();
    let f = FieldDescription::new("x", BasicType::Email);
    let first = c.choose(&f, Role::Compact).unwrap().block.name.clone();
    for _style in ["fiori", "apple", "windows", "simple"] {
        assert_eq!(c.choose(&f, Role::Compact).unwrap().block.name, first);
    }
}

/// Pattern: Specification – a predicate over the real contract files: they read and hold the blocks, slots and small components named here (nearest catalog pattern; the test has no behaviour of its own).
/// C10: the real contract files (`contract/blocks/*.toml`) read, every block names `accepts`, the board blocks have their slots.
#[test]
fn c10_real_contract() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../contract");
    let read = |p: &str| std::fs::read_to_string(format!("{dir}/{p}")).unwrap();
    let files: Vec<(String, String)> = [
        "dialogs", "fields", "frames", "general", "programs", "widgets",
    ]
    .iter()
    .map(|n| (format!("{n}.toml"), read(&format!("blocks/{n}.toml"))))
    .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let c = BlockCatalog::read(&read("contract.toml"), &refs).expect("the contract reads");
    // 120 blocks (`calendar-view` dropped in B7) + the three small components (`field-shortcut`, `swatch`, `swatch-grid`) + `code-editor` + the frames `split`, `tabs`
    assert_eq!(c.len(), 126);
    // `field-shortcut` has no `renders` until the basic type `shortcut` exists in fs-types
    assert!(c.need("field-shortcut").unwrap().renders.is_none());
    assert!(c.need("swatch").is_ok() && c.need("swatch-grid").is_ok());
    let board = c.need("board").unwrap();
    assert!(board.slot("cards").unwrap().required);
    assert_eq!(board.slot("card-facts").unwrap().repeat, 4);
    // `board-detail` is part of the board, not a block of its own: no slots, no widget (K6)
    let detail = c.need("board-detail").unwrap();
    assert!(detail.accepts.is_empty() && detail.widget.is_none());
    let picker = c.need("person-picker").unwrap();
    assert!(picker.renders.as_ref().unwrap().names_target("Person"));
}

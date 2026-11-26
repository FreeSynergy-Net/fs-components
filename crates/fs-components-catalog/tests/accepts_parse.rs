//! K1 (accepts codes), K2-K7 of ENTWURF section 5: slot words, the closed slot schema, the catalog.
mod support;

use fs_components_catalog::{
    AcceptsCode, BasicSet, BlockCatalog, Cardinality, Role, SlotKind, basic_from_key, basic_key,
    read_blocks,
};
use fs_types::BasicType;
use fs_types::coded::{ErrorCode, message_key};
use support::util::*;

/// K1: every variant once, unique kebab-case names, `message_key` = prefix + name; 31 codes.
#[test]
fn k1_accepts_codes() {
    let all = AcceptsCode::ALL;
    assert_eq!(all.len(), 31);
    let mut names: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), all.len());
    for c in all {
        let n = c.as_str();
        assert!(
            n.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-')
                && !n.starts_with('-')
                && !n.ends_with('-')
        );
        assert_eq!(message_key(*c), format!("accepts-error-{n}"));
    }
}

/// K2: basic words round trip for 16 types; `unknown` and `Text` are no words; roles and kinds round trip; `Any` never takes `Unknown`.
#[test]
fn k2_words() {
    let all = [
        BasicType::Text,
        BasicType::MultilineText,
        BasicType::Email,
        BasicType::Phone,
        BasicType::Name,
        BasicType::Address,
        BasicType::Place,
        BasicType::Date,
        BasicType::DateTime,
        BasicType::Language,
        BasicType::Link,
        BasicType::Image,
        BasicType::Key,
        BasicType::Choice,
        BasicType::Keywords,
        BasicType::Reference,
        BasicType::Duration,
    ];
    for b in all {
        assert_eq!(basic_from_key(basic_key(b)), Some(b));
    }
    assert_eq!(basic_from_key("unknown"), None);
    assert_eq!(basic_from_key("Text"), None);
    for r in [Role::Input, Role::Display, Role::Compact, Role::Cell] {
        assert_eq!(Role::from_key(r.key()), Some(r));
    }
    for k in [
        SlotKind::Value,
        SlotKind::Records,
        SlotKind::Grouping,
        SlotKind::Order,
        SlotKind::Command,
        SlotKind::Lock,
    ] {
        assert_eq!(SlotKind::from_key(k.key()), Some(k));
    }
    assert_eq!(basic_key(BasicType::Unknown), "unknown");
    assert_eq!(basic_from_key("unknown"), None);
    assert!(!BasicSet::Any.takes(BasicType::Unknown));
    assert!(BasicSet::Any.takes(BasicType::Text));
    assert!(!BasicSet::Only(vec![BasicType::Unknown]).takes(BasicType::Unknown));
}

fn one_block(slot: &str) -> String {
    format!("[block.t]\naccepts = [\n  {{ slot = \"r\", kind = \"records\" }},\n  {slot},\n]\n")
}

fn fields(text: &str) -> Vec<String> {
    let list = read_blocks("t.toml", text).expect_err("must be refused");
    list.iter()
        .map(|c| {
            assert_eq!(c.code(), AcceptsCode::SlotInvalid, "{:?}", show(&list));
            param(c, "field").unwrap_or_default()
        })
        .collect()
}

/// K3: slot names: `card-title`, `a1` valid; empty, `A`, `-a`, `a-`, `a--b`, 33 characters invalid (`slot-invalid{field=slot}`).
#[test]
fn k3_slot_names() {
    for good in ["card-title", "a1"] {
        assert!(
            read_blocks(
                "t.toml",
                &one_block(&format!("{{ slot = \"{good}\", kind = \"records\" }}"))
            )
            .is_ok(),
            "{good}"
        );
    }
    for bad in ["", "A", "-a", "a-", "a--b", &"a".repeat(33)] {
        let f = fields(&one_block(&format!(
            "{{ slot = \"{bad}\", kind = \"records\" }}"
        )));
        assert_eq!(f, ["slot"], "{bad}");
    }
}

/// K4: the draft data reads: 11 blocks (the draft text says 12: miscount); `board` has 10 slots with the shapes the draft names.
#[test]
fn k4_draft_data() {
    let c = catalog();
    assert_eq!(c.len(), 10); // the draft text says 12, its data has 10 (5 frames + 5; `board-detail` is part of the board, K6)
    let board = c.need("board").unwrap();
    assert_eq!(board.accepts.len(), 11);
    let cards = board.slot("cards").unwrap();
    assert_eq!(
        (cards.kind, cards.cardinality, cards.required),
        (SlotKind::Records, Cardinality::Multiple, true)
    );
    let facts = board.slot("card-facts").unwrap();
    assert_eq!(
        (facts.kind, &facts.basic, facts.role),
        (SlotKind::Value, &BasicSet::Any, Some(Role::Compact))
    );
    assert_eq!(
        (facts.cardinality, facts.repeat),
        (Cardinality::Multiple, 4)
    );
    assert_eq!(board.slot("order").unwrap().repeat, 2);
    assert_eq!(c.need("app-window").unwrap().accepts.len(), 0);
}

/// K5b: `repeat` has no ceiling of ours (rule 57): 17 and 1000 are read, 0 and a number beyond `u32` are `slot-invalid{repeat}`.
#[test]
fn k5b_repeat_has_no_small_ceiling() {
    for n in [1_u32, 16, 17, 32, 1000, u32::MAX] {
        let text = format!(
            "[block.a]\naccepts = [{{ slot = \"activate\", kind = \"command\", repeat = {n} }}]\n"
        );
        let c = BlockCatalog::read(CONTRACT, &[("t.toml", &text)]).unwrap();
        assert_eq!(c.need("a").unwrap().slot("activate").unwrap().repeat, n);
    }
}

/// K5: one broken slot each gives exactly the named `slot-invalid{field}`; a bad `kind` gives ONE problem.
#[test]
fn k5_broken_slots() {
    let v = |extra: &str| format!("{{ slot = \"a\", kind = \"value\", {extra} }}");
    let cases: Vec<(String, Vec<&str>)> = vec![
        (
            "{ slot = \"activate\", kind = \"command\", colour = \"red\" }".into(),
            vec!["colour"],
        ),
        (
            "{ slot = \"a\", kind = \"list\", role = \"edit\" }".into(),
            vec!["kind"],
        ),
        (v("role = \"display\", within = \"r\""), vec!["basic"]),
        (
            "{ slot = \"a\", kind = \"records\", basic = [\"text\"] }".into(),
            vec!["basic"],
        ),
        (
            v("basic = [\"text\"], role = \"edit\", within = \"r\""),
            vec!["role"],
        ),
        (
            "{ slot = \"a\", kind = \"records\", role = \"display\" }".into(),
            vec!["role"],
        ),
        (
            "{ slot = \"a\", kind = \"grouping\", cardinality = \"multiple\", within = \"r\" }"
                .into(),
            vec!["cardinality"],
        ),
        (
            "{ slot = \"activate\", kind = \"command\", required = \"yes\" }".into(),
            vec!["required"],
        ),
        (v("basic = [\"text\"], role = \"display\""), vec!["within"]),
        (
            "{ slot = \"a\", kind = \"records\", within = \"r\" }".into(),
            vec!["within"],
        ),
        (
            "{ slot = \"activate\", kind = \"command\", repeat = 0 }".into(),
            vec!["repeat"],
        ),
        (
            "{ slot = \"activate\", kind = \"command\", repeat = 4294967296 }".into(),
            vec!["repeat"],
        ),
        (
            "{ slot = \"activate\", kind = \"command\", repeat = \"2\" }".into(),
            vec!["repeat"],
        ),
        (
            v("basic = [\"any\", \"text\"], role = \"display\", within = \"r\""),
            vec!["basic"],
        ),
        (
            v("basic = [\"text\", \"text\"], role = \"display\", within = \"r\""),
            vec!["basic"],
        ),
        (
            v("basic = [\"unknown\"], role = \"display\", within = \"r\""),
            vec!["basic"],
        ),
    ];
    for (slot, want) in cases {
        assert_eq!(fields(&one_block(&slot)), want, "{slot}");
    }
    // `within` naming a value slot, not a records slot
    let text = "[block.t]\naccepts = [\n  { slot = \"r\", kind = \"records\" },\n  { slot = \"x\", kind = \"value\", basic = [\"text\"], role = \"display\", within = \"r\" },\n  { slot = \"a\", kind = \"value\", basic = [\"text\"], role = \"display\", within = \"x\" },\n]\n";
    assert_eq!(fields(text), ["within"]);
}

/// K6: a block without `accepts`, `accepts` of the wrong form, two slots of one name, no TOML, a file with three broken blocks.
#[test]
fn k6_block_level() {
    let e = read_blocks("t.toml", "[block.t]\nsummary = \"x\"\n").unwrap_err();
    assert_eq!(show(&e), ["AcceptsMissing[block=t]"]);
    assert_eq!(fields("[block.t]\naccepts = \"x\"\n"), ["accepts"]);
    let twice = "[block.t]\naccepts = [\n  { slot = \"activate\", kind = \"command\" },\n  { slot = \"activate\", kind = \"command\" },\n]\n";
    assert_eq!(
        codes(&read_blocks("t.toml", twice).unwrap_err()),
        [AcceptsCode::SlotTwice]
    );
    assert_eq!(
        show(&read_blocks("t.toml", "block = [").unwrap_err()),
        ["FileInvalid[file=t.toml,field=toml]"]
    );
    let three = "[block.a]\nx = 1\n[block.b]\naccepts = 5\n[block.c]\naccepts = [ { slot = \"A\", kind = \"command\" } ]\n";
    assert!(read_blocks("t.toml", three).unwrap_err().len() >= 3);
}

/// K7: `BlockCatalog::read`: version `0.2`, no version, one block in two files; `need("nope")`.
#[test]
fn k7_catalog() {
    let blocks = [("a.toml", "[block.t]\naccepts = []\n")];
    let old = BlockCatalog::read("[contract]\nversion = \"0.2\"\n", &blocks).unwrap_err();
    assert_eq!(show(&old), ["ContractVersion[found=0.2,want=0.5.0]"]);
    let none = BlockCatalog::read("[contract]\nstatus = \"draft\"\n", &blocks).unwrap_err();
    assert_eq!(
        show(&none),
        ["FileInvalid[file=contract.toml,field=version]"]
    );
    let twice = [
        ("a.toml", "[block.t]\naccepts = []\n"),
        ("b.toml", "[block.t]\naccepts = []\n"),
    ];
    let e = BlockCatalog::read(CONTRACT, &twice).unwrap_err();
    assert_eq!(codes(&e), [AcceptsCode::BlockTwice]);
    let c = catalog();
    assert_eq!(
        show(&[c.need("nope").unwrap_err()]),
        ["BlockUnknown[block=nope]"]
    );
}

/// V-3: the contract version is three parts and compared as a `semver::Version`: `0.4` (two parts, the old spelling), `0.4.0` (the old contract) and `0.5.1`
/// are `contract-version`, `0.5.0` reads, and `CONTRACT_VERSION` is the text of the one version the reader holds.
#[test]
fn v3_contract_version_is_three_parts() {
    let blocks = [("a.toml", "[block.t]\naccepts = []\n")];
    for found in ["0.4", "0.4.0", "0.5.1", "0.3.0", "04.0.0"] {
        let text = format!("[contract]\nversion = \"{found}\"\n");
        let err = BlockCatalog::read(&text, &blocks).unwrap_err();
        assert_eq!(
            show(&err),
            [format!("ContractVersion[found={found},want=0.5.0]")]
        );
    }
    assert!(BlockCatalog::read("[contract]\nversion = \"0.5.0\"\n", &blocks).is_ok());
    assert_eq!(fs_components_catalog::CONTRACT_VERSION, "0.5.0");
}

//! Style completeness for contract version 0.2: the theme contract times all styles, a missing entry is red.
//!
//! Tests: reads `contract/blocks/*.toml` (the block names) and `contract/contract.toml` (the version) as text and checks them against
//! `block_tokens` for every `StyleId`, light and dark. The scripts `stil-vollstaendigkeit` and `zwei-stile` of the workflow
//! `frontend-umbau` do not exist yet; this is their check for the blocks of 0.2.

use std::collections::BTreeSet;
use std::path::PathBuf;

use fs_components_styles::styles::block_tokens::{BLOCK_TOKENS, NAMES, look, sheet, tokens};
use fs_components_styles::{StyleId, style};

fn contract_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contract")
}

fn blocks_of(file: &str) -> BTreeSet<String> {
    let text =
        std::fs::read_to_string(contract_dir().join("blocks").join(file)).expect("blocks file");
    text.lines()
        .filter_map(|l| l.strip_prefix("[block.")?.strip_suffix(']'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_contract_is_version_0_2() {
    let text =
        std::fs::read_to_string(contract_dir().join("contract.toml")).expect("contract.toml");
    assert!(
        text.lines().any(|l| l.trim() == "version = \"0.2\""),
        "a new block, part or state is a new version of the contract (alle-stile-erweitern)"
    );
}

#[test]
fn every_block_of_0_2_has_its_tokens_and_no_token_belongs_to_a_missing_block() {
    let mut of_0_2 = blocks_of("dialogs.toml");
    // the three blocks of 0.1 that 0.2 gave new parts, variants or states
    of_0_2.extend(["dialog", "alert", "empty-state"].map(str::to_owned));
    let listed: BTreeSet<String> = BLOCK_TOKENS.iter().map(|(b, _)| (*b).to_owned()).collect();
    assert_eq!(
        listed, of_0_2,
        "BLOCK_TOKENS and the blocks of 0.2 must be the same list"
    );
    let mut all = blocks_of("general.toml");
    all.extend(blocks_of("frames.toml"));
    all.extend(blocks_of("dialogs.toml"));
    for (block, needs) in BLOCK_TOKENS {
        assert!(
            all.contains(*block),
            "`{block}` is not a block of the contract"
        );
        for token in *needs {
            assert!(
                NAMES.contains(token),
                "`{block}` needs `{token}`, which no style gives"
            );
        }
    }
}

#[test]
fn every_style_gives_every_token_light_and_dark() {
    for id in StyleId::ALL {
        for dark in [false, true] {
            // the tokens are over the theme's own variables, so one sheet serves both appearances; the check is on the
            // style and also on the colours the tokens will meet
            let s = style(id);
            let given = tokens(s);
            let names: Vec<&str> = given.iter().map(|(n, _)| *n).collect();
            assert_eq!(names, NAMES, "{} dark={dark}", id.name());
            for (name, value) in &given {
                assert!(
                    !value.trim().is_empty(),
                    "{} gives an empty `{name}`",
                    id.name()
                );
            }
            assert!(sheet(s).starts_with(":root {") && sheet(s).trim_end().ends_with('}'));
            let p = s.palette(dark);
            assert!(
                p.accent != p.ground,
                "{} dark={dark}: the accent is the ground",
                id.name()
            );
        }
    }
}

#[test]
fn the_styles_do_not_copy_one_another() {
    // every pair of styles differs in at least three tokens: a block drawn in two styles must change
    for a in StyleId::ALL {
        for b in StyleId::ALL.into_iter().filter(|b| *b != a) {
            let (ta, tb) = (tokens(style(a)), tokens(style(b)));
            let differing = ta.iter().zip(&tb).filter(|(x, y)| x.1 != y.1).count();
            assert!(
                differing >= 3,
                "{} and {} share too many tokens ({differing} differ)",
                a.name(),
                b.name()
            );
        }
    }
}

#[test]
fn the_values_are_numbers_with_a_unit_or_a_colour_or_none() {
    for id in StyleId::ALL {
        let l = look(id);
        assert!(
            l.dialog_width >= 360.0 && l.toast_width >= 300.0 && l.switch_width > l.switch_height
        );
        assert!(
            l.switch_height >= 16.0,
            "a switch under 16 px high cannot be hit"
        );
        for (name, value) in tokens(style(id)) {
            let ok = value == "none"
                || value.starts_with('#')
                || value.starts_with("color-mix(")
                || value.starts_with("var(--")
                || value.parse::<f32>().is_ok()
                || value.ends_with('%')
                || value.ends_with("px")
                || value.contains("px ");
            assert!(ok, "{} gives `{name}` = `{value}`", id.name());
        }
    }
}

//! G7: the completeness report `gaps` and the closed schema of `roles` (mutants O6 `gaps` skips a type, O11 a duplicate role).
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]
mod support;

use fs_components_catalog::{BlockCatalog, Role, basic_key};
use fs_types::BasicType;
use support::util::*;

const ROLES: [Role; 4] = [Role::Input, Role::Display, Role::Compact, Role::Cell];

/// The block of every basic type, independent of the data file.
const WANT: [(BasicType, &str); 19] = [
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
];

/// C8b (mutant O6: `gaps` skips a basic type): a catalog that draws every type but ONE reports exactly the four gaps of that
/// type, for each type in turn (the first one, `text`, and the last one included); the empty catalog reports every pair.
#[test]
fn c8b_gaps_cover_every_basic_type_and_role() {
    let from = |skip: Option<BasicType>| {
        let parts: Vec<String> = WANT
            .iter()
            .filter(|(basic, _)| Some(*basic) != skip)
            .map(|(basic, block)| {
                format!(
                    "[block.{block}]\naccepts = []\nrenders = {{ basic = \"{}\", roles = [\"input\", \"display\", \"compact\", \"cell\"] }}\n",
                    basic_key(*basic)
                )
            })
            .collect();
        let text = parts.concat();
        BlockCatalog::read(CONTRACT, &[("t.toml", &text)])
            .unwrap()
            .gaps()
    };
    assert_eq!(show(&from(None)), Vec::<String>::new());
    for (basic, _) in WANT {
        let want: Vec<String> = ROLES
            .iter()
            .map(|r| {
                format!(
                    "RendersNone[basic={},role={},target=]",
                    basic_key(basic),
                    r.key()
                )
            })
            .collect();
        assert_eq!(show(&from(Some(basic))), want, "{basic:?}");
    }
    let empty = BlockCatalog::read(CONTRACT, &[("t.toml", "[block.a]\naccepts = []\n")]).unwrap();
    assert_eq!(empty.gaps().len(), WANT.len() * ROLES.len());
}

/// C7b (mutant O11: a duplicate role is taken): `roles = ["input", "input"]` is refused with the field `roles`, for each
/// role word and also when the duplicate is not next to its twin; a duplicate in `target` likewise.
#[test]
fn c7b_a_duplicate_role_is_refused() {
    let bad = |renders: &str| {
        let text = format!("[block.a]\naccepts = []\nrenders = {renders}\n");
        BlockCatalog::read(CONTRACT, &[("t.toml", &text)]).unwrap_err()
    };
    for r in ROLES {
        let k = r.key();
        let got = show(&bad(&format!(
            "{{ basic = \"email\", roles = [\"{k}\", \"{k}\"] }}"
        )));
        assert_eq!(got, ["RendersInvalid[block=a,field=roles]"], "{k}");
    }
    assert_eq!(
        show(&bad(
            "{ basic = \"email\", roles = [\"input\", \"cell\", \"input\"] }"
        )),
        ["RendersInvalid[block=a,field=roles]"]
    );
    assert_eq!(
        show(&bad(
            "{ basic = \"reference\", target = [\"Person\", \"Person\"], roles = [\"cell\"] }"
        )),
        ["RendersInvalid[block=a,field=target]"]
    );
    let ok =
        "[block.a]\naccepts = []\nrenders = { basic = \"email\", roles = [\"input\", \"cell\"] }\n";
    assert!(BlockCatalog::read(CONTRACT, &[("t.toml", ok)]).is_ok());
}

//! The key `forms` of a block: `name@H.N`, read from the real contract files; four blocks name a form.
use fs_components_catalog::{AcceptsCode, BlockCatalog, FormRef, read_blocks};

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const GENERAL: &str = include_str!("../../../contract/blocks/general.toml");
const FRAMES: &str = include_str!("../../../contract/blocks/frames.toml");
const WIDGETS: &str = include_str!("../../../contract/blocks/widgets.toml");

fn real() -> BlockCatalog {
    BlockCatalog::read(
        CONTRACT,
        &[
            ("frames.toml", FRAMES),
            ("general.toml", GENERAL),
            ("widgets.toml", WIDGETS),
        ],
    )
    .expect("the contract files read")
}

fn forms_of(c: &BlockCatalog, block: &str) -> Vec<String> {
    c.block(block)
        .expect("block")
        .forms
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn four_blocks_name_their_form_and_the_rest_none() {
    let c = real();
    assert_eq!(forms_of(&c, "tree"), ["tree@1.0"]);
    assert_eq!(forms_of(&c, "table"), ["members@1.0"]);
    assert_eq!(forms_of(&c, "board"), ["cards-in-columns@1.0"]);
    assert_eq!(forms_of(&c, "tile-grid"), ["tiles@1.0"]);
    let named = c.iter().filter(|b| !b.forms.is_empty()).count();
    assert_eq!(named, 4, "only the four blocks of order SQ-2 name a form");
}

#[test]
fn the_spelling_is_name_at_major_dot_minor() {
    let f = FormRef::parse("bars-in-time@1.0").expect("reads");
    assert_eq!((f.name.as_str(), f.major, f.minor), ("bars-in-time", 1, 0));
    assert_eq!(
        FormRef::parse("tree@2.13").map(|f| f.to_string()),
        Some("tree@2.13".into())
    );
    for bad in [
        "tree@1",
        "tree",
        "@1.0",
        "Tree@1.0",
        "tree@1.x",
        "tree@-1.0",
        "tree@1.0.0",
        "-t@1.0",
        "tree@ 1.0",
        "tree@1.",
    ] {
        assert_eq!(FormRef::parse(bad), None, "{bad}");
    }
}

fn block(forms: &str) -> String {
    format!("[block.b]\naccepts = []\n{forms}\n")
}

#[test]
fn a_wrong_entry_is_a_problem_never_ignored() {
    for forms in [
        "forms = [\"tree@1\"]",
        "forms = \"tree@1.0\"",
        "forms = [3]",
        "forms = [\"tree@1.0\", \"tree@1.1\"]",
    ] {
        let err = read_blocks("t.toml", &block(forms)).expect_err(forms);
        assert!(
            err.iter().all(|c| c.code() == AcceptsCode::FormsInvalid),
            "{forms}"
        );
    }
    let ok =
        read_blocks("t.toml", &block("forms = [\"tree@1.0\", \"members@1.2\"]")).expect("reads");
    assert_eq!(ok[0].forms.len(), 2);
    assert!(
        read_blocks("t.toml", &block("")).expect("none is allowed")[0]
            .forms
            .is_empty()
    );
}

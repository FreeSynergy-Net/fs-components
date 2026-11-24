//! The key `forms` of a block: `name@<req>` (V-3), read from the real contract files; twelve blocks name a form.
use fs_components_catalog::{AcceptsCode, BlockCatalog, FormRef, read_blocks};
use fs_types::interface::InterfaceVersion;

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const GENERAL: &str = include_str!("../../../contract/blocks/general.toml");
const FRAMES: &str = include_str!("../../../contract/blocks/frames.toml");
const WIDGETS: &str = include_str!("../../../contract/blocks/widgets.toml");
const PROGRAMS: &str = include_str!("../../../contract/blocks/programs.toml");

fn real() -> BlockCatalog {
    BlockCatalog::read(
        CONTRACT,
        &[
            ("frames.toml", FRAMES),
            ("general.toml", GENERAL),
            ("programs.toml", PROGRAMS),
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
fn twelve_blocks_name_their_form_and_the_rest_none() {
    let c = real();
    assert_eq!(forms_of(&c, "tree"), ["tree@^1.0.0"]);
    assert_eq!(forms_of(&c, "table"), ["rows@^1.0.0"]);
    assert_eq!(forms_of(&c, "board"), ["cards-in-columns@^1.0.0"]);
    assert_eq!(forms_of(&c, "tile-grid"), ["tiles@^1.0.0"]);
    assert_eq!(forms_of(&c, "program-card"), ["tiles@^1.1.0"]);
    assert_eq!(forms_of(&c, "file-tile"), ["tiles@^1.1.0"]);
    for b in ["program-row", "grid", "search-hit"] {
        assert_eq!(forms_of(&c, b), ["rows@^1.0.0"], "{b}");
    }
    for b in ["program-detail", "install-job"] {
        assert_eq!(forms_of(&c, b), ["single-record@^1.0.0"], "{b}");
    }
    assert_eq!(
        forms_of(&c, "detail-page"),
        ["single-record@^1.0.0", "fields-in-groups@^1.0.0"]
    );
    let named = c.iter().filter(|b| !b.forms.is_empty()).count();
    assert_eq!(named, 12, "only these blocks name a form (SQ-2 and A2)");
}

#[test]
fn the_spelling_is_name_at_a_cargo_need() {
    let f = FormRef::parse("bars-in-time@^1.0.0").expect("reads");
    assert_eq!(f.name(), "bars-in-time");
    // the generator's full form, with a comma
    let range = FormRef::parse("tree@>=2.13.0, <3.0.0").expect("reads");
    assert_eq!(range.to_string(), "tree@>=2.13.0, <3.0.0");
    for bad in [
        "tree@1",
        "tree",
        "@1.0.0",
        "Tree@^1.0.0",
        "tree@1.x",
        "tree@1.0", // the old two-part spelling
        "tree@1.0.0-rc.1",
        "-t@^1.0.0",
        "t.ree@^1.0.0",
        "tree@*",
        "tree@>=1.0.0 <2.0.0", // no comma
        "tree@^1.",
    ] {
        assert_eq!(FormRef::parse(bad), None, "{bad}");
    }
}

/// The need is met by the version of the catalog (`fs-interfaces` `forms.txt`): `cards-in-columns@^1.0.0` draws 1.1.0, not 2.0.0.
#[test]
fn a_form_is_drawn_when_its_version_meets_the_need() {
    let need = FormRef::parse("cards-in-columns@^1.0.0").expect("reads");
    let at = |t: &str| t.parse::<InterfaceVersion>().expect("version");
    assert!(need.draws(&at("cards-in-columns@1.1.0")));
    assert!(!need.draws(&at("cards-in-columns@2.0.0")));
    assert!(!need.draws(&at("tiles@1.0.0")));
}

fn block(forms: &str) -> String {
    format!("[block.b]\naccepts = []\n{forms}\n")
}

#[test]
fn a_wrong_entry_is_a_problem_never_ignored() {
    for forms in [
        "forms = [\"tree@1\"]",
        "forms = [\"tree@1.0\"]",
        "forms = \"tree@^1.0.0\"",
        "forms = [3]",
        "forms = [\"tree@^1.0.0\", \"tree@^1.1.0\"]",
    ] {
        let err = read_blocks("t.toml", &block(forms)).expect_err(forms);
        assert!(
            err.iter().all(|c| c.code() == AcceptsCode::FormsInvalid),
            "{forms}"
        );
    }
    let ok = read_blocks(
        "t.toml",
        &block("forms = [\"tree@^1.0.0\", \"members@^1.2.0\"]"),
    )
    .expect("reads");
    assert_eq!(ok[0].forms.len(), 2);
    let none = read_blocks("t.toml", &block("")).expect("none is allowed");
    assert_eq!(none[0].forms, Vec::<FormRef>::new());
}

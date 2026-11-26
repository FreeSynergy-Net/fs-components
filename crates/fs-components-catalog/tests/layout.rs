//! A5 + A5b, C2: the layout keys of a block (`places`, `user`, `sizing`, `reads`, `sets`, `defaults`) and the closed vocabulary.
mod support;

use fs_components_catalog::{
    AcceptsCode, BlockCatalog, Max, PlaceOrder, Sizing, Takes, UserAct, Vocabulary, read_blocks_in,
};
use support::util::*;

const VOCAB: &str = "[contract]\nversion = \"0.6.0\"\n[vocabulary]\nstates = [\"default\", \"hidden\"]\n\
    reads = [\"bounds\", \"baseline\"]\nsets = [\"min-width\", \"axis\", \"opacity\"]\n";

fn block(extra: &str) -> String {
    format!("[block.f]\naccepts = []\nparts = [\"root\", \"pane\", \"side\"]\n{extra}\n")
}

fn read(extra: &str) -> Result<BlockCatalog, Vec<fs_types::coded::Coded<AcceptsCode>>> {
    BlockCatalog::read(VOCAB, &[("t.toml", &block(extra))])
}

fn bad(extra: &str) -> Vec<String> {
    show(&read(extra).expect_err(extra))
}

/// The keys read: every field of a place, `user`, `sizing`, the attribute lists and the vocabulary.
#[test]
fn c2_reads_the_layout_keys() {
    let c = read(
        "places = [{ place = \"pane\", takes = [\"form\", \"frame\"], min = 2, max = \"n\", order = \"user\" },\
         { place = \"side\", takes = [\"any\"], min = 0, max = 1 }]\n\
         user = [\"resize\", \"float\"]\nsizing = \"fill\"\nreads = [\"bounds\"]\nsets = [\"axis\", \"min-width\"]\n\
         defaults = { min-width = \"12em\" }",
    )
    .unwrap();
    let b = c.need("f").unwrap();
    assert_eq!(b.places.len(), 2);
    assert_eq!(b.places[0].takes, [Takes::Form, Takes::Frame]);
    assert_eq!((b.places[0].min, b.places[0].max), (2, Max::Many));
    assert_eq!(b.places[0].order, Some(PlaceOrder::User));
    assert_eq!((b.places[1].max, b.places[1].order), (Max::Count(1), None));
    assert_eq!(b.user, [UserAct::Resize, UserAct::Float]);
    assert_eq!(b.sizing, Some(Sizing::Fill));
    assert_eq!(
        (b.reads.as_slice(), b.sets.len()),
        (["bounds".to_owned()].as_slice(), 2)
    );
    assert_eq!(
        b.defaults.get("min-width").map(String::as_str),
        Some("12em")
    );
    assert!(c.vocabulary().has_set("axis") && !c.vocabulary().has_set("bounds"));
    assert_eq!(c.vocabulary().states, ["default", "hidden"]);
}

/// A block with none of the keys has empty values (every old block).
#[test]
fn c2_no_keys_is_empty() {
    let c = read("").unwrap();
    let b = c.need("f").unwrap();
    assert!(b.places.is_empty() && b.user.is_empty() && b.sizing.is_none());
    assert!(b.reads.is_empty() && b.sets.is_empty() && b.defaults.is_empty());
}

/// Counter-probes: each wrong key is its own problem with the key as parameter.
#[test]
fn c2_refuses() {
    let place = |s: &str| format!("places = [{{ place = \"pane\", {s} }}]");
    // an unknown `takes` class
    assert_eq!(
        bad(&place("takes = [\"widget\"], min = 0, max = 1")),
        ["PlaceInvalid[block=f,place=pane,field=takes]"]
    );
    // `min` above `max`
    assert_eq!(
        bad(&place("takes = [\"form\"], min = 3, max = 2")),
        ["PlaceInvalid[block=f,place=pane,field=min]"]
    );
    // an unknown key of a place, `max = 0`, an unknown order
    assert!(
        bad(&place("takes = [\"form\"], min = 0, max = 1, colour = 1"))
            .iter()
            .any(|p| p.contains("colour"))
    );
    assert!(
        bad(&place("takes = [\"form\"], min = 0, max = 0"))
            .iter()
            .any(|p| p.contains("field=max"))
    );
    assert!(
        bad(&place(
            "takes = [\"form\"], min = 0, max = 1, order = \"random\""
        ))
        .iter()
        .any(|p| p.contains("field=order"))
    );
    // a place that is no part of the block
    assert_eq!(
        bad("places = [{ place = \"nowhere\", takes = [\"form\"], min = 0, max = 1 }]"),
        ["PlaceNotPart[block=f,place=nowhere]"]
    );
    // a place twice
    let twice = "places = [{ place = \"pane\", takes = [\"any\"], min = 0, max = 1 }, { place = \"pane\", takes = [\"any\"], min = 0, max = 1 }]";
    assert_eq!(bad(twice), ["PlaceInvalid[block=f,place=pane,field=place]"]);
    // an attribute that is not in the vocabulary: in `sets`, in `reads`, a read word in `sets`, twice, a default of a word not set
    assert_eq!(
        bad("sets = [\"flavour\"]"),
        ["AttrUnknown[block=f,field=sets,name=flavour]"]
    );
    assert_eq!(
        bad("reads = [\"axis\"]"),
        ["AttrUnknown[block=f,field=reads,name=axis]"]
    );
    assert_eq!(
        bad("sets = [\"bounds\"]"),
        ["AttrUnknown[block=f,field=sets,name=bounds]"]
    );
    assert_eq!(
        bad("sets = [\"axis\", \"axis\"]"),
        ["AttrUnknown[block=f,field=sets,name=axis]"]
    );
    assert_eq!(
        bad("sets = [\"axis\"]\ndefaults = { opacity = \"1\" }"),
        ["AttrUnknown[block=f,field=defaults,name=opacity]"]
    );
    // unknown words of `user`, `sizing`
    assert_eq!(
        bad("user = [\"fly\"]"),
        ["LayoutInvalid[block=f,field=user]"]
    );
    assert_eq!(
        bad("sizing = \"huge\""),
        ["LayoutInvalid[block=f,field=sizing]"]
    );
    // a contract without `[vocabulary]` knows no attribute (closed, not open)
    let e = BlockCatalog::read(
        "[contract]\nversion = \"0.6.0\"\n",
        &[("t.toml", &block("sets = [\"axis\"]"))],
    )
    .unwrap_err();
    assert_eq!(codes(&e), [AcceptsCode::AttrUnknown]);
}

/// `read_blocks_in` with a vocabulary from text; a damaged `[vocabulary]` is `file-invalid`.
#[test]
fn c2_vocabulary() {
    let v = Vocabulary::read(VOCAB).unwrap();
    assert!(read_blocks_in("t.toml", &block("sets = [\"axis\"]"), &v).is_ok());
    assert!(Vocabulary::read("[vocabulary]\nsets = [1]\n").is_none());
    let e = BlockCatalog::read(
        "[contract]\nversion = \"0.6.0\"\n[vocabulary]\nsets = 3\n",
        &[],
    )
    .unwrap_err();
    assert_eq!(
        show(&e),
        ["FileInvalid[file=contract.toml,field=vocabulary]"]
    );
}

/// The real contract: the frames have their places, every attribute is in the vocabulary, the states are in the list.
#[test]
fn c2_real_contract() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../contract");
    let rd = |p: &str| std::fs::read_to_string(format!("{dir}/{p}")).unwrap();
    let files: Vec<(String, String)> = [
        "dialogs", "fields", "frames", "general", "programs", "widgets",
    ]
    .iter()
    .map(|n| (format!("{n}.toml"), rd(&format!("blocks/{n}.toml"))))
    .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let c = BlockCatalog::read(&rd("contract.toml"), &refs).expect("the contract reads");
    let split = c.need("split").unwrap();
    assert_eq!((split.places[0].min, split.places[0].max), (2, Max::Many));
    assert_eq!(split.sets, ["axis", "sizes"]);
    let tabs = c.need("tabs").unwrap();
    assert_eq!(tabs.places[0].order, Some(PlaceOrder::User));
    let wb = c.need("workbench").unwrap();
    assert_eq!(wb.places.len(), 5);
    assert_eq!(
        wb.places[3].takes,
        [Takes::Form, Takes::Value, Takes::Frame]
    );
    assert_eq!(
        c.need("form-field")
            .unwrap()
            .defaults
            .get("min-width")
            .map(String::as_str),
        Some("12em")
    );
    assert_eq!(c.need("detail-page").unwrap().sizing, Some(Sizing::Fill));
    assert_eq!(c.need("contact-row").unwrap().sizing, Some(Sizing::Content));
    assert!(c.vocabulary().states.iter().any(|s| s == "withheld"));
    // every place is a part, every attribute is in the lists (also checked by the reader; here against the frames by name)
    for b in c.iter() {
        for p in &b.places {
            assert!(
                p.min
                    <= match p.max {
                        Max::Count(n) => n,
                        Max::Many => u32::MAX,
                    }
            );
        }
        for s in &b.sets {
            assert!(c.vocabulary().has_set(s), "{}: {s}", b.name);
        }
    }
}

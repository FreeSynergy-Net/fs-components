//! E1 of the draft "settings as data": `catalog/settings.json` into `Declaration`s (counter-probes G1 to G3, G5, G9) and
//! the rule that a file can set neither a lock nor an order of the layers.
mod support;

use fs_settings::{Catalog, CatalogFile, Declares, Kind, Place, Value};
use support::{name, text};

const FILE: &str = include_str!("data/settings.json");

fn parse(t: &str) -> Result<CatalogFile, fs_settings::Coded<fs_settings::SettingCode>> {
    CatalogFile::parse(t, "settings.json")
}

/// The fixture with one entry's text replaced (the first `from` only).
fn with(from: &str, to: &str) -> String {
    assert!(FILE.contains(from), "fixture lacks {from}");
    FILE.replacen(from, to, 1)
}

fn cause(t: &str) -> (&'static str, Option<String>) {
    let e = parse(t).unwrap_err();
    (name(&e), text(&e, "cause"))
}

/// The five kinds, group, place and `human-only` arrive in the declarations; the catalog takes them all.
#[test]
fn the_file_gives_the_declarations() {
    let f = parse(FILE).unwrap();
    assert_eq!(f.owner().name(), "open");
    let list = f.declarations().unwrap();
    assert_eq!(list.len(), 5);
    let by = |k: &str| list.iter().find(|d| d.key.as_str() == k).unwrap();
    let when = by("fs.open.when-open");
    assert_eq!(when.default, Value::Choice("ask".into()));
    assert!(matches!(&when.kind, Kind::Choice { keys } if keys.len() == 4));
    assert!(matches!(by("fs.open.ask-again").kind, Kind::Flag));
    assert!(by("fs.open.ask-again").human_only);
    assert_eq!(by("fs.open.recent").place, Place::Device);
    assert_eq!(by("fs.open.recent").kind, Kind::Whole { min: 0, max: 50 });
    assert!(matches!(by("fs.open.language").kind, Kind::Language));
    assert!(matches!(
        by("fs.open.title").kind,
        Kind::Text { max_chars: 40 }
    ));
    assert_eq!(by("fs.open.title").group.as_str(), "open");
    assert_eq!(Catalog::of(list).unwrap().iter().count(), 5);
}

#[test]
fn choice_key_follows_label_key() {
    let list = parse(FILE).unwrap().declarations().unwrap();
    let d = list
        .iter()
        .find(|d| d.key.as_str() == "fs.open.when-open")
        .unwrap();
    assert_eq!(d.choice_key("tab"), "setting-choice-fs-open-when-open-tab");
    assert_eq!(d.label_key(), "setting-label-fs-open-when-open");
}

/// G1: a field the reader does not know (a typo, `lock`) is refused on every level; nothing is declared.
#[test]
fn g1_unknown_field_is_refused() {
    let want = ("catalog-invalid", Some("field".to_owned()));
    for t in [
        with(r#""default": "ask""#, r#""defualt": "ask""#),
        with(
            r#""group": "open" },"#,
            r#""group": "open", "lock": "never" },"#,
        ),
        with(r#""version": "0.1","#, r#""version": "0.1", "extra": 1,"#),
        with(
            r#""type": "string", "enum""#,
            r#""type": "string", "pattern": "x", "enum""#,
        ),
    ] {
        assert_eq!(cause(&t), want, "{t}");
    }
    assert_eq!(cause("{ nonsense").1.as_deref(), Some("json"));
}

/// G2: a default that is not in the list is `default-invalid` with the key (from `Catalog::add`).
#[test]
fn g2_default_outside_the_list() {
    let e = parse(&with(r#""default": "ask""#, r#""default": "later""#)).unwrap_err();
    assert_eq!(name(&e), "default-invalid");
    assert_eq!(text(&e, "key").as_deref(), Some("fs.open.when-open"));
    let e = parse(&with(r#""default": true"#, r#""default": "yes""#)).unwrap_err();
    assert_eq!(name(&e), "default-invalid");
}

/// G3: a key of another owner in the file is refused; so is an owner that is no owner.
#[test]
fn g3_foreign_key_and_bad_owner() {
    let e = parse(&with("fs.open.title", "program.tasks.x")).unwrap_err();
    assert_eq!(
        (name(&e), text(&e, "cause").as_deref()),
        ("catalog-invalid", Some("owner"))
    );
    for owner in ["fs.opens", "nowhere.open", "open", "fs.Open"] {
        let (code, c) = cause(&with(
            r#""owner": "fs.open""#,
            &format!(r#""owner": "{owner}""#),
        ));
        assert_eq!(
            (code, c.as_deref()),
            ("catalog-invalid", Some("owner")),
            "{owner}"
        );
    }
}

/// A `schema` outside the closed list (`number`, a range with one end, an empty `enum`'s kin) is refused.
#[test]
fn schema_outside_the_closed_list() {
    for t in [
        with(
            r#""type": "integer", "minimum": 0, "#,
            r#""type": "number", "minimum": 0, "#,
        ),
        with(r#""minimum": 0, "#, ""),
        with(
            r#""type": "boolean""#,
            r#""type": "boolean", "maxLength": 3"#,
        ),
        with(r#""format": "bcp47""#, r#""format": "email""#),
    ] {
        assert_eq!(
            cause(&t),
            ("catalog-invalid", Some("schema".to_owned())),
            "{t}"
        );
    }
}

/// G5: the same key twice, in one file or in two files, is `duplicate-key`.
#[test]
fn g5_duplicate_key() {
    let twice = with(r#""key": "fs.open.title""#, r#""key": "fs.open.recent""#);
    assert_eq!(name(&parse(&twice).unwrap_err()), "duplicate-key");
    let mut all = Catalog::new();
    for d in parse(FILE).unwrap().declarations().unwrap() {
        all.add(d).unwrap();
    }
    let other = parse(FILE).unwrap().declarations().unwrap();
    let e = all.add(other[0].clone()).err().unwrap();
    assert_eq!(name(&e), "duplicate-key");
}

/// A file cannot set a lock or an order of the layers: no such field exists (`can_lock` is fs-access's), and the
/// declared keys are plain `SettingKey`s with the ordinary default layer only.
#[test]
fn a_file_cannot_lock_or_rank() {
    for field in [
        "lock", "locked", "can_lock", "levels", "rank", "priority", "layer",
    ] {
        let t = with(
            r#""group": "open" },"#,
            &format!(r#""group": "open", "{field}": true }},"#),
        );
        assert_eq!(
            cause(&t),
            ("catalog-invalid", Some("field".to_owned())),
            "{field}"
        );
        let t = with(
            r#""default": "ask""#,
            &format!(r#""default": "ask", "{field}": "x""#),
        );
        assert_eq!(cause(&t).1.as_deref(), Some("field"), "{field} in schema");
    }
}

/// G9: every `schema` of the fixture is valid JSON Schema 2020-12 and accepts its own default.
#[test]
fn g9_schemas_are_json_schema_2020_12() {
    let v: serde_json::Value = serde_json::from_str(FILE).unwrap();
    let list = v["settings"].as_array().unwrap();
    assert_eq!(list.len(), 5);
    for e in list {
        let s = &e["schema"];
        assert!(jsonschema::draft202012::meta::is_valid(s), "{s}");
        assert!(
            jsonschema::draft202012::is_valid(s, &s["default"]),
            "default of {s}"
        );
    }
    assert!(!jsonschema::draft202012::meta::is_valid(
        &serde_json::json!({"type": "int"})
    ));
}

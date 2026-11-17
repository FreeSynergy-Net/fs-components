//! K3 and K4 of ENTWURF section 8: kinds and the catalog.
mod support;

use fs_setting::{
    Catalog, Declaration, GroupId, Kind, Layer, Owner, OwnerKind, Raw, SettingKey, Value,
};
use fs_types::{BasicValue, Language};
use support::{name, text};

fn k(local: &str) -> SettingKey {
    SettingKey::new(&Owner::new(OwnerKind::Framework, "t").unwrap(), local).unwrap()
}

fn code(kind: &Kind, raw: &Raw) -> &'static str {
    name(&kind.read(&k("x"), Layer::Person, raw).unwrap_err())
}

/// K3: every kind reads its own type and refuses the rest, with `key` and `layer`.
#[test]
fn k3_kind_read() {
    let key = k("x");
    let read = |kind: &Kind, raw: &Raw| kind.read(&key, Layer::Program, raw);
    assert_eq!(
        read(&Kind::Flag, &Raw::Bool(true)).unwrap(),
        Value::Flag(true)
    );
    let whole = Kind::Whole { min: 1, max: 7 };
    assert_eq!(read(&whole, &Raw::Int(1)).unwrap(), Value::Whole(1));
    assert_eq!(read(&whole, &Raw::Int(7)).unwrap(), Value::Whole(7));
    assert_eq!(code(&whole, &Raw::Int(0)), "out-of-range");
    assert_eq!(code(&whole, &Raw::Int(8)), "out-of-range");
    let choice = Kind::Choice {
        keys: vec!["a".to_owned(), "b".to_owned()],
    };
    assert_eq!(
        read(&choice, &Raw::Str("b".to_owned())).unwrap(),
        Value::Choice("b".to_owned())
    );
    assert_eq!(code(&choice, &Raw::Str("c".to_owned())), "unknown-choice");
    assert_eq!(
        read(&Kind::Language, &Raw::Str("DE-at".to_owned())).unwrap(),
        Value::Language(Language::parse("de-AT").unwrap())
    );
    assert_eq!(
        code(&Kind::Language, &Raw::Str("de_DE".to_owned())),
        "bad-language"
    );
    let t = Kind::Text { max_chars: 3 };
    assert!(read(&t, &Raw::Str("abc".to_owned())).is_ok());
    assert_eq!(code(&t, &Raw::Str("abcd".to_owned())), "text-too-long");
    assert_eq!(code(&t, &Raw::Str("a\nb".to_owned())), "text-too-long");
    assert_eq!(code(&Kind::Flag, &Raw::Str("yes".to_owned())), "wrong-type");
    let e = read(&whole, &Raw::Int(9)).unwrap_err();
    assert_eq!(text(&e, "key").as_deref(), Some("fs.t.x"));
    assert_eq!(text(&e, "layer").as_deref(), Some("program"));
}

/// K4: the catalog refuses a key twice and a default that does not fit; label keys.
#[test]
fn k4_catalog() {
    let group = || GroupId::new("g").unwrap();
    let flag = |local: &str| Declaration::new(k(local), Kind::Flag, Value::Flag(false), group());
    let mut c = Catalog::new();
    c.add(flag("a")).unwrap();
    assert_eq!(name(&c.add(flag("a")).unwrap_err()), "duplicate-key");
    let bad = Declaration::new(
        k("n"),
        Kind::Whole { min: 0, max: 1 },
        Value::Whole(5),
        group(),
    );
    assert_eq!(name(&c.add(bad).unwrap_err()), "default-invalid");
    assert_eq!(c.iter().count(), 1);
    let d = Declaration::new(
        SettingKey::new(
            &Owner::new(OwnerKind::Framework, "shell").unwrap(),
            "language",
        )
        .unwrap(),
        Kind::Flag,
        Value::Flag(true),
        group(),
    );
    assert_eq!(d.label_key(), "setting-label-fs-shell-language");
    assert_eq!(d.about_key(), "setting-about-fs-shell-language");
    assert!(Catalog::of(vec![flag("a"), flag("a")]).is_err());
}

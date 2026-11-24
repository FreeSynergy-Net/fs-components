//! Every `SettingCode` (32) has a sentence in `en.mf2` and `de.mf2` (ADR 0057), and its placeholders are exactly the
//! parameters the code carries. A code without a text, or a text with another placeholder, is red.
use std::collections::BTreeSet;

use fs_settings::{SettingCode, TEXT_DE, TEXT_EN};
use fs_types::coded::{ErrorCode, message_key};

/// The parameters each code carries (the codes' doc comments); `line` of `file-broken` is optional and not worded.
const PARAMS: &[(&str, &[&str])] = &[
    ("key-invalid", &["key"]),
    ("program-invalid", &["program"]),
    ("duplicate-key", &["key"]),
    ("default-invalid", &["key"]),
    ("not-declared", &["key"]),
    ("wrong-type", &["key", "layer"]),
    ("out-of-range", &["key", "layer"]),
    ("unknown-choice", &["key", "layer"]),
    ("bad-language", &["key", "layer"]),
    ("text-too-long", &["key", "layer"]),
    ("locked", &["key", "reason", "by"]),
    ("lock-refused", &["key"]),
    ("unknown-key", &["key", "layer"]),
    ("file-unreadable", &["file", "kind"]),
    ("file-broken", &["file"]),
    ("file-too-large", &["file"]),
    ("not-regular", &["file"]),
    ("write-failed", &["file", "kind"]),
    ("no-folder", &["which"]),
    ("no-right", &["key"]),
    ("profile", &["cause", "name"]),
    ("store-broken", &["file", "cause"]),
    ("store-newer", &["file", "found", "known"]),
    ("store-unreachable", &["socket"]),
    ("store-busy", &[]),
    ("read-only", &["which"]),
    ("index-changed", &[]),
    ("import-invalid", &["file", "line", "cause"]),
    ("import-conflict", &["key", "cause"]),
    ("export-failed", &["file", "kind"]),
    ("catalog-invalid", &["file", "cause"]),
    ("reason-missing", &["key"]),
];

fn line_of<'a>(text: &'a str, id: &str) -> Option<&'a str> {
    text.lines()
        .find_map(|l| l.strip_prefix(id).and_then(|r| r.strip_prefix(" = ")))
}

fn vars(sentence: &str) -> BTreeSet<String> {
    sentence
        .split("{$")
        .skip(1)
        .filter_map(|p| p.split_once('}').map(|(n, _)| n.to_owned()))
        .collect()
}

/// The problems of `en` and `de` against all codes (empty = fine).
fn problems(en: &str, de: &str) -> Vec<String> {
    let mut out = Vec::new();
    for c in SettingCode::ALL {
        let id = message_key(*c);
        let Some((_, want)) = PARAMS.iter().find(|(n, _)| *n == c.as_str()) else {
            out.push(format!("{id}: no row in PARAMS"));
            continue;
        };
        let want: BTreeSet<String> = want.iter().map(|s| (*s).to_owned()).collect();
        for (lang, text) in [("en", en), ("de", de)] {
            match line_of(text, &id) {
                None => out.push(format!("{id}: missing in {lang}")),
                Some(s) if s.trim().is_empty() => out.push(format!("{id}: empty in {lang}")),
                Some(s) if vars(s) != want => {
                    out.push(format!("{id}: placeholders in {lang} are {:?}", vars(s)));
                }
                Some(_) => {}
            }
        }
    }
    out
}

#[test]
fn every_code_has_en_and_de_with_its_parameters() {
    assert_eq!(SettingCode::ALL.len(), 32);
    assert_eq!(PARAMS.len(), 32);
    assert_eq!(problems(TEXT_EN, TEXT_DE), Vec::<String>::new());
}

/// Mutants: a text that is gone, empty, or has a wrong placeholder turns the check red.
#[test]
fn a_missing_empty_or_wrong_text_is_red() {
    let drop = |t: &str, id: &str| {
        t.lines()
            .filter(|l| !l.starts_with(&format!("{id} =")))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let en = drop(TEXT_EN, "setting-error-store-newer");
    assert_eq!(
        problems(&en, TEXT_DE),
        ["setting-error-store-newer: missing in en"]
    );
    let de = drop(TEXT_DE, "setting-error-read-only");
    assert_eq!(
        problems(TEXT_EN, &de),
        ["setting-error-read-only: missing in de"]
    );
    let wrong = TEXT_DE.replace(
        "(Version {$found}, dieses kennt {$known})",
        "(Version {$found})",
    );
    assert_eq!(problems(TEXT_EN, &wrong).len(), 1);
    let empty = TEXT_EN.replace(
        "setting-error-store-busy = The settings store is busy; try again.",
        "setting-error-store-busy = ",
    );
    assert_eq!(problems(&empty, TEXT_DE).len(), 1);
}

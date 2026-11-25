//! Tests: the words of the standard commands `app.undo` and `app.redo` (S3-9) exist in German AND English and are not the key.
//!
//! Pattern: Registry tests (fixed ids, fixed keys), like `frontend_texts.rs`.
//!
//! Intended mutant: a line `shell-undo` or `shell-redo` is missing in a language file or repeats its key.

use fs_components_app::TEXTS;
use fs_components_commands::standard::{REDO, UNDO};
use fs_components_commands::standard_commands;
use fs_components_mf2::{Mf2Texts, Source, check_pairs};
use fs_components_words::{Args, Locale, TextId, Words};

#[test]
fn undo_and_redo_have_their_words_in_both_languages() {
    let list = standard_commands();
    let keys: Vec<String> = [UNDO, REDO]
        .iter()
        .map(|id| list.get(&(*id).into()).unwrap().label.as_str().to_owned())
        .collect();
    assert_eq!(keys, ["shell-undo", "shell-redo"]);
    let unpaired = check_pairs(TEXTS);
    assert!(unpaired.is_empty(), "{unpaired:?}");
    for source in TEXTS {
        let only = [Source {
            language: source.language,
            text: source.text,
        }];
        let texts = Mf2Texts::new(&only, &Locale::parse(source.language).unwrap());
        for key in &keys {
            let said = texts.say(&TextId::new(key), &Args::default());
            assert!(said.is_found(), "no `{key}` in {}", source.language);
            assert_ne!(said.to_string(), *key, "`{key}` says its own key");
        }
    }
}

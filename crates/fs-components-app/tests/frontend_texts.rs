//! Tests: every text the command `layout.frontend` names exists in German AND English (the pair check only proves that de and en
//! agree with each other).
//!
//! Intended mutant (drawn, see BERICHT-frontend-c0.md): the label or the parameter label has no line in a language file.

use fs_components_app::TEXTS;
use fs_components_commands::{FrontendRow, frontend_command};
use fs_components_mf2::{Mf2Texts, Source, check_pairs};
use fs_components_words::{Args, Locale, TextId, Words};

#[test]
fn every_text_of_layout_frontend_exists_in_both_languages() {
    let c = frontend_command(&[FrontendRow::new("x")]);
    let keys = [
        c.label.as_str().to_owned(),
        c.params[0].label.as_str().to_owned(),
    ];
    assert_eq!(keys, ["shell-frontend", "shell-frontend-param"]);
    let unpaired = check_pairs(TEXTS);
    assert!(unpaired.is_empty(), "{unpaired:?}");
    for source in TEXTS {
        let only = [Source {
            language: source.language,
            text: source.text,
        }];
        let texts = Mf2Texts::new(&only, &Locale::parse(source.language).unwrap());
        for key in &keys {
            assert!(
                texts.say(&TextId::new(key), &Args::default()).is_found(),
                "no `{key}` in {}",
                source.language
            );
        }
    }
}

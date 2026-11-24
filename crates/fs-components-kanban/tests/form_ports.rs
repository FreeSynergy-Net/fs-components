//! F2(A): the two ports the host uses instead of knowing the board: the carried channels (`carried_channels`) and the words
//! (`WORD_KEYS`, `WORD_TEXTS`).
//!
//! Mutants (each makes the named test fail): a key dropped from `WORD_KEYS` -> `every_key_the_adapter_asks_for_is_listed`; a key in
//! the list without a sentence in one language -> `every_key_has_a_sentence_in_both_languages`; a German sentence left as English or
//! as its id -> `german_differs_from_english`; a refusal code added to the code without the list -> `every_refusal_code_has_a_key`;
//! a channel dropped from `carried_channels` (e.g. `before`) -> `the_gestures_carry_these_channels`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

use fs_components_kanban::{REFUSED_CODES, WORD_KEYS, WORD_TEXTS, carried_channels, word_id};
use fs_components_mf2::{Mf2Texts, check_pairs};
use fs_components_words::Locale;

fn texts(lang: &str) -> Mf2Texts {
    Mf2Texts::new(WORD_TEXTS, &Locale::parse(lang).unwrap())
}

#[test]
fn the_two_language_files_are_a_pair() {
    assert_eq!(check_pairs(WORD_TEXTS), Vec::<String>::new());
}

#[test]
fn every_key_has_a_sentence_in_both_languages() {
    for lang in ["en", "de"] {
        let t = texts(lang);
        for key in WORD_KEYS {
            let id = word_id(key);
            let said = t.plain(&id);
            assert!(
                !said.is_empty() && said != id,
                "{lang}: {key} has no sentence"
            );
        }
    }
}

#[test]
fn german_differs_from_english() {
    let (en, de) = (texts("en"), texts("de"));
    for key in WORD_KEYS {
        let id = word_id(key);
        assert_ne!(en.plain(&id), de.plain(&id), "{key} is not translated");
    }
}

#[test]
fn every_key_the_adapter_asks_for_is_listed() {
    // Every `"kanban.…"` literal in the adapter sources (the refusal keys are built from the codes, checked below).
    let sources = [
        include_str!("../src/form/dialogs.rs"),
        include_str!("../src/form/source.rs"),
        include_str!("../src/form/view.rs"),
        include_str!("../src/form/ask.rs"),
        include_str!("../src/form/read.rs"),
        include_str!("../src/form/events.rs"),
    ];
    let mut found = 0;
    for src in sources {
        for part in src.split("\"kanban.").skip(1) {
            let key = format!("kanban.{}", part.split('"').next().unwrap());
            if key.contains('{') || key.ends_with('.') {
                continue;
            }
            found += 1;
            assert!(
                WORD_KEYS.contains(&key.as_str()),
                "{key} is asked for but not listed"
            );
        }
    }
    assert!(found > 5, "the scan found only {found} keys");
    // and the other way: no listed key that nothing asks for (the refusal keys are built from the codes)
    let all: String = sources.concat();
    for key in WORD_KEYS
        .iter()
        .filter(|k| !k.starts_with("kanban.refused."))
    {
        assert!(
            all.contains(&format!("\"{key}\"")),
            "{key} is listed but nothing asks for it"
        );
    }
}

#[test]
fn every_refusal_code_has_a_key() {
    for code in REFUSED_CODES {
        assert!(
            WORD_KEYS.contains(&format!("kanban.refused.{code}").as_str()),
            "{code}"
        );
    }
    // The codes the adapter makes itself (`locked`, `ask-unsupported`, `unsupported`) are among them.
    for own in ["locked", "unsupported", "ask-unsupported"] {
        assert!(REFUSED_CODES.contains(&own), "{own}");
    }
    // No stray key: a key outside the refusal family is not a code.
    assert_eq!(
        WORD_KEYS
            .iter()
            .filter(|k| k.starts_with("kanban.refused."))
            .count(),
        REFUSED_CODES.len()
    );
}

#[test]
fn the_gestures_carry_these_channels() {
    assert_eq!(
        carried_channels(),
        [
            "after", "before", "column", "create", "lane", "move", "title"
        ]
    );
}

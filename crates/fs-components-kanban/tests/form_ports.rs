//! F2(A): the two ports the host uses instead of knowing the board: the carried channels (`carried_channels`) and the words
//! (`WORD_KEYS`, `WORD_TEXTS`).
//!
//! Mutants (each makes the named test fail): a key dropped from `WORD_KEYS` -> `every_key_the_adapter_asks_for_is_listed`; a key in
//! the list without a sentence in one language -> `every_key_has_a_sentence_in_both_languages`; a German sentence left as English or
//! as its id -> `german_differs_from_english`; a refusal code added to the code without the list -> `every_refusal_code_has_a_key`;
//! a channel dropped from `carried_channels` (e.g. `before`) or `lane` put back into it -> `the_gestures_carry_these_channels` and
//! `lane_is_not_carried_for_move_but_a_new_card_in_a_laned_column_sends_it`; a `reason(.., "x")` literal in the adapter (also behind nested parens like
//! `reason(self.say.as_ref(), "x")`; none is found, in all of `src/form/*.rs`) -> `the_adapter_makes_no_refusal_code_itself`; a gesture
//! code (`gesture-unbound`) outside the list -> `every_refusal_code_has_a_key`; `ask-unsupported` dropped again -> `every_refusal_code_has_a_key`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

mod common;

use fs_components_core::form_write::{ASK_UNSUPPORTED, GESTURE_UNBOUND};
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
    // the adapter makes no code itself (the host answers with its codes: `locked`, `gesture-unbound`, ...); all are in the list.
    for own in ["locked", "unsupported", GESTURE_UNBOUND] {
        assert!(REFUSED_CODES.contains(&own), "{own}");
    }
    // `ask-unsupported` is a code of the port (core `form_write::ASK_UNSUPPORTED`); the host only passes the code, the sentence
    // is the adapter's (`kanban.refused.ask-unsupported`), in both languages.
    assert!(REFUSED_CODES.contains(&ASK_UNSUPPORTED));
    for lang in ["en", "de"] {
        let id = word_id("kanban.refused.ask-unsupported");
        assert_ne!(texts(lang).plain(&id), id, "{lang}");
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
    assert_eq!(carried_channels(), ["after", "before", "column", "title"]);
}

/// The host appends every carried name to EVERY call (`lane` = "" too), which would stop `move` from taking the lane from the
/// record of the card: `lane` stays out of the flat list until K5b, although a new card in a laned column sends it itself.
#[test]
fn lane_is_not_carried_for_move_but_a_new_card_in_a_laned_column_sends_it() {
    use std::sync::{Arc, Mutex};

    use fs_components_core::form_write::{FormWrite, Gesture, WriteArg, WriteDone, WriteRefused};
    use fs_components_fields::KeysAsText;
    use fs_components_kanban::{
        BoardEvents, Card, Column, FormEvents, MemoryBoard, MoveRequest, NewCard,
    };

    type Calls = Mutex<Vec<(String, Vec<String>)>>;
    struct Rec(Calls);
    impl FormWrite for Rec {
        fn can(&self, _: &str, _: Option<&str>) -> bool {
            true
        }
        fn may_fire(&self, gesture: Gesture<'_>, record: Option<&str>) -> Result<(), WriteRefused> {
            if self.can(common::channel_of(gesture), record) {
                Ok(())
            } else {
                Err(WriteRefused::new("locked"))
            }
        }

        fn fire(
            &self,
            gesture: Gesture<'_>,
            record: Option<&str>,
            args: &[WriteArg],
        ) -> Result<WriteDone, WriteRefused> {
            self.send(common::channel_of(gesture), record, args)
        }

        fn send(
            &self,
            c: &str,
            _: Option<&str>,
            a: &[WriteArg],
        ) -> Result<WriteDone, WriteRefused> {
            self.0
                .lock()
                .unwrap()
                .push((c.to_owned(), a.iter().map(|a| a.name.clone()).collect()));
            Ok(WriteDone::default())
        }
    }
    let rec = Arc::new(Rec(Mutex::default()));
    let mut laned = Column::new("b", "");
    laned.lane = Some("l".to_owned());
    laned.cards = vec![Card::new("x", "")];
    let events = FormEvents::new(
        rec.clone(),
        Arc::new(MemoryBoard::new(vec![laned])),
        Arc::new(KeysAsText),
    );
    let _ = events.moved(&MoveRequest::new("", "", "b", 0, None));
    let _ = events.new_card(&NewCard::new("b", "", false));
    let calls = rec.0.lock().unwrap();
    let args = |ch: &str| calls.iter().find(|c| c.0 == ch).unwrap().1.clone();
    assert!(
        !args("move").contains(&"lane".to_owned()),
        "move must not send lane"
    );
    assert!(args("create").contains(&"lane".to_owned()));
    assert!(!carried_channels().contains(&"lane".to_owned()));
}

/// The second argument of every `reason(say, <code>)` call, if it is a string literal (a variable is a code of the host).
fn reason_literals(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in src.match_indices("reason(") {
        if src[..at].ends_with("fn ") {
            continue;
        }
        let (mut depth, mut args, mut cur) = (1, Vec::new(), String::new());
        for c in src[at + "reason(".len()..].chars() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                break;
            }
            if c == ',' && depth == 1 {
                args.push(std::mem::take(&mut cur));
            } else {
                cur.push(c);
            }
        }
        args.push(cur);
        let code = args.get(1).map(|a| a.trim());
        if let Some(lit) = code
            .and_then(|c| c.strip_prefix('"'))
            .and_then(|c| c.strip_suffix('"'))
        {
            out.push(lit.to_owned());
        }
    }
    out
}

#[test]
fn reason_literals_are_read_past_nested_parens() {
    assert_eq!(
        reason_literals(
            "fn reason(a: &dyn Say) {} x(reason(self.say.as_ref(), \"x\")); reason(s, &r.code)"
        ),
        ["x"]
    );
}

#[test]
fn the_adapter_makes_no_refusal_code_itself() {
    let mut found = Vec::new();
    for src in [
        include_str!("../src/form/ask.rs"),
        include_str!("../src/form/carried.rs"),
        include_str!("../src/form/dialogs.rs"),
        include_str!("../src/form/events.rs"),
        include_str!("../src/form/mod.rs"),
        include_str!("../src/form/read.rs"),
        include_str!("../src/form/source.rs"),
        include_str!("../src/form/view.rs"),
        include_str!("../src/form/words.rs"),
    ] {
        found.extend(reason_literals(src));
    }
    assert_eq!(
        found,
        Vec::<String>::new(),
        "the adapter makes a code of its own (or the scan lost one)"
    );
}

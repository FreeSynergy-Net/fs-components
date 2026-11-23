//! Tests: names, messages (code with parameters), the codes and the texts (L8 of the settings: both languages, same variables).
//!
//! Intended mutants (none drawn unless marked `drawn`; BERICHT-a1.md lists what was drawn): a name with upper case, a slash or `..` is accepted (`names_are_words`); a message from a code loses its parameters
//! (`a_message_keeps_the_code_of_another_crate`); a code is missing from `ALL` or has no text (`every_code_has_its_text_in_both_languages`).
mod support;

use fs_components_mf2::{Source, check_pairs};
use fs_notification::{
    ActionId, Category, Message, NoteId, NotificationCode, NotificationSettings, Scope, TEXT_DE,
    TEXT_EN,
};
use fs_settings::{Declares, SettingCode};
use fs_types::coded::{Coded, ErrorCode, Param};

fn has(text: &str, id: &str) -> bool {
    text.lines().any(|l| l.starts_with(&format!("{id} =")))
}

#[test]
fn names_are_words() {
    for ok in ["n1", "a", "later", "a-b-c", "x9"] {
        assert!(NoteId::new(ok).is_ok(), "{ok}");
        assert!(ActionId::new(ok).is_ok());
        assert!(Category::new(ok).is_ok());
        assert!(Scope::new(ok).is_ok());
        assert!(Message::new(ok).is_ok());
    }
    for bad in [
        "",
        "N1",
        "a b",
        "../x",
        "a/b",
        "a..b",
        "-a",
        "a-",
        "a--b",
        "ü",
        &"x".repeat(65),
    ] {
        for r in [
            NoteId::new(bad).map(|_| ()),
            ActionId::new(bad).map(|_| ()),
            Category::new(bad).map(|_| ()),
            Scope::new(bad).map(|_| ()),
            Message::new(bad).map(|_| ()),
        ] {
            assert_eq!(
                r.unwrap_err().code(),
                NotificationCode::NameInvalid,
                "{bad:?}"
            );
        }
    }
    assert_eq!(
        Message::new("ok")
            .unwrap()
            .with_text("Bad Name", "x")
            .unwrap_err()
            .code(),
        NotificationCode::NameInvalid
    );
}

#[test]
fn a_message_keeps_the_code_of_another_crate() {
    let code = Coded::new(SettingCode::Locked)
        .with_text("key", "fs.look.style")
        .with_number("n", 3)
        .erase();
    let m = Message::from_code(&code).unwrap();
    assert_eq!(m.key(), "setting-error-locked");
    let params: Vec<(&str, &Param)> = m.params().collect();
    assert_eq!(
        params,
        [
            ("key", &Param::Text("fs.look.style".into())),
            ("n", &Param::Number(3))
        ]
    );
}

#[test]
fn redacting_masks_the_values_and_keeps_the_names_and_the_text() {
    let m = Message::new("hello")
        .unwrap()
        .with_text("name", "Maria")
        .unwrap()
        .with_number("n", 2)
        .unwrap();
    let r = m.redacted();
    assert_eq!(r.key(), "hello");
    let mask = Param::Text(fs_notification::MASK.into());
    assert_eq!(
        r.params().collect::<Vec<_>>(),
        [("name", &mask), ("n", &mask)],
        "names and order stay, every value (a number too) is masked"
    );
    assert_eq!(m.params().count(), 2, "the original is untouched");
    assert_eq!(
        m.params().next(),
        Some(("name", &Param::Text("Maria".into())))
    );
}

#[test]
fn every_code_has_its_text_in_both_languages() {
    let sources = [
        Source {
            language: "en",
            text: TEXT_EN,
        },
        Source {
            language: "de",
            text: TEXT_DE,
        },
    ];
    assert_eq!(check_pairs(&sources), Vec::<String>::new());
    let mut names: Vec<&str> = NotificationCode::ALL.iter().map(|c| c.as_str()).collect();
    for c in NotificationCode::ALL {
        let key = fs_types::coded::message_key(*c);
        assert!(has(TEXT_EN, &key) && has(TEXT_DE, &key), "{key}");
    }
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), NotificationCode::ALL.len(), "each code once");
    // the match in as_str is total: a new variant must be added to ALL (9 now)
    assert_eq!(NotificationCode::ALL.len(), 9);
    for d in NotificationSettings.declarations().unwrap() {
        for id in [d.label_key(), d.about_key()] {
            assert!(has(TEXT_EN, &id) && has(TEXT_DE, &id), "{id}");
        }
    }
}

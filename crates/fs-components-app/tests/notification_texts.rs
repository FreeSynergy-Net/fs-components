//! Tests: every text the notification commands name (the command labels, the parameter labels, the options of a choice) exists in
//! the shell texts, in German AND in English (review r1 S5; ADR 0106 point 3). `check_pairs` only proves that de and en agree with
//! each other, not that the commands find their words.
//!
//! Intended mutant (drawn, see BERICHT-a1.md r1): a label of `notification_commands()` has no line in the text files, or an option of
//! `urgency` has none (`every_text_of_the_notification_commands_exists_in_both_languages`).

use fs_components_app::TEXTS;
use fs_components_commands::{ParamType, notification_commands};
use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, TextId, Words};

fn keys() -> Vec<String> {
    let mut keys = Vec::new();
    for command in notification_commands().iter() {
        keys.push(command.label.as_str().to_owned());
        for param in &command.params {
            keys.push(param.label.as_str().to_owned());
            if let Some(ParamType::Choice { options }) = &param.ty {
                keys.extend(options.iter().cloned());
            }
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

#[test]
fn every_text_of_the_notification_commands_exists_in_both_languages() {
    let keys = keys();
    // 3 commands, the parameters (`title` ... `id`) and the 3 options of the urgency: the 13 `shell-notification-*` and the options
    assert_eq!(
        keys.iter()
            .filter(|k| k.starts_with("shell-notification-"))
            .count(),
        13,
        "{keys:?}"
    );
    for key in ["low", "normal", "critical"] {
        assert!(
            keys.iter().any(|k| k == key),
            "the option {key} is among them"
        );
    }
    for source in TEXTS {
        let only = [Source {
            language: source.language,
            text: source.text,
        }];
        let texts = Mf2Texts::new(&only, &Locale::parse(source.language).unwrap());
        for key in &keys {
            assert!(
                texts.say(&TextId::new(key), &Args::default()).is_found(),
                "no text `{key}` in {}",
                source.language
            );
        }
    }
}

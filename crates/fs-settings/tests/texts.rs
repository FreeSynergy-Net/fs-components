//! L8: the text files of the crate (MessageFormat 2) are sound and complete.
mod support;

use fs_components_mf2::{Source, check_pairs};
use fs_settings::{Declares, TEXT_DE, TEXT_EN};
use support::shell;

fn has(text: &str, id: &str) -> bool {
    text.lines().any(|l| l.starts_with(&format!("{id} =")))
}

/// L8: the text files are sound (every id in both languages, same variables) and hold the five standard settings and
/// `reason-missing`.
#[test]
fn l8_texts_are_sound_and_complete() {
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
    let decls = shell().declarations().unwrap();
    assert_eq!(decls.len(), 6);
    for d in &decls {
        for id in [d.label_key(), d.about_key()] {
            assert!(has(TEXT_EN, &id) && has(TEXT_DE, &id), "{id}");
        }
    }
    assert!(
        has(TEXT_EN, "setting-error-reason-missing")
            && has(TEXT_DE, "setting-error-reason-missing")
    );
}

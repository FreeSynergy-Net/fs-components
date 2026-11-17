//! The words: every id in German and English, every problem code has a sentence in both, parameters appear in it.
//!
//! Pattern: Model tests over the MF2 files. Mutants: an id removed from `de.mf2`; a problem code without a sentence; a variable
//! renamed in one language; a parameter not passed to the message.

use fs_components_filepicker::{Labels, Mode, Problem, ProblemCode, TEXTS};
use fs_components_mf2::check_pairs;
use fs_components_words::{Locale, Localized};

fn de() -> Labels {
    Labels::for_locale(&Locale::parse("de").unwrap())
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
}

#[test]
fn german_differs_from_english() {
    let (en, de) = (Labels::default(), de());
    for mode in [
        Mode::OpenFile,
        Mode::OpenFiles,
        Mode::SaveFile,
        Mode::Folder,
    ] {
        assert_ne!(en.title(mode), de.title(mode), "{mode:?}");
        assert_ne!(en.button(mode), de.button(mode), "{mode:?}");
    }
    assert_ne!(en.provisional, de.provisional);
    assert!(de.provisional.starts_with("Vorläufig"));
}

fn full(code: ProblemCode) -> Problem {
    Problem::new(code)
        .with("limit-ms", "5000")
        .with("detail", "DETAIL")
        .with("path", "/PATH")
}

#[test]
fn every_problem_code_has_a_sentence_in_both_languages() {
    for labels in [Labels::default(), de()] {
        for code in ProblemCode::ALL {
            let sentence = labels.problem(&full(code));
            assert!(!sentence.trim().is_empty(), "{code:?}");
            assert!(
                !sentence.contains("filepicker-problem"),
                "{code:?} is missing: {sentence}"
            );
        }
    }
}

#[test]
fn the_parameters_are_in_the_sentence() {
    let en = Labels::default();
    assert!(
        en.problem(&full(ProblemCode::PortalTimeout))
            .contains("5000")
    );
    assert!(
        en.problem(&full(ProblemCode::PortalDenied))
            .contains("DETAIL")
    );
    assert!(
        en.problem(&full(ProblemCode::PathMissing))
            .contains("/PATH")
    );
    assert!(en.problem(&full(ProblemCode::Io)).contains("DETAIL"));
    assert!(
        de().problem(&full(ProblemCode::PathUnreadable))
            .contains("/PATH")
    );
}

#[test]
fn the_names_of_the_codes_are_stable() {
    let names: Vec<&str> = ProblemCode::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(
        names,
        [
            "portal-unavailable",
            "portal-timeout",
            "portal-denied",
            "portal-failed",
            "io",
            "path-missing",
            "path-unreadable",
            "wrong-kind"
        ]
    );
}

#[test]
fn only_missing_dialogs_allow_the_fallback() {
    for code in ProblemCode::ALL {
        let expect = matches!(
            code,
            ProblemCode::PortalUnavailable | ProblemCode::PortalTimeout
        );
        assert_eq!(code.means_no_dialog(), expect, "{code:?}");
    }
}

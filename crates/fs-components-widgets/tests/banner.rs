//! `banner`: role by step, action, dismiss mark (key and click), hiding, the words in two languages.
//!
//! Tests: attribute reads on the built banner, an in-process pointer and keyboard.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, frame, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::banner::{Banner, Labels, TEXTS};
use fs_components_widgets::states::Severity;
use fs_components_words::{Locale, Localized};

type Log = Arc<Mutex<Vec<&'static str>>>;

fn banner(severity: Severity, log: &Log) -> Banner {
    let (act, gone) = (Arc::clone(log), Arc::clone(log));
    Banner::new(&Labels::default(), severity, "You are offline.")
        .id("bn")
        .action("Reconnect", move || act.lock().unwrap().push("action"))
        .on_dismiss(move || gone.lock().unwrap().push("dismissed"))
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.close, "Ausblenden");
    assert_eq!(Labels::default().close, "Dismiss");
}

#[test]
fn the_role_follows_the_step_and_is_not_the_landmark_banner() {
    let _g = serial();
    for (severity, role) in [
        (Severity::Info, "status"),
        (Severity::Ok, "status"),
        (Severity::Warn, "alert"),
        (Severity::Error, "alert"),
    ] {
        let tree = layout(banner(severity, &Log::default()).build(), 600.0, 80.0);
        assert_eq!(
            attr(&tree, "bn", "role").as_deref(),
            Some(role),
            "{severity:?}"
        );
    }
}

#[test]
fn the_action_runs_and_the_dismiss_mark_hides_it_by_key_or_click() {
    let _g = serial();
    let log = Log::default();
    let bn = banner(Severity::Warn, &log);
    let mut tree = layout(bn.build(), 600.0, 80.0);
    let mut input = Input::new();
    input.click(&mut tree, "bn-action");
    assert_eq!(*log.lock().unwrap(), ["action"]);
    assert_eq!(attr(&tree, "bn-close", "role").as_deref(), Some("button"));
    assert_eq!(
        attr(&tree, "bn-close", "aria-label").as_deref(),
        Some("Dismiss")
    );
    assert!(input.tab_to(&mut tree, "bn-close"));
    input.key(&mut tree, 13);
    frame(&mut tree, 600.0, 80.0);
    assert!(!bn.is_open());
    bn.show();
    frame(&mut tree, 600.0, 80.0);
    input.click(&mut tree, "bn-close");
    assert!(!bn.is_open());
    assert_eq!(*log.lock().unwrap(), ["action", "dismissed", "dismissed"]);
}

#[test]
fn a_banner_that_cannot_be_dismissed_has_no_mark_and_only_the_program_resolves_it() {
    let _g = serial();
    let log = Log::default();
    let bn = banner(Severity::Error, &log).dismissible(false);
    let mut tree = layout(bn.build(), 600.0, 80.0);
    assert!(tree.query_by_id("bn-close").is_none());
    assert!(!bn.dismiss(), "cannot be dismissed");
    assert!(bn.is_open());
    bn.resolve();
    frame(&mut tree, 600.0, 80.0);
    assert!(!bn.is_open());
    assert!(
        log.lock().unwrap().is_empty(),
        "resolved by the program: no dismissal callback"
    );
}

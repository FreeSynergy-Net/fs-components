//! The date fallback (ADR 0057, entscheide 9f): without date data the message STAYS and only the date or time value is
//! neutral; a broken data blob changes nothing and never panics. Each with its counter-probe.

use std::sync::Arc;

use fs_components_mf2::{DateData, Mf2Texts, NoDates, Source};
use fs_components_words::{Args, Locale, TextId, Words};

const THEN: i64 = 1_795_694_700; // 2026-11-26 12:05 UTC

const EN: &str = "\
at = Event: {$when :datetime}
day = on {$d :date}, at {$t :time}
plain = on {$d}
";
const DE: &str = "\
at = Termin am {$when :datetime}
day = am {$d :date} um {$t :time}
plain = am {$d}
";
const SOURCES: &[Source] = &[
    Source {
        language: "en",
        text: EN,
    },
    Source {
        language: "de",
        text: DE,
    },
];

/// Hands out bytes that are no date data.
struct Broken;
impl DateData for Broken {
    fn blob(&self, _tag: &Locale) -> Option<Arc<[u8]>> {
        Some(Arc::from(&b"this is not an ICU4X blob"[..]))
    }
}

fn say(dates: &dyn DateData, locale: &str, id: &str, args: &Args) -> String {
    Mf2Texts::with_dates(SOURCES, &Locale::parse(locale).unwrap(), dates)
        .say(&TextId::new(id), args)
        .text()
        .to_owned()
}

#[test]
fn without_date_data_the_message_stays_and_only_the_time_is_neutral() {
    let a = Args::new().date("when", THEN);
    assert_eq!(
        say(&NoDates, "de", "at", &a),
        "Termin am 2026-11-26 12:05",
        "German sentence, neutral time, not English and not the id"
    );
    assert_eq!(say(&NoDates, "en", "at", &a), "Event: 2026-11-26 12:05");
    let a = Args::new().date("d", THEN).time("t", THEN % 86_400);
    assert_eq!(say(&NoDates, "de", "day", &a), "am 2026-11-26 um 12:05");
}

#[test]
fn a_date_in_a_message_without_a_date_function_stays_neutral_too() {
    let a = Args::new().date("d", THEN);
    assert_eq!(say(&NoDates, "de", "plain", &a), "am 2026-11-26");
}

#[test]
fn a_broken_blob_gives_the_same_fallback_and_no_panic() {
    let a = Args::new().date("when", THEN);
    assert_eq!(say(&Broken, "de", "at", &a), "Termin am 2026-11-26 12:05");
    assert_eq!(say(&Broken, "en", "at", &a), "Event: 2026-11-26 12:05");
}

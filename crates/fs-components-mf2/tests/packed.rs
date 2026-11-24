//! The runtime way (no `compile`): texts from the catalogs of the packer, dates by the language from its date table.
//! Pattern: none (test). The data in `tests/data/packed` is the OUTPUT of `fs-release pack-texts` (see `tests/data/README.md`).
//! Each rule has its counter-probe: no table (fr) keeps the message and shows the date neutrally; a broken table does the
//! same and never panics; a damaged package is an `Err`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fs_components_mf2::{DateData, DateDir, LoadError, Mf2Texts, NoDates};
use fs_components_words::{Args, Locale, TextId, Words};

const THEN: i64 = 1_795_694_700; // 2026-11-26 12:05 UTC

fn packed() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/packed")
}

fn say(texts: &Mf2Texts, id: &str, args: &Args) -> String {
    texts.say(&TextId::new(id), args).text().to_owned()
}

fn texts(locale: &str, dates: &dyn DateData) -> Mf2Texts {
    Mf2Texts::from_packed(&packed(), &Locale::parse(locale).unwrap(), dates).expect("package loads")
}

/// The date tables of the package, but only for the languages named (so a language can be left without one).
struct Only(&'static [&'static str], DateDir);
impl DateData for Only {
    fn blob(&self, tag: &Locale) -> Option<Arc<[u8]>> {
        self.0
            .contains(&tag.as_str())
            .then(|| self.1.blob(tag))
            .flatten()
    }
}

/// A real table cut in half.
struct Cut;
impl DateData for Cut {
    fn blob(&self, tag: &Locale) -> Option<Arc<[u8]>> {
        let all = DateDir::new(packed()).blob(tag)?;
        Some(Arc::from(&all[..all.len() / 2]))
    }
}

/// Bytes that are no date table.
struct Broken;
impl DateData for Broken {
    fn blob(&self, _tag: &Locale) -> Option<Arc<[u8]>> {
        Some(Arc::from(&b"this is not a date table"[..]))
    }
}

#[test]
fn the_catalog_loads_without_compile_and_a_message_resolves() {
    let de = texts("de", &NoDates);
    assert_eq!(
        say(&de, "hello", &Args::new().text("name", "Welt")),
        "Hallo, Welt!"
    );
    assert_eq!(say(&de, "files", &Args::new().count("n", 3)), "3 Dateien");
    assert_eq!(say(&de, "files", &Args::new().count("n", 1)), "1 Datei");
}

#[test]
fn with_the_date_table_de_and_en_format_by_the_language() {
    let dates = DateDir::new(packed());
    let a = Args::new().date("when", THEN);
    assert_eq!(
        say(&texts("de", &dates), "at", &a),
        "Termin am 26.11.2026, 12:05"
    );
    assert_eq!(
        say(&texts("en", &dates), "at", &a),
        "Event: Nov 26, 2026, 12:05\u{202f}PM"
    );
    let a = Args::new().date("d", THEN);
    assert_eq!(say(&texts("de", &dates), "day", &a), "am 26.11.2026");
}

#[test]
fn a_language_without_a_table_keeps_its_message_and_shows_the_time_neutrally() {
    let dates = Only(&["de", "en"], DateDir::new(packed()));
    let a = Args::new().date("when", THEN);
    assert_eq!(
        say(&texts("fr", &dates), "at", &a),
        "Rendez-vous le 2026-11-26 12:05",
        "French sentence, neutral time, not English, not the id"
    );
    // The same package, a language that has its table, is not touched by it.
    assert_eq!(
        say(&texts("de", &dates), "at", &a),
        "Termin am 26.11.2026, 12:05"
    );
    let a = Args::new().time("t", THEN % 86_400);
    assert_eq!(say(&texts("fr", &NoDates), "clock", &a), "à 12:05");
}

#[test]
fn a_broken_table_is_the_same_fallback_and_no_panic() {
    let a = Args::new().date("when", THEN);
    assert_eq!(
        say(&texts("de", &Broken), "at", &a),
        "Termin am 2026-11-26 12:05"
    );
    assert_eq!(
        say(&texts("en", &Broken), "at", &a),
        "Event: 2026-11-26 12:05"
    );
    // A table cut in half is broken too.
    assert_eq!(
        say(&texts("de", &Cut), "at", &a),
        "Termin am 2026-11-26 12:05"
    );
}

#[test]
fn the_chain_falls_back_to_english_and_a_missing_id_is_the_id() {
    let de_at = texts("de-AT", &NoDates);
    assert_eq!(
        say(&de_at, "hello", &Args::new().text("name", "A")),
        "Hallo, A!"
    );
    let sv = texts("sv", &NoDates);
    assert_eq!(
        say(&sv, "hello", &Args::new().text("name", "A")),
        "Hello, A!",
        "no sv: English"
    );
    assert_eq!(sv.plain("nothing-here"), "nothing-here");
}

fn copy_package(to: &Path) {
    for e in std::fs::read_dir(packed()).unwrap().flatten() {
        if e.path().is_file() {
            std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}

#[test]
fn a_damaged_package_is_an_error_not_a_panic() {
    let en = Locale::parse("de").unwrap();
    let dir = tempdir("no-dir");
    assert!(matches!(
        Mf2Texts::from_packed(&dir.join("absent"), &en, &NoDates),
        Err(LoadError::Read(_))
    ));
    // An empty directory: no manifest.
    assert!(matches!(
        Mf2Texts::from_packed(&dir, &en, &NoDates),
        Err(LoadError::Read(_))
    ));
    // A manifest that is cut: damaged.
    copy_package(&dir);
    let m = std::fs::read(dir.join("manifest.mf2m")).unwrap();
    std::fs::write(dir.join("manifest.mf2m"), &m[..m.len() - 3]).unwrap();
    assert!(matches!(
        Mf2Texts::from_packed(&dir, &en, &NoDates),
        Err(LoadError::Manifest(_))
    ));
    // A catalog of another package (its manifest differs): damaged.
    std::fs::write(dir.join("manifest.mf2m"), &m).unwrap();
    let de = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .find(|e| e.file_name().to_string_lossy().starts_with("de."))
        .unwrap();
    std::fs::write(de.path(), b"not a catalog").unwrap();
    assert!(matches!(
        Mf2Texts::from_packed(&dir, &en, &NoDates),
        Err(LoadError::Catalog(_))
    ));
    // A package without the language and without English: nothing to say.
    let dir = tempdir("only-fr");
    std::fs::copy(packed().join("manifest.mf2m"), dir.join("manifest.mf2m")).unwrap();
    assert!(matches!(
        Mf2Texts::from_packed(&dir, &en, &NoDates),
        Err(LoadError::NoLanguage)
    ));
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-mf2-packed-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

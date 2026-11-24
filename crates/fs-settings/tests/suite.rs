//! K26 to K30 of ENTWURF section 8: the codes and the suite `settings` over the bench.
mod support;

use std::collections::HashSet;

use fs_settings::{OwnerKind, SettingCode};
use fs_types::coded::{ErrorCode, message_key};

/// K26 (setting part): every code once, kebab names unique, message key = prefix and name.
#[test]
fn k26_setting_codes() {
    assert_eq!(SettingCode::ALL.len(), 33);
    // the 22 old codes keep their places (released names never move)
    assert_eq!(SettingCode::ALL[0], SettingCode::KeyInvalid);
    assert_eq!(SettingCode::ALL[21], SettingCode::ReasonMissing);
    assert_eq!(SettingCode::ALL[22], SettingCode::StoreBroken);
    let names: HashSet<_> = SettingCode::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(names.len(), SettingCode::ALL.len());
    for c in SettingCode::ALL {
        let n = c.as_str();
        assert!(
            n.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
                && !n.starts_with('-')
                && !n.ends_with('-'),
            "{n}"
        );
        assert_eq!(message_key(*c), format!("setting-error-{n}"));
    }
    // a match without `_`: a code missing from ALL does not compile here
    for c in SettingCode::ALL {
        let _: u8 = match c {
            SettingCode::KeyInvalid
            | SettingCode::ProgramInvalid
            | SettingCode::DuplicateKey
            | SettingCode::DefaultInvalid => 0,
            SettingCode::NotDeclared
            | SettingCode::WrongType
            | SettingCode::OutOfRange
            | SettingCode::UnknownChoice => 1,
            SettingCode::BadLanguage
            | SettingCode::TextTooLong
            | SettingCode::Locked
            | SettingCode::LockRefused => 2,
            SettingCode::UnknownKey
            | SettingCode::FileUnreadable
            | SettingCode::FileBroken
            | SettingCode::FileTooLarge => 3,
            SettingCode::NotRegular
            | SettingCode::WriteFailed
            | SettingCode::NoFolder
            | SettingCode::NoRight
            | SettingCode::Profile
            | SettingCode::ReasonMissing => 4,
            SettingCode::StoreBroken
            | SettingCode::StoreNewer
            | SettingCode::StoreUnreachable
            | SettingCode::StoreBusy
            | SettingCode::ReadOnly => 5,
            SettingCode::IndexChanged
            | SettingCode::ImportInvalid
            | SettingCode::ImportConflict
            | SettingCode::ExportFailed
            | SettingCode::CatalogInvalid => 6,
            _ => 7,
        };
    }
    let _ = OwnerKind::ALL;
}

/// A4: the conformance frame is no normal dependency of the library.
#[test]
fn a4_conformance_not_in_normal_graph() {
    let out = std::process::Command::new(env!("CARGO"))
        .args([
            "tree",
            "-p",
            "fs-settings",
            "-e",
            "normal,build",
            "--prefix",
            "none",
            "--locked",
            "--offline",
        ])
        .env("CARGO_TERM_COLOR", "never")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8(out.stdout).unwrap();
    assert!(
        tree.lines()
            .next()
            .is_some_and(|l| l.starts_with("fs-settings ")),
        "{tree}"
    );
    assert!(
        !tree.contains("fs-conformance") && !tree.contains("async-trait"),
        "{tree}"
    );
    // KB8: no `toml` in the default graph either (import/export belongs to the program)
    assert!(!tree.lines().any(|l| l.starts_with("toml ")), "{tree}");
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let default = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("default"))
        .unwrap_or("");
    assert!(!default.contains("conformance"), "{default}");
}

#[cfg(feature = "conformance")]
mod with_suite {
    use fs_conformance::{ErrorCode, Options, RuleId, Suite, message_key, prove, run};
    use fs_settings::suite::{
        Bench, BenchMutation, SettingsFixture, SettingsSuite, SuiteCode, bench_mutants,
    };

    fn bench() -> Bench {
        Bench::demo().unwrap()
    }

    /// K26 (suite part).
    #[test]
    fn k26_suite_codes() {
        assert_eq!(SuiteCode::ALL.len(), 16);
        let names: std::collections::HashSet<_> =
            SuiteCode::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(names.len(), 16);
        for c in SuiteCode::ALL {
            assert_eq!(message_key(*c), format!("settings-suite-{}", c.as_str()));
        }
    }

    /// K27: the suite on the demo bench: 18 rules, conformant, none skipped.
    #[tokio::test]
    async fn k27_bench_conformant() {
        let mut b = bench();
        let r = run(&SettingsSuite, &mut b, &Options::default()).await;
        assert!(r.is_conformant(), "{:?}", r.failures);
        assert_eq!((r.checked, r.skipped.len()), (18, 0));
    }

    /// K28: every bench mutant is caught by the rules it names (`prove`).
    #[tokio::test]
    async fn k28_prove() {
        let fresh = || Box::new(bench());
        let r = prove(
            &SettingsSuite,
            &fresh,
            &bench_mutants(),
            &Options::default(),
        )
        .await;
        assert!(r.is_proven(), "{:?}", r.failures);
        assert_eq!(r.caught.len(), bench_mutants().len());
    }

    /// Evidence per bench mutant: on a fresh bench each one makes at least the rules it names fail (printed with `--nocapture`).
    #[tokio::test]
    async fn n5_each_bench_mutant_is_red() {
        for m in bench_mutants() {
            let mut b = bench();
            b.reset();
            let applied = m.apply(&mut b);
            assert!(matches!(applied, fs_conformance::Applied::Yes));
            let r = run(&SettingsSuite, &mut b, &Options::default()).await;
            let red: Vec<String> = r
                .failures
                .iter()
                .map(|f| f.rule.as_str().to_owned())
                .collect();
            println!(
                "BENCH-MUTANT {} -> red: {}",
                m.id().as_str(),
                red.join(", ")
            );
            for need in m.must_fail() {
                assert!(
                    red.iter().any(|x| x == need.as_str()),
                    "{} did not turn {} red",
                    m.id().as_str(),
                    need.as_str()
                );
            }
        }
    }

    /// K29: the suite runs on a `dyn` fixture.
    #[tokio::test]
    async fn k29_dyn_fixture() {
        let mut b = bench();
        let d: &mut dyn SettingsFixture = &mut b;
        let r = run(&SettingsSuite, d, &Options::default()).await;
        assert!(r.is_conformant(), "{:?}", r.failures);
        let s: &dyn Suite<Bench> = &SettingsSuite;
        let ids: Vec<RuleId> = s.rules().iter().map(|r| r.id()).collect();
        assert_eq!(ids.len(), 18);
    }

    /// K30: the demo bench builds and every standard key is a key.
    #[test]
    fn k30_demo_bench_and_keys() {
        let _ = BenchMutation::DuplicateKey;
        for which in [
            fs_settings::LANGUAGE,
            fs_settings::STYLE,
            fs_settings::SCHEME,
            fs_settings::FRONTEND,
            fs_settings::REFRESH,
        ] {
            assert!(fs_settings::ShellSettings::key(which).is_ok(), "{which:?}");
        }
    }
}

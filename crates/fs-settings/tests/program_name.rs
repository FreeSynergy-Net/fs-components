//! `ProgramName` is normalised (read tolerantly, write strictly): `fs-texts` and `texts` are one program.
mod support;

use support::{name, program};

/// `fs-texts` and `texts` are one program (read tolerantly, write strictly): the short form is kept; only one
/// prefix is cut; nothing left or `language` stays invalid.
#[test]
fn l10_program_name_is_normalised() {
    assert_eq!(program("fs-texts"), program("texts"));
    assert_eq!(program("fs-texts").as_str(), "texts");
    // a second prefix is refused (review A3)
    for twice in ["fs-fs-x", "fs-fs-"] {
        let e = fs_settings::ProgramName::new(twice).unwrap_err();
        assert_eq!(name(&e), "program-invalid", "{twice}");
    }
    // idempotent: the result never begins with `fs-`, and a second pass changes nothing (review A3)
    for good in ["texts", "fs-texts", "fs-x", "x"] {
        let p = program(good);
        assert!(!p.as_str().starts_with("fs-"), "{good}");
        assert_eq!(program(p.as_str()), p, "{good}");
    }
    for bad in [
        "fs-",
        "",
        "fs-language",
        "language",
        "Fs-texts",
        "FS-texts",
        "fs-Texts",
        "fs-a/b",
    ] {
        let e = fs_settings::ProgramName::new(bad).unwrap_err();
        assert_eq!(name(&e), "program-invalid", "{bad:?}");
        assert_eq!(support::text(&e, "program").as_deref(), Some(bad));
    }
    assert!(fs_settings::ProgramName::new(&format!("fs-{}", "a".repeat(33))).is_err());
    assert!(fs_settings::ProgramName::new(&format!("fs-{}", "a".repeat(32))).is_ok());
}

/// A5: the names Windows reserves for devices are no program names, also behind the prefix; their neighbours are.
#[test]
fn l12_windows_device_names_are_refused() {
    let devices: Vec<String> = ["con", "prn", "aux", "nul"]
        .iter()
        .map(|d| (*d).to_owned())
        .chain((1..=9).flat_map(|n| [format!("com{n}"), format!("lpt{n}")]))
        .collect();
    assert_eq!(devices.len(), 22);
    for d in &devices {
        for text in [d.clone(), format!("fs-{d}")] {
            let e = fs_settings::ProgramName::new(&text).unwrap_err();
            assert_eq!(name(&e), "program-invalid", "{text}");
        }
    }
    for fine in [
        "com0",
        "com10",
        "lpt0",
        "console",
        "nulx",
        "auxiliary",
        "fs-comx",
        "com",
    ] {
        assert!(fs_settings::ProgramName::new(fine).is_ok(), "{fine}");
    }
}

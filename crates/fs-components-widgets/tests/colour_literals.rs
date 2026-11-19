//! No colour literal outside `fs-components-styles` (ADR 0066: colours are tokens of a style, so a change of style changes them).
//!
//! Tests: reads the sources of every crate of the workspace (`crates/*/src/**/*.rs`) without the style crate, the unit tests
//! (everything after `#[cfg(test)]`) and comment lines, and looks for the ways a colour is written as a literal: `#rrggbb` in a
//! string, `0xRRGGBB` with 6 or 8 digits, `Rgba::hex(`, `Color::rgba(` and `Color::rgb(` with a number as the first argument, a
//! colour constant (`Color::WHITE`), `rgba(` and `rgb(` with a number in a CSS string. `Color::TRANSPARENT` is no colour value
//! (it is "no ground") and is allowed. A mutant test shows that the scan sees a literal when one is planted.

use std::fs;
use std::path::{Path, PathBuf};

/// Files that hold a number of the shape of a colour that is no colour: a bit mask, a size. Each has its reason.
const NOT_COLOURS: &[(&str, &str)] = &[
    (
        "fs-components-app/src/frame_log.rs",
        "a 32 bit mask of a size",
    ),
    ("fs-components-map/src/wire.rs", "a 32 bit mask of a value"),
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// The ways a colour literal looks in one line of production code.
fn literals_in(line: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    // `#rrggbb`, `#rgb`, `#rrggbbaa` in a string: a `#` followed by 3, 4, 6 or 8 hex digits and then no letter or digit
    let chars: Vec<char> = line.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c == '#' {
            let digits = chars[i + 1..].iter().take_while(|c| is_hex(**c)).count();
            let next = chars.get(i + 1 + digits);
            let clean = next.is_none_or(|n| !n.is_alphanumeric() && *n != '_');
            if matches!(digits, 3 | 4 | 6 | 8) && clean && !line.contains("#[") {
                found.push("hex string");
            }
        }
    }
    // `0xRRGGBB` (with underscores): 6 or 8 hex digits
    let mut rest = line;
    while let Some(at) = rest.find("0x") {
        let digits: String = rest[at + 2..]
            .chars()
            .take_while(|c| is_hex(*c) || *c == '_')
            .filter(|c| *c != '_')
            .collect();
        if matches!(digits.len(), 6 | 8) {
            found.push("0x literal");
        }
        rest = &rest[at + 2..];
    }
    for (needle, what) in [
        ("Rgba::hex(", "Rgba::hex"),
        ("Color::rgba(", "Color::rgba"),
        ("Color::rgb(", "Color::rgb"),
    ] {
        let mut from = 0;
        while let Some(at) = line[from..].find(needle) {
            let after = &line[from + at + needle.len()..];
            // a conversion of a colour that is already a value (`c.r`, `f32::from(...)`) is no literal
            if after.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
                found.push(what);
            }
            from += at + needle.len();
        }
    }
    for name in [
        "WHITE", "BLACK", "RED", "GREEN", "BLUE", "YELLOW", "GRAY", "GREY",
    ] {
        if line.contains(&format!("Color::{name}")) {
            found.push("colour constant");
        }
    }
    for needle in ["rgba(", "rgb("] {
        let mut from = 0;
        while let Some(at) = line[from..].find(needle) {
            let before_ok = !line[..from + at].ends_with("::")
                && !line[..from + at].ends_with(|c: char| c.is_alphanumeric());
            let after = &line[from + at + needle.len()..];
            if before_ok && after.trim_start().starts_with(|c: char| c.is_ascii_digit()) {
                found.push("css rgb");
            }
            from += at + needle.len();
        }
    }
    found
}

fn production_lines(text: &str) -> Vec<(usize, &str)> {
    let mut lines = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        lines.push((n + 1, line));
    }
    lines
}

#[test]
fn no_colour_literal_outside_the_style_crate() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    for entry in fs::read_dir(&crates).expect("crates dir").flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "fs-components-styles" {
            continue;
        }
        sources(&entry.path().join("src"), &mut files);
    }
    assert!(
        files.len() > 200,
        "the scan must see the workspace ({} files)",
        files.len()
    );
    let mut bad = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&crates)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if rel.ends_with("_tests.rs") || NOT_COLOURS.iter().any(|(f, _)| rel == *f) {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap();
        for (n, line) in production_lines(&text) {
            for what in literals_in(line) {
                bad.push(format!("{rel}:{n}: {what}: {}", line.trim()));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "colour literals outside the styles:\n{}",
        bad.join("\n")
    );
}

#[test]
fn the_scan_sees_a_planted_literal_and_lets_a_conversion_pass() {
    for planted in [
        r##"let a = "#12ab34";"##,
        "let b = 0x00F1_B531;",
        "let c = Rgba::hex(0x123456);",
        "let d = Color::rgba(0.1, 0.2, 0.3, 1.0);",
        "let e = Color::WHITE;",
        r#"let f = "rgba(0,0,0,0.12)";"#,
    ] {
        assert!(!literals_in(planted).is_empty(), "not seen: {planted}");
    }
    for fine in [
        "let a = Color::rgba(f32::from(c.r) / 255.0, 0.0, 0.0, 1.0);",
        "let b = Color::TRANSPARENT;",
        "#[derive(Debug)]",
        "let c = u32::MAX & 0xFFFF;",
        r#"let d = "var(--surface)";"#,
    ] {
        assert!(
            literals_in(fine).is_empty(),
            "wrongly seen: {fine}: {:?}",
            literals_in(fine)
        );
    }
}

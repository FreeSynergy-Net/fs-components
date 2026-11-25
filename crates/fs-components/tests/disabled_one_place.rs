//! Conformance rule K22 (test G7 of ENTWURF-unfertig-tooltip): the look and the ARIA state of a control that cannot be used are drawn in
//! ONE place, `fs-components-overlays/src/disabled/` (`Disabled::wrap`). `aria-disabled` and the class `fs-disabled` must not appear in
//! the code of any other crate source. Comment lines do not count.
//!
//! Pattern: none (tests). The rule is a scan over the sources: a new hand-made `aria-disabled` anywhere else turns the test red.

use std::fs;
use std::path::{Path, PathBuf};

/// The one place that may draw it.
const THE_PLACE: &str = "fs-components-overlays/src/disabled/";

/// Known leftovers that still draw `aria-disabled` by hand. U3b, open: they move to `Disabled::wrap` one by one; each entry is removed
/// when its file is converted. A NEW entry needs a reason in the review, the list may only shrink.
const KNOWN: [&str; 6] = [
    "fs-components-app/src/menu_line.rs", // U3b, open (menu line without reason)
    "fs-components-lists/src/listbox/view.rs", // U3b, open
    "fs-components-widgets/src/disclosure/parts.rs", // U3b, open
    "fs-components-widgets/src/dropdown/panel.rs", // U3b, open
    "fs-components-widgets/src/tabs/view.rs", // U3b, open
    "fs-components-widgets/src/toolbar/view.rs", // U3b, open
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `(relative path, line number)` of every code line (not a `//` comment) that holds `aria-disabled` or `fs-disabled`.
fn hits(crates: &Path) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let Ok(members) = fs::read_dir(crates) else {
        return found;
    };
    for member in members.flatten() {
        let mut files = Vec::new();
        sources(&member.path().join("src"), &mut files);
        for file in files {
            let rel = file
                .strip_prefix(crates)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&file).expect("source file is readable");
            for (n, line) in text.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                if line.contains("aria-disabled") || line.contains("fs-disabled") {
                    found.push((rel.clone(), n + 1));
                }
            }
        }
    }
    found
}

#[test]
fn aria_disabled_and_fs_disabled_are_drawn_only_by_the_disabled_module() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let all = hits(&crates);
    assert!(
        all.iter().any(|(p, _)| p.starts_with(THE_PLACE)),
        "the scan found nothing in {THE_PLACE}: the scan is broken, not the rule"
    );
    let stray: Vec<String> = all
        .iter()
        .filter(|(p, _)| !p.starts_with(THE_PLACE) && !KNOWN.contains(&p.as_str()))
        .map(|(p, n)| format!("{p}:{n}"))
        .collect();
    assert!(
        stray.is_empty(),
        "K22: `aria-disabled` / `fs-disabled` outside {THE_PLACE}, use `Disabled::wrap`: {stray:?}"
    );
}

#[test]
fn the_known_list_only_names_files_that_still_have_the_hit() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let all = hits(&crates);
    let stale: Vec<&str> = KNOWN
        .iter()
        .copied()
        .filter(|k| !all.iter().any(|(p, _)| p == k))
        .collect();
    assert!(
        stale.is_empty(),
        "converted already, remove from KNOWN: {stale:?}"
    );
}

//! The mono face Craft falls back to.
//!
//! Tests: the lookup over the machine's fonts. With no mono font installed the lookup test is RED when `CI` is set and says
//! SKIPPED (no proof) otherwise; `mono_font_available` is `#[ignore]` and always red without one: the CI runs it with
//! `--include-ignored` (or `-- --ignored`).

use fs_components_styles::styles::craft::MONO;

#[test]
fn the_face_that_is_found_is_installed_and_monospaced() {
    let Some(face) = fs_components_render::installed_mono_face() else {
        assert!(
            std::env::var_os("CI").is_none(),
            "no monospace font installed on a CI machine: install one (for example fonts-dejavu-core or fonts-noto-mono)"
        );
        eprintln!(
            "SKIPPED: no monospace font installed (render/mono); this run is no proof that Craft finds a mono face"
        );
        return;
    };
    assert!(
        face.monospaced,
        "{} is found but the font database does not mark it monospaced",
        face.family
    );
}

#[test]
#[ignore = "needs an installed mono font: the CI runs it with --include-ignored"]
fn mono_font_available() {
    let face = fs_components_render::installed_mono_face();
    assert!(
        face.is_some(),
        "no monospace font installed (looked for {}): Craft would draw its labels in the default face",
        MONO.join(", ")
    );
}

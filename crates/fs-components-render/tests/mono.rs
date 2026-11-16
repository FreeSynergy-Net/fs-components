//! The mono face Craft falls back to.
//!
//! Tests: the lookup over the machine's fonts; with no mono font installed the test says SKIPPED and is no proof.

use fs_components_styles::styles::craft::MONO;

#[test]
fn some_mono_face_of_our_list_is_installed() {
    let Some(face) = fs_components_render::installed_mono_face() else {
        eprintln!(
            "SKIPPED: no monospace font installed (render/mono); this run is no proof that Craft finds a mono face"
        );
        return;
    };
    assert!(
        MONO.contains(&face),
        "{face} is not in the list of Craft's mono faces"
    );
}

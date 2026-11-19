//! The initials on a tile of a plan carry the ink class of their tile (`fs-plan-ink-N`, the colour is the token `--plan-tile-text-N`
//! of the style: white or black, whichever reads on the tile).
//!
//! Tests: class reads on the built circle, one per person.

mod common;

use common::{layout, serial};
use fs_components_styles::styles::craft::accent_index;
use fs_components_widgets::plan::initials_circle;

#[test]
fn the_initials_have_the_ink_class_of_their_tile() {
    let _g = serial();
    for name in [
        "Ada Lovelace",
        "Bob Marley",
        "Cy Twombly",
        "Di Prince",
        "Eve Online",
    ] {
        let tile = accent_index(name);
        let tree = layout(initials_circle(name), 120.0, 60.0);
        let reg = tree.element_registry();
        let root = tree.root().expect("a root");
        let all: Vec<_> = std::iter::once(root)
            .chain(reg.get_descendants(root))
            .collect();
        assert!(
            all.iter()
                .any(|n| reg.has_class(*n, &format!("fs-plan-ink-{tile}"))),
            "{name}: no ink class for tile {tile}"
        );
        assert!(
            all.iter()
                .any(|n| reg.has_class(*n, &format!("fs-plan-tile-{tile}"))),
            "{name}: no tile class {tile}"
        );
    }
}

#[test]
fn every_ink_class_reads_the_token_of_its_own_tile() {
    // Mutant P2 of the run (the ink of tile 3 read the token of tile 2) survived: only the class was read, not the rule behind it.
    let sheet = fs_components_widgets::plan::STYLES;
    for n in 0..7 {
        assert!(
            sheet.contains(&format!(
                ".fs-plan-ink-{n} {{ color: var(--plan-tile-text-{n}); }}"
            )),
            "ink {n} does not read its own token"
        );
        assert!(
            sheet.contains(&format!(
                ".fs-plan-tile-{n} {{ background: var(--plan-tile-{n}); }}"
            )),
            "tile {n} does not read its own token"
        );
    }
}

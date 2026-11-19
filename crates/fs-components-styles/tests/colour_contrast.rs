//! The ink laid on the coloured grounds of a style reads on them in EVERY style: the initials on the tiles of a plan (text, 4.5:1) and
//! the sign on a pinned folder (a graphic, 3:1, WCAG 1.4.11). A style that lightens its colours (Apple, Craft) must not keep white ink.
//!
//! Tests: model tests over the tokens of each style (the tile and its ink are read from the `:root` sheet of the style, as the
//! widgets read them, and from `pinned_sign`); a counter-test shows that a fixed white would fall through.

use fs_components_styles::styles::colour_roles::{
    IconKind, folder_top, kind_colour, pinned_sign, readable_on, sheet,
};
use fs_components_styles::{Rgba, StyleId};

const TEXT: f32 = 4.5;
const GRAPHIC: f32 = 3.0;
const WHITE: Rgba = Rgba::hex(0x00FF_FFFF);

/// The colour of the variable `--name` in the `:root` sheet of `id` (`#rrggbb`).
fn token(id: StyleId, name: &str) -> Rgba {
    let css = sheet(id);
    let at = css
        .find(&format!("--{name}: #"))
        .unwrap_or_else(|| panic!("{} gives no --{name}", id.name()));
    let hex = &css[at + name.len() + 5..at + name.len() + 11];
    Rgba::hex(u32::from_str_radix(hex, 16).expect("six hex digits"))
}

const PINNED: [IconKind; 6] = [
    IconKind::Desktop,
    IconKind::Downloads,
    IconKind::Documents,
    IconKind::Pictures,
    IconKind::Music,
    IconKind::Videos,
];

#[test]
fn the_initials_read_on_every_tile_in_every_style() {
    for id in StyleId::ALL {
        for n in 0..7 {
            let tile = token(id, &format!("plan-tile-{n}"));
            let ink = token(id, &format!("plan-tile-text-{n}"));
            let c = ink.contrast(tile);
            assert!(c >= TEXT, "{} tile {n}: initials {c:.2}:1", id.name());
        }
    }
}

#[test]
fn the_sign_on_a_pinned_folder_reads_on_the_whole_ground_in_every_style() {
    for id in StyleId::ALL {
        for kind in PINNED {
            let ground = kind_colour(id, kind);
            let sign = pinned_sign(id, kind);
            for (place, g) in [
                ("kind colour", ground),
                ("lightest part", folder_top(ground)),
            ] {
                let c = sign.contrast(g);
                assert!(
                    c >= GRAPHIC,
                    "{} {kind:?}: sign {c:.2}:1 on the {place}",
                    id.name()
                );
            }
        }
    }
}

#[test]
fn a_fixed_white_would_fail_these_measures() {
    // The first version of the tokens: white everywhere. Some tile and some folder in some style is too pale for it.
    let tile_fails = StyleId::ALL
        .into_iter()
        .any(|id| (0..7).any(|n| WHITE.contrast(token(id, &format!("plan-tile-{n}"))) < TEXT));
    let folder_fails = StyleId::ALL.into_iter().any(|id| {
        PINNED
            .into_iter()
            .any(|kind| WHITE.contrast(folder_top(kind_colour(id, kind))) < GRAPHIC)
    });
    assert!(tile_fails && folder_fails, "the measure cannot fail");
}

#[test]
fn the_choice_between_white_and_black_keeps_the_better_one() {
    let pale = Rgba::hex(0x00F2_A516);
    let deep = Rgba::hex(0x0010_2030);
    assert_eq!(readable_on(&[pale]), Rgba::hex(0));
    assert_eq!(readable_on(&[deep]), WHITE);
    // over two grounds the least contrast decides: a ground that fades from pale to deep has no good ink for both ends,
    // the choice is the one whose worse end is better
    let both = readable_on(&[pale, deep]);
    let worse = |ink: Rgba| ink.contrast(pale).min(ink.contrast(deep));
    assert!(worse(both) >= worse(WHITE) && worse(both) >= worse(Rgba::hex(0)));
}

#[test]
fn the_lightest_part_of_a_folder_is_22_percent_toward_white_in_every_channel() {
    // Mutant C4 of the run (the red channel not lightened) survived: no test read the lightened colour itself.
    assert_eq!(
        folder_top(Rgba::hex(0)),
        Rgba {
            r: 56,
            g: 56,
            b: 56,
            a: 255
        }
    );
    let top = folder_top(Rgba::hex(0x0010_2030));
    let want = |c: u32| u8::try_from(c + (255 - c) * 220 / 1000).expect("a byte");
    assert_eq!((top.r, top.g, top.b), (want(0x10), want(0x20), want(0x30)));
}

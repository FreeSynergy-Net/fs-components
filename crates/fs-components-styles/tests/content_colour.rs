//! The reading of a CSS colour and the look of a content colour in every style, light and dark (ADR 0074 sections 2 and 9.4).
//!
//! Tests: tables. Same colour from four spellings; the list of what is no colour; for every style x appearance x colour: the ink
//! reads on the ground (4.5:1), the mark is the colour as given, `faint` exactly below 3:1 (then an outline), `quiet` reads too.

use fs_components_styles::StyleId;
use fs_components_styles::{ContentLook, Rgba, style};

#[test]
fn four_spellings_are_one_colour() {
    let red = Rgba::hex(0x00FF_0000);
    for text in ["red", "#f00", "rgb(255 0 0)", "hsl(0 100% 50%)", " RED "] {
        assert_eq!(Rgba::from_css(text), Some(red), "{text}");
    }
}

#[test]
fn what_is_no_colour_is_none() {
    for text in [
        "",
        "lane-a",
        "#12",
        "300",
        "transparent",
        "ffffff",
        "abc",
        "rgba(1 2 3 / 0)",
    ] {
        assert_eq!(Rgba::from_css(text), None, "{text:?}");
    }
}

#[test]
fn mix_is_per_mille_and_opaque() {
    let (b, w) = (Rgba::hex(0), Rgba::hex(0x00FF_FFFF));
    assert_eq!(b.mix(w, 0), b);
    assert_eq!(b.mix(w, 1000), w);
    assert_eq!(b.mix(w, 500), Rgba::hex(0x0080_8080));
    assert_eq!(b.mix(w.alpha(7), 1000).a, 255);
}

const COLOURS: [&str; 8] = [
    "#000", "#fff", "yellow", "navy", "#2a7de1", "red", "#eeeeee", "#111111",
];

#[test]
fn every_style_reads_every_colour() {
    for id in StyleId::ALL {
        for dark in [false, true] {
            let p = *style(id).palette(dark);
            let mut grounds = vec![p.ground, p.panel]
                .into_iter()
                .map(Rgba::css)
                .collect::<Vec<_>>();
            grounds.extend(COLOURS.iter().map(ToString::to_string));
            for text in &grounds {
                let colour = Rgba::from_css(text).expect("a colour");
                let look = ContentLook::of(colour, &p, p.content_tint);
                let at = format!("{} dark={dark} {text}", id.name());
                assert!(look.ink.contrast(look.ground) >= 4.5, "{at}: ink");
                assert_eq!(look.mark, colour, "{at}: mark is as given");
                assert_eq!(look.faint, colour.contrast(p.ground) < 3.0, "{at}: faint");
                assert_eq!(look.outline.is_some(), look.faint, "{at}: outline");
                if let Some(o) = look.outline {
                    assert!(o.contrast(p.ground) >= 4.5, "{at}: the outline reads");
                }
            }
        }
    }
}

#[test]
fn a_colour_equal_to_the_ground_is_faint_and_a_strong_one_is_not() {
    let p = *style(StyleId::Admin).palette(false);
    assert!(ContentLook::of(p.ground, &p, p.content_tint).faint);
    assert!(!ContentLook::of(Rgba::hex(0), &p, p.content_tint).faint);
}

#[test]
fn the_tint_is_the_token_of_every_style() {
    for id in StyleId::ALL {
        for dark in [false, true] {
            let t = style(id).palette(dark).content_tint;
            assert!((1..=1000).contains(&t), "{} dark={dark}: {t}", id.name());
        }
    }
}

#[test]
fn the_quiet_look_reads_and_is_not_a_status() {
    for id in StyleId::ALL {
        for dark in [false, true] {
            let p = *style(id).palette(dark);
            let q = ContentLook::quiet(&p, p.content_tint);
            let at = format!("{} dark={dark}", id.name());
            assert!(q.ink.contrast(q.ground) >= 4.5, "{at}: ink");
            assert_eq!((q.ground, q.mark), (p.hover, p.text_weak), "{at}");
            assert!(q.outline.is_none() && !q.faint, "{at}");
            assert_ne!(q.ground, p.error);
            assert_ne!(q.ground, p.success);
        }
    }
}

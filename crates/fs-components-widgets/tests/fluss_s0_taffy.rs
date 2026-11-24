//! Spike S0 (A), measuring only: does `taffy` alone (the version Blinc brings, via `blinc_layout::style`) give the flow lines of
//! ENTWURF-fluss-panels.md section C? The rule: wrap only when even the minimum widths do not fit; per line start at `min`, free
//! space first up to the wish `width` (shares by `width - min`), then by `grow` up to `max`, the rest by `justify` (centre).
//! The table is worked out BY HAND (`HAND`), a small reference function must agree with it, then taffy runs two ways:
//! `Min` = `flex-basis: min` + `flex-grow` + `max-width` + wrap; `Wish` = `flex-basis: width` + `flex-shrink: 1` + `min-width` + `max-width` + wrap.
//! Run with `--nocapture` to see the table.
//!
//! Pattern: none; tests (taffy as the oracle of a reference function).
#![allow(clippy::pedantic)]

use blinc_layout::style::{
    length, AvailableSpace, Dimension, Display, FlexDirection, FlexWrap, JustifyContent,
    LengthPercentageAuto, Size, Style, TaffyTree,
};

#[derive(Clone, Copy)]
struct It {
    min: f32,
    width: f32,
    max: f32,
    grow: f32,
}
const fn it(min: f32, width: f32, max: f32, grow: f32) -> It {
    It {
        min,
        width,
        max,
        grow,
    }
}

/// One line: the widths and the offset of its first element (centre).
type Line = (Vec<f32>, f32);

/// The rule of the draft, written out (the candidate for `flow::lines`).
fn lines(avail: f32, items: &[It], gap: f32) -> Vec<Line> {
    let mut groups: Vec<Vec<It>> = Vec::new();
    let mut used = 0.0;
    for i in items {
        let add = if groups.last().is_some_and(|g| !g.is_empty()) {
            gap + i.min
        } else {
            i.min
        };
        if groups.is_empty() || used + add > avail {
            groups.push(vec![*i]);
            used = i.min;
        } else {
            groups.last_mut().unwrap().push(*i);
            used += add;
        }
    }
    groups.iter().map(|g| spread(avail, g, gap)).collect()
}

fn spread(avail: f32, g: &[It], gap: f32) -> Line {
    let mut w: Vec<f32> = g.iter().map(|i| i.min).collect();
    let gaps = gap * (g.len() as f32 - 1.0);
    let mut free = (avail - w.iter().sum::<f32>() - gaps).max(0.0);
    let need: f32 = g.iter().map(|i| i.width - i.min).sum();
    if need > 0.0 {
        let give = free.min(need);
        for (x, i) in w.iter_mut().zip(g) {
            *x += give * (i.width - i.min) / need;
        }
        free -= give;
    }
    let mut frozen: Vec<bool> = w.iter().zip(g).map(|(x, i)| *x >= i.max).collect();
    while free > 1e-4 {
        let sum: f32 = g
            .iter()
            .zip(&frozen)
            .filter(|(_, f)| !**f)
            .map(|(i, _)| i.grow)
            .sum();
        if sum <= 0.0 {
            break;
        }
        let mut spent = 0.0;
        for k in 0..g.len() {
            if frozen[k] {
                continue;
            }
            let add = (free * g[k].grow / sum).min(g[k].max - w[k]);
            w[k] += add;
            spent += add;
            if w[k] >= g[k].max - 1e-4 {
                frozen[k] = true;
            }
        }
        free -= spent;
        if spent <= 1e-6 {
            break;
        }
    }
    let used = w.iter().sum::<f32>() + gaps;
    (w, ((avail - used) / 2.0).max(0.0))
}

#[derive(Clone, Copy, Debug)]
enum Basis {
    Min,
    Wish,
}

/// What taffy lays out: lines (grouped by `y`), widths and the x of the first element.
fn taffy_lines(avail: f32, items: &[It], gap: f32, basis: Basis) -> Vec<Line> {
    let mut t: TaffyTree<()> = TaffyTree::new();
    t.disable_rounding(); // taffy rounds to whole pixels by default (161 162 161); the rule is compared unrounded
    let kids: Vec<_> = items
        .iter()
        .map(|i| {
            t.new_leaf(Style {
                flex_basis: length(match basis {
                    Basis::Min => i.min,
                    Basis::Wish => i.width,
                }),
                flex_grow: i.grow,
                flex_shrink: 1.0,
                min_size: Size {
                    width: length(i.min),
                    height: LengthPercentageAuto::auto(),
                },
                max_size: Size {
                    width: length(i.max),
                    height: LengthPercentageAuto::auto(),
                },
                size: Size {
                    width: Dimension::auto(),
                    height: length(10.0),
                },
                ..Default::default()
            })
            .unwrap()
        })
        .collect();
    let root = t
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: Some(JustifyContent::CENTER),
                gap: Size {
                    width: length(gap),
                    height: length(0.0),
                },
                size: Size {
                    width: length(avail),
                    height: Dimension::auto(),
                },
                ..Default::default()
            },
            &kids,
        )
        .unwrap();
    t.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(avail),
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();
    let mut out: Vec<(f32, Line)> = Vec::new();
    for k in kids {
        let l = t.layout(k).unwrap();
        match out.last_mut() {
            Some((y, line)) if (*y - l.location.y).abs() < 0.01 => line.0.push(l.size.width),
            _ => out.push((l.location.y, (vec![l.size.width], l.location.x))),
        }
    }
    out.into_iter().map(|(_, l)| l).collect()
}

struct Case {
    name: &'static str,
    avail: f32,
    gap: f32,
    items: Vec<It>,
    /// By hand, from the rule: widths per line (the offset is checked by the reference only).
    hand: Vec<Vec<f32>>,
}

fn cases() -> Vec<Case> {
    let a = it(200.0, 300.0, 400.0, 1.0);
    vec![
        Case {
            name: "1 two fit their wish",
            avail: 600.0,
            gap: 0.0,
            items: vec![a, a],
            hand: vec![vec![300.0, 300.0]],
        },
        Case {
            name: "2 shrink below wish, one line",
            avail: 500.0,
            gap: 0.0,
            items: vec![a, a],
            hand: vec![vec![250.0, 250.0]],
        },
        Case {
            name: "3 wide: both to max",
            avail: 1000.0,
            gap: 0.0,
            items: vec![a, a],
            hand: vec![vec![400.0, 400.0]],
        },
        Case {
            name: "4 unequal wish, then grow",
            avail: 600.0,
            gap: 0.0,
            items: vec![it(100.0, 300.0, 600.0, 1.0), it(100.0, 200.0, 600.0, 1.0)],
            hand: vec![vec![350.0, 250.0]],
        },
        Case {
            name: "5 wrap by the minimum sum",
            avail: 450.0,
            gap: 0.0,
            items: vec![a, a, a],
            hand: vec![vec![225.0, 225.0], vec![400.0]],
        },
        Case {
            name: "6 gap 8, three, one line",
            avail: 500.0,
            gap: 8.0,
            items: vec![it(100.0, 150.0, 200.0, 1.0); 3],
            hand: vec![vec![161.0 + 1.0 / 3.0; 3]],
        },
        Case {
            name: "7 one item wider than the line",
            avail: 150.0,
            gap: 0.0,
            items: vec![a],
            hand: vec![vec![200.0]],
        },
        Case {
            name: "8 shrink first, wish 600 > 500",
            avail: 500.0,
            gap: 0.0,
            items: vec![it(100.0, 300.0, 600.0, 0.0); 2],
            hand: vec![vec![250.0, 250.0]],
        },
    ]
}

fn same(a: &[Line], b: &[Vec<f32>]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            x.0.len() == y.len() && x.0.iter().zip(y).all(|(p, q)| (p - q).abs() < 0.05)
        })
}

fn show(l: &[Line]) -> String {
    l.iter()
        .map(|(w, _)| {
            format!(
                "[{}]",
                w.iter()
                    .map(|x| format!("{x:.1}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_reference_function_agrees_with_the_hand_table() {
    for c in cases() {
        let got = lines(c.avail, &c.items, c.gap);
        assert!(
            same(&got, &c.hand),
            "{}: reference {} vs hand {:?}",
            c.name,
            show(&got),
            c.hand
        );
    }
}

#[test]
fn taffy_against_the_rule() {
    let mut diff = [0usize; 2];
    for c in cases() {
        println!(
            "{} (avail {}, gap {})  hand {:?}",
            c.name, c.avail, c.gap, c.hand
        );
        for (n, b) in [Basis::Min, Basis::Wish].into_iter().enumerate() {
            let t = taffy_lines(c.avail, &c.items, c.gap, b);
            let ok = same(&t, &c.hand);
            diff[n] += usize::from(!ok);
            println!(
                "    taffy {b:?}: {}  -> {}",
                show(&t),
                if ok { "same" } else { "DIFFERENT" }
            );
        }
    }
    println!(
        "cases different: basis=min {} / 8, basis=wish {} / 8",
        diff[0], diff[1]
    );
    // The measurement, held: neither mapping gives the rule in all cases, so taffy alone is not enough.
    assert!(
        diff[0] > 0 && diff[1] > 0,
        "if this fails taffy alone would do: read the table"
    );
}

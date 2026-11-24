//! `flow`: the lines of the frame `flow` as a pure function (draft "Fluss-Panels" part C).
//!
//! Panels in reading order and the available width in, lines with widths and offset out. Rule: wrap only when even the
//! minimum widths do not fit (shrink BEFORE wrap); per line start at `min`, give free space first up to the wish `width`
//! (shares by `width - min`, all reach it together), then by `grow` up to `max` (a panel that reaches `max` drops out, the
//! rest is shared anew, as in CSS Flexbox "resolve flexible lengths"); the remainder after `max` is placed by `justify`.
//! A panel whose `min` is wider than the line gets the line (it must manage with its narrowest shape; no horizontal scroll).
//!
//! Looked at first (BEFUND-flow-crates.md, 2026-11-24): morphorm 0.9.0 (wraps before shrinking), taffy 0.14/0.16 (wish/min
//! mapping wrong, S0 cases 2, 4, 5, 8), stretch (unmaintained), cassowary/kasuari (no wrapping), egui `horizontal_wrapped`
//! (no shrinking), `AdwWrapBox` `wrap-policy = minimum` (C/GTK/LGPL, template only), `iced_aw` (not read), slint/floem/masonry/
//! vello/`bevy_ui` (whole frameworks), unicode-linebreak (text). No crate gives shrink-before-wrap with min/width/max + grow +
//! justify as a pure function, so this is written here. taffy appears only as the oracle in the tests; this module does not use
//! it (taffy is already in the runtime tree through Blinc, not through us).
//!
//! Pattern: none – pure functions (arguments in, result out, no state, no Blinc type).

use crate::fit::Fit;

#[allow(clippy::cast_precision_loss)] // panel counts are tiny
const fn count(n: usize) -> f32 {
    n as f32
}

/// One panel as the flow sees it (taken from `Fit`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowItem {
    /// Smallest width.
    pub min: f32,
    /// Wished width.
    pub width: f32,
    /// Largest width.
    pub max: f32,
    /// Share of the free space after the wish.
    pub grow: f32,
}

impl From<&Fit> for FlowItem {
    fn from(f: &Fit) -> Self {
        Self {
            min: f.min,
            width: f.width,
            max: f.max,
            grow: f.grow,
        }
    }
}

/// Where the remainder after `max` goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    /// Panels at the start, the rest at the end.
    Start,
    /// Panels in the middle (default: an empty strip only on one side looks like an error).
    #[default]
    Center,
    /// Panels at the end.
    End,
}

/// Settings of the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlowOpts {
    /// Space between two panels of a line.
    pub gap: f32,
    /// Place of the remainder.
    pub justify: Justify,
}

/// One line: the panels `first .. first + widths.len()`.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Index of the first panel.
    pub first: usize,
    /// Width of each panel.
    pub widths: Vec<f32>,
    /// Space before the first panel (by `justify`).
    pub offset: f32,
}

fn clean(i: &FlowItem, avail: f32) -> FlowItem {
    let ok = |v: f32, d: f32| if v.is_finite() { v.max(0.0) } else { d };
    let min = ok(i.min, 0.0).min(avail);
    let max = if i.max.is_nan() {
        f32::INFINITY
    } else {
        i.max.max(min)
    };
    let width = ok(i.width, min).clamp(min, max);
    FlowItem {
        min,
        width,
        max,
        grow: ok(i.grow, 0.0),
    }
}

/// The lines for `avail` (lpx) and the panels in reading order. Empty input gives no lines.
///
/// A non-finite `avail` (NaN or infinite) counts as 0: every panel gets width 0, "unbounded" is not a frame.
pub fn lines(avail: f32, items: &[FlowItem], opts: &FlowOpts) -> Vec<Line> {
    let avail = if avail.is_finite() {
        avail.max(0.0)
    } else {
        0.0
    };
    let gap = if opts.gap.is_finite() {
        opts.gap.max(0.0)
    } else {
        0.0
    };
    let items: Vec<FlowItem> = items.iter().map(|i| clean(i, avail)).collect();
    let mut out = Vec::new();
    let (mut first, mut used) = (0, 0.0);
    for (k, i) in items.iter().enumerate() {
        if k > first && used + gap + i.min > avail {
            out.push(spread(avail, first, &items[first..k], gap, opts.justify));
            first = k;
            used = i.min;
        } else {
            used += if k > first { gap + i.min } else { i.min };
        }
    }
    if first < items.len() {
        out.push(spread(avail, first, &items[first..], gap, opts.justify));
    }
    out
}

fn spread(avail: f32, first: usize, g: &[FlowItem], gap: f32, justify: Justify) -> Line {
    let mut w: Vec<f32> = g.iter().map(|i| i.min).collect();
    let gaps = gap * (count(g.len()) - 1.0);
    let mut free = (avail - w.iter().sum::<f32>() - gaps).max(0.0);
    // (a) up to the wish, shares by `width - min`
    let need: f32 = g.iter().map(|i| i.width - i.min).sum();
    if need > 0.0 {
        let give = free.min(need);
        for (x, i) in w.iter_mut().zip(g) {
            *x += give * (i.width - i.min) / need;
        }
        free -= give;
    }
    // (b) by `grow` up to `max`, freeze whoever reaches `max`
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
            let add = (free * (g[k].grow / sum)).min(g[k].max - w[k]);
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
    let rest = (avail - w.iter().sum::<f32>() - gaps).max(0.0);
    let offset = match justify {
        Justify::Start => 0.0,
        Justify::Center => rest / 2.0,
        Justify::End => rest,
    };
    Line {
        first,
        widths: w,
        offset,
    }
}

/// Height of each line and whether the whole flow scrolls vertically.
///
/// A line is as high as its highest panel (`Fit::height`). If all lines fit into `avail_h`, the rest is shared equally by
/// the lines that hold a `fill` panel; otherwise the flow scrolls as a whole (`true`) and the lines keep their height.
pub fn heights(avail_h: f32, lines: &[Line], fits: &[Fit]) -> (Vec<f32>, bool) {
    let mut h: Vec<f32> = Vec::with_capacity(lines.len());
    let mut fills: Vec<bool> = Vec::with_capacity(lines.len());
    for l in lines {
        let fs = fits.get(l.first..l.first + l.widths.len()).unwrap_or(&[]);
        h.push(fs.iter().map(|f| f.height.max(f.min_h)).fold(0.0, f32::max));
        fills.push(fs.iter().any(|f| f.fill));
    }
    let total: f32 = h.iter().sum();
    if total > avail_h {
        return (h, true);
    }
    let n = fills.iter().filter(|f| **f).count();
    if n > 0 {
        let share = (avail_h - total) / count(n);
        for (x, f) in h.iter_mut().zip(&fills) {
            if *f {
                *x += share;
            }
        }
    }
    (h, false)
}

/// For `split`: all side by side (`true`) or all stacked (`false`); a divider between half lines makes no sense
/// (Every Layout "Switcher"). Side by side only if the minimum widths and gaps fit.
pub fn side_by_side(avail: f32, items: &[FlowItem], gap: f32) -> bool {
    if items.is_empty() {
        return true;
    }
    let sum: f32 = items.iter().map(|i| i.min.max(0.0)).sum();
    sum + gap.max(0.0) * (count(items.len()) - 1.0) <= avail
}

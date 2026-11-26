//! The axis: two rows of labels and thin lines down the rows, drawn on ONE canvas.
//!
//! Pattern: Adapter — ticks from `layout` in, calls on Blinc's `DrawContext` out. One node for the
//! whole axis instead of two per tick: ten years at day zoom are 3650 ticks, and a node per tick
//! made every redraw cost time in proportion to them. Month names come from `DateNames`
//! (`DrawCtx.dates`, CLDR: `DateNames::month`), the day numbers and years are plain numbers.

use blinc_core::{
    Brush, ClipShape, Color, CornerRadius, DrawContext, FontWeight, Point, Rect as CRect,
    TextAlign, TextBaseline, TextStyle,
};
use blinc_layout::canvas::canvas;
use blinc_layout::div::{Div, div};
use fs_components_fields::Look;
use fs_components_words::Width;

use super::parts::at;
use crate::handle::Timeline;
use crate::layout::{Rect, Scale, Tick, TickLabel, month_marks, ticks};
use crate::model::{Day, Zoom};

/// Height of one of the two label rows.
pub fn row_height(t: &Timeline) -> f32 {
    t.look.style.heights.row_dense
}

/// A month tick this wide (pixels) or wider has room for the long name (`September 2026`); a narrower one gets the medium one.
const LONG_FROM: f32 = 120.0;

fn label(t: &Timeline, tick: &Tick, room: f32) -> String {
    match tick.label {
        TickLabel::Day(n) => n.to_string(),
        TickLabel::Month(m, y) => {
            let width = if room >= LONG_FROM {
                Width::Long
            } else {
                Width::Medium
            };
            format!("{} {y}", t.dates.month(m, width))
        }
    }
}

/// What the canvas draws: lines (x, top) and labels (x, y, text).
struct Marks {
    lines: Vec<(f32, f32)>,
    labels: Vec<(f32, f32, f32, String)>,
}

fn marks(t: &Timeline, zoom: Zoom, (first, last): (Day, Day), scale: &Scale) -> Marks {
    let h = row_height(t);
    let pad = t.look.style.spacing.xs;
    let mut m = Marks {
        lines: Vec::new(),
        labels: Vec::new(),
    };
    let mut put = |tick: &Tick, y: f32, line: bool| {
        let x = scale.x_of(tick.day);
        // The label may use the room of its tick: it is cut there, not drawn over the next one.
        m.labels.push((
            x + pad,
            y,
            scale.px * f32::from(tick.days) - pad,
            label(t, tick, scale.px * f32::from(tick.days)),
        ));
        if line {
            // A line starts under the row of its own label, not through the row above it.
            m.lines.push((x, y));
        }
    };
    if zoom == Zoom::Month {
        for k in &ticks(zoom, first, last) {
            put(k, 0.0, true);
        }
    } else {
        for k in &month_marks(first, last) {
            put(k, 0.0, false);
        }
        for k in &ticks(zoom, first, last) {
            put(k, h, true);
        }
    }
    m
}

fn pen(
    ctx: &mut dyn DrawContext,
    m: &Marks,
    size: (f32, f32),
    family: &str,
    line: Color,
    text: Color,
    caption: f32,
) {
    for (x, top) in &m.lines {
        ctx.fill_rect(
            CRect::new(*x, *top, 1.0, size.1 - *top),
            CornerRadius::default(),
            Brush::Solid(line),
        );
    }
    let style = TextStyle {
        family: family.to_owned(),
        size: caption,
        color: text,
        weight: FontWeight::Regular,
        align: TextAlign::Left,
        baseline: TextBaseline::Top,
        ..TextStyle::default()
    };
    for (x, y, w, s) in &m.labels {
        // Room for fewer letters than the label has (up to four) would leave a stub that reads as a
        // word: draw nothing. A day number of two digits needs little, a month name more.
        let letters = s.chars().count().min(4) as f32;
        if *w < letters * caption * 0.6 {
            continue;
        }
        ctx.push_clip(ClipShape::Rect(CRect::new(
            *x,
            *y,
            w.max(0.0),
            caption * 2.0,
        )));
        ctx.draw_text(s, Point::new(*x, *y + 2.0), &style);
        ctx.pop_clip();
    }
}

/// The axis as one element over `total_w` x `total_h`; a rule under the two label rows.
pub fn axis(t: &Timeline, zoom: Zoom, range: (Day, Day), scale: &Scale, total_h: f32) -> Vec<Div> {
    let (s, pal) = (t.look.style, t.look.palette());
    let total_w = scale.x_of(range.1.add_days(1));
    let m = marks(t, zoom, range, scale);
    let (line, text) = (Look::color(pal.border), Look::color(pal.text_weak));
    let (family, caption) = (s.fonts[0], s.text.caption);
    let drawing = at(Rect {
        x: 0.0,
        y: 0.0,
        w: total_w,
        h: total_h,
    })
    .id("tl-axis")
    .pointer_events_none()
    .child(
        canvas(move |ctx, _| pen(ctx, &m, (total_w, total_h), family, line, text, caption))
            .redraw_while(|| false)
            .w(total_w)
            .h(total_h),
    );
    let rule = div()
        .absolute()
        .left(0.0)
        .top(2.0 * row_height(t) - 1.0)
        .w(total_w)
        .h(1.0)
        .bg(Look::color(pal.border));
    vec![drawing, rule]
}

/// The words on the axis, in the order drawn (months and days; the canvas holds no tree nodes, so a test reads them here).
pub fn texts(t: &Timeline, zoom: Zoom, range: (Day, Day), scale: &Scale) -> Vec<String> {
    marks(t, zoom, range, scale)
        .labels
        .into_iter()
        .map(|(_, _, _, s)| s)
        .collect()
}

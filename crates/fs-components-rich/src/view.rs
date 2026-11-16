//! The element of the surface: the geometry drawn, the caret on it, and the handlers of the mouse and the keyboard.
//!
//! Pattern: View function — a pure function from the state (session, geometry, theme) to elements, built again whenever the
//! counter of the surface moves; the handlers only pass what they hear to [`RichSurface`]. Every line of text is an element of its
//! own at the place the geometry gave it, so what is drawn and where the caret and a click land agree.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use blinc_layout::text;
use blinc_layout::widgets::text_input::set_focused_editable_node;
use fs_components_widgets::{Glyph, Tone, icon};

use crate::geometry::{Geometry, Leaf, Marker, Shape, Sort};
use crate::look::Look;
use crate::measure::Face;
use crate::surface::{RichSurface, key_of_code};

/// The width of the caret.
const CARET: f32 = 2.0;
/// The width of the bar of a quotation.
const BAR: f32 = 3.0;
/// The side of a bullet.
const BULLET: f32 = 6.0;

fn at(x: f32, y: f32) -> Div {
    div().absolute().left(x).top(y)
}

/// The lines of one leaf.
fn lines(leaf: &Leaf, look: &Look, root: Div) -> Div {
    let mut root = root;
    for line in &leaf.lines {
        let Some(piece) = leaf
            .text
            .get(line.start..line.end)
            .filter(|p| !p.is_empty())
        else {
            continue;
        };
        let mut t = text(piece)
            .size(leaf.face.size)
            .color(look.text)
            .line_height(leaf.face.line)
            .no_wrap();
        if leaf.face.bold {
            t = t.bold();
        }
        if leaf.face.mono {
            t = t.monospace();
        }
        root = root.child(at(leaf.x, line.y).child(t));
    }
    root
}

fn shape(s: &Shape, look: &Look, root: Div) -> Div {
    match s {
        Shape::Ground { x, y, w, h, lang } => {
            let ground = at(*x, *y).w(*w).h(*h).class("fs-re-code");
            let root = root.child(ground);
            if lang.is_empty() {
                return root;
            }
            let size = look.small * 0.85;
            let label = text(lang.as_str()).size(size).color(look.weak).no_wrap();
            let wide = Face::new(size, false, false, 1.0).width(lang);
            root.child(at(*x + *w - look.pad - wide, *y + 2.0).child(label))
        }
        Shape::Bar { x, y, h } => root.child(at(*x, *y).w(BAR).h(*h).class("fs-re-bar")),
        Shape::Rule { x, y, w } => root.child(at(*x, *y).w(*w).class("fs-re-rule")),
    }
}

fn marker(m: &Marker, look: &Look, lh: f32, root: Div) -> Div {
    match m {
        Marker::Bullet { x, y } => root.child(
            at(x - BULLET / 2.0, y - BULLET / 2.0)
                .w(BULLET)
                .h(BULLET)
                .rounded_full()
                .bg(look.text),
        ),
        Marker::Number { x, y, w, text: t } => {
            let t = text(t.as_str())
                .size(look.base)
                .color(look.weak)
                .line_height(look.line)
                .no_wrap();
            root.child(at(*x, *y).w(*w).h(lh).flex_row().justify_end().child(t))
        }
        Marker::Task {
            x, y, side, done, ..
        } => {
            let mut b = at(*x, *y).w(*side).h(*side).class("fs-re-box");
            if *done {
                b = b
                    .class("fs-re-box-on")
                    .child(icon(Glyph::Check, side - 5.0, Tone::Accent));
            }
            root.child(b)
        }
    }
}

/// Everything of `g` as children of `root`, the caret last.
fn draw(g: &Geometry, look: &Look, caret: Option<(f32, f32, f32)>) -> Div {
    let mut root = div()
        .class("fs-re")
        .relative()
        .w_full()
        .h(g.height.max(look.base * look.line));
    for s in &g.shapes {
        root = shape(s, look, root);
    }
    for l in g.leaves.iter().filter(|l| l.sort != Sort::Rule) {
        root = lines(l, look, root);
    }
    let lh = g.leaves.first().map_or(look.base * look.line, |l| l.line_h);
    for m in &g.markers {
        root = marker(m, look, lh, root);
    }
    if let Some((x, y, h)) = caret {
        // Centred on the edge between two characters, so it covers the ink of neither.
        root = root.child(at(x - CARET / 2.0, y).w(CARET).h(h).class("fs-re-caret"));
    }
    root
}

impl RichSurface {
    /// The element of the surface. Put it in a column that has a width: the text wraps at that width.
    #[must_use]
    pub fn view(self: &Arc<Self>) -> Div {
        let (build, layout, mouse, key, text_in) = (
            Arc::clone(self),
            Arc::clone(self),
            Arc::clone(self),
            Arc::clone(self),
            Arc::clone(self),
        );
        let version = self.version;
        let page = stateful::<NoState>()
            .deps([version.id()])
            .on_state(move |_| {
                let (look, g) = (Look::current(), build.geometry());
                let c = build.cursor();
                let caret = build
                    .focused()
                    .then(|| g.caret(&c.path, c.at))
                    .flatten()
                    .map(|c| (c.x, c.y, c.h));
                draw(&g, &look, caret)
            })
            .on_layout(move |b| {
                if layout.set_width(b.width) {
                    layout.version.update(|v| v.wrapping_add(1));
                }
            });
        div()
            .w_full()
            .on_mouse_down(move |e| {
                mouse.click(e.local_x, e.local_y);
                let blur = Arc::clone(&mouse);
                set_focused_editable_node(e.node_id, Some(Box::new(move || blur.set_focus(false))));
            })
            .on_text_input(move |e| {
                if let Some(c) = e.key_char.filter(|c| !c.is_control() && !e.ctrl && !e.meta) {
                    text_in.press(fs_components_editor::rich::Key::Char(c));
                }
            })
            .on_key_down(move |e| {
                if let Some(k) = key_of_code(e.key_code).filter(|_| !e.ctrl && !e.meta) {
                    key.press(k);
                }
            })
            .child(page)
    }
}

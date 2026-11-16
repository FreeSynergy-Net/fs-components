//! Placing the tree on the surface: every leaf, marker and shape at its pixel, for a given width.
//!
//! Pattern: Visitor — one function per kind of block walks the tree of the editor outside the data and puts what it finds on a
//! [`Geometry`]. Blocks stand under each other, a quote and a list indent their children; text wraps at the width it has.

use fs_components_editor::rich::{List, Node, RichEditor};

use crate::geometry::{Geometry, Leaf, Line, Marker, Shape, Sort};
use crate::look::Look;
use crate::measure::Face;

/// The width of the bar of a quotation, and the room between it and the text.
const BAR: f32 = 3.0;

struct Placer<'a> {
    look: &'a Look,
    g: Geometry,
}

/// Put the tree of `editor` on a surface `width` px wide.
#[must_use]
pub fn place(editor: &RichEditor, look: &Look, width: f32) -> Geometry {
    let mut p = Placer {
        look,
        g: Geometry::default(),
    };
    let mut y = 0.0;
    for (b, block) in editor.blocks().iter().enumerate() {
        y = p.node(&block.node, &mut vec![b], 0.0, width, y) + look.gap;
    }
    p.g.height = (y - look.gap).max(0.0);
    p.g
}

impl Placer<'_> {
    /// Place `node` at `(x, y)` in a column `w` wide; the bottom of it.
    fn node(&mut self, node: &Node, path: &mut Vec<usize>, x: f32, w: f32, y: f32) -> f32 {
        let look = self.look;
        let body = Face::new(look.base, false, false, look.line);
        match node {
            Node::Para(t) => self.text(path, Sort::Text, t, body, x, w, y, true),
            Node::Heading(n, t) => {
                let size = look.heading[usize::from((*n).clamp(1, 6)) - 1];
                let top = look.gap * 0.5;
                self.text(
                    path,
                    Sort::Heading,
                    t,
                    Face::new(size, true, false, 1.3),
                    x,
                    w,
                    y + top,
                    true,
                )
            }
            Node::Raw(t) => {
                let face = Face::new(look.small, false, true, look.line);
                self.text(path, Sort::Code, t, face, x, w, y, false)
            }
            Node::Code(lang, t) => self.code(path, lang, t, x, w, y),
            Node::Rule => {
                let h = look.gap * 2.0;
                self.g.shapes.push(Shape::Rule {
                    x,
                    y: y + h / 2.0,
                    w,
                });
                self.g.leaves.push(Leaf {
                    path: path.clone(),
                    sort: Sort::Rule,
                    text: String::new(),
                    x,
                    y,
                    w,
                    face: body,
                    lines: vec![Line {
                        start: 0,
                        end: 0,
                        y,
                    }],
                    line_h: h,
                    h,
                });
                y + h
            }
            Node::Quote(children) => {
                let indent = look.gap + BAR;
                let mut end = y;
                for (i, c) in children.iter().enumerate() {
                    path.push(i);
                    end = self.node(c, path, x + indent, w - indent, end);
                    path.pop();
                    end += if i + 1 < children.len() {
                        look.gap
                    } else {
                        0.0
                    };
                }
                self.g.shapes.push(Shape::Bar { x, y, h: end - y });
                end
            }
            Node::List(l) => {
                let mut end = y;
                for (i, item) in l.items.iter().enumerate() {
                    if i > 0 {
                        end += look.tight;
                    }
                    let first = end;
                    for (j, c) in item.children.iter().enumerate() {
                        path.extend([i, j]);
                        end = self.node(c, path, x + look.marker, w - look.marker, end);
                        path.truncate(path.len() - 2);
                        if j + 1 < item.children.len() {
                            end += look.tight;
                        }
                    }
                    self.marker(l, i, path, x, first);
                }
                end
            }
        }
    }

    /// The marker of item `i` of the list `l` (at `path`), its first line at `y`.
    fn marker(&mut self, l: &List, i: usize, path: &[usize], x: f32, y: f32) {
        let look = self.look;
        let lh = Face::new(look.base, false, false, look.line).line_height();
        let mut text_path = path.to_vec();
        text_path.extend([i, 0]);
        self.g.markers.push(if let Some(done) = l.items[i].task {
            Marker::Task {
                x: x + 2.0,
                y: y + (lh - look.box_side) / 2.0,
                side: look.box_side,
                done,
                path: text_path,
            }
        } else if l.ordered {
            let n = l.start.saturating_add(u32::try_from(i).unwrap_or(0));
            Marker::Number {
                x,
                y,
                w: look.marker - look.tight * 2.0,
                text: format!("{n}."),
            }
        } else {
            Marker::Bullet {
                x: x + look.marker / 2.0 - look.tight,
                y: y + lh / 2.0,
            }
        });
    }

    fn code(&mut self, path: &[usize], lang: &str, text: &str, x: f32, w: f32, y: f32) -> f32 {
        let look = self.look;
        let face = Face::new(look.small, false, true, 1.3);
        let top = if lang.is_empty() {
            look.pad * 0.7
        } else {
            look.pad
        };
        let inner_x = x + look.pad;
        let end = self.text(
            path,
            Sort::Code,
            text,
            face,
            inner_x,
            w - 2.0 * look.pad,
            y + top,
            false,
        );
        let h = end + look.pad * 0.7 - y;
        self.g.shapes.push(Shape::Ground {
            x,
            y,
            w,
            h,
            lang: lang.to_owned(),
        });
        y + h
    }

    /// A leaf of text; the bottom of its last line. `wrap`: long lines break at the width, else they run on.
    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        path: &[usize],
        sort: Sort,
        text: &str,
        face: Face,
        x: f32,
        w: f32,
        y: f32,
        wrap: bool,
    ) -> f32 {
        let lh = face.line_height();
        let mut lines = Vec::new();
        let mut at = 0;
        let mut top = y;
        for logical in text.split('\n') {
            let pieces = if wrap {
                face.lines(logical, w)
            } else {
                vec![(0, logical.len())]
            };
            for (s, e) in pieces {
                lines.push(Line {
                    start: at + s,
                    end: at + e,
                    y: top,
                });
                top += lh;
            }
            at += logical.len() + 1;
        }
        self.g.leaves.push(Leaf {
            path: path.to_vec(),
            sort,
            text: text.to_owned(),
            x,
            y,
            w,
            face,
            lines,
            line_h: lh,
            h: top - y,
        });
        top
    }
}

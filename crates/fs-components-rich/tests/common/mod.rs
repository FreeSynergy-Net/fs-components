//! The test harness of the writing surface: a person at a keyboard and a mouse, and a picture of what they see.
//!
//! Tests: test helpers. The keys go through the handlers of the built element (`broadcast_key_event`, `broadcast_text_input_event`,
//! the mouse through the window's event router), the way the window runner gives them; the picture is drawn from the same state.

#![allow(dead_code, clippy::cast_possible_truncation)]

pub mod shoot;

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::div::div;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use fs_components_editor::Doc;
use fs_components_render::{Canvas, draw_to_png};
use fs_components_rich::{RichSurface, STYLES};
use fs_components_styles::{StyleId, style};

/// The room around the surface in the picture: a click at `(x, y)` of the geometry is at `(PAD + x, PAD + y)` of the window.
pub const PAD: f32 = 24.0;

pub const BACKSPACE: u32 = 8;
pub const ENTER: u32 = 13;
pub const END: u32 = 35;
pub const HOME: u32 = 36;
pub const LEFT: u32 = 37;
pub const UP: u32 = 38;
pub const RIGHT: u32 = 39;
pub const DOWN: u32 = 40;

/// The CSS of the picture: the sheets of the widgets and the one of the surface.
pub fn css() -> String {
    format!("{}{}", fs_components_widgets::styles(), STYLES)
}

/// The theme of a process is one state shared by every test of it (`use_style`, the theme of Blinc): two tests that choose a
/// style at the same time would draw each other's pictures. A test therefore holds this lock from the moment it chooses its
/// style until it is done (a [`Typist`] holds it), so the tests of one style never see the style of another.
static THEME: Mutex<()> = Mutex::new(());

/// Take the theme for one test.
pub fn own_theme() -> MutexGuard<'static, ()> {
    THEME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What a style must draw, from the style itself and not from the theme the process happens to hold.
pub struct Expect {
    /// The ground of the page, when the style names it (Craft; the standard theme is the platform's).
    pub ground: Option<blinc_core::Color>,
    /// The colour of the cursor, when the style has a token for it.
    pub cursor: Option<blinc_core::Color>,
}

fn rgb(c: fs_components_styles::Rgba) -> blinc_core::Color {
    blinc_core::Color::rgb(
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
    )
}

/// The ground and cursor the style `name` asks for.
pub fn expect(name: &str) -> Expect {
    match StyleId::from_name(name) {
        Some(id) => {
            let p = style(id).palette(id == StyleId::Craft);
            Expect {
                ground: Some(rgb(p.ground)),
                cursor: Some(rgb(p.cursor)),
            }
        }
        None => Expect {
            ground: None,
            cursor: None,
        },
    }
}

pub fn close(p: image::Rgba<u8>, c: blinc_core::Color, tol: i32) -> bool {
    let ch = |v: f32| (v * 255.0).round() as i32;
    (i32::from(p[0]) - ch(c.r)).abs() <= tol
        && (i32::from(p[1]) - ch(c.g)).abs() <= tol
        && (i32::from(p[2]) - ch(c.b)).abs() <= tol
}

/// The theme guard: the page ground in the picture (a fixed place in the frame around the surface) has to be the ground the
/// requested style names; a style without a ground of its own (the standard theme) must not be dressed in the ground of any
/// style that has one (Craft). A picture drawn in the wrong theme fails here, whatever the other checks read from the theme.
pub fn assert_theme(img: &image::RgbaImage, name: &str) {
    let seen = *img.get_pixel(3, 3);
    match expect(name).ground {
        Some(ground) => assert!(
            close(seen, ground, 2),
            "{name}: the page ground is {seen:?}, the style asks for {ground:?} (picture drawn in another theme)"
        ),
        None => {
            for id in StyleId::ALL {
                if id == StyleId::Craft {
                    assert!(
                        !close(seen, rgb(style(id).palette(true).ground), 2),
                        "{name}: the page ground {seen:?} is the one of {} (picture drawn in another theme)",
                        id.name()
                    );
                }
            }
        }
    }
}

/// Make the runtime, the fonts and the theme of `style` ready (a window does this before its first frame).
pub fn warm_up(style: &str) {
    fs_components_render::use_style(StyleId::from_name(style));
    let tmp = std::env::temp_dir().join(format!("fs-rich-warm-{}.png", std::process::id()));
    draw_to_png(
        &Canvas::new(120, 60, css()).with_window_styles(),
        |_, _| div().child(blinc_layout::prelude::text("warm up").size(20.0)),
        || {},
        &tmp,
    )
    .expect("a picture can be drawn (graphics and fonts)");
    let _ = std::fs::remove_file(&tmp);
}

/// A person typing on a built surface.
pub struct Typist {
    pub surface: Arc<RichSurface>,
    pub tree: RenderTree,
    pub size: (f32, f32),
    router: EventRouter,
    _theme: MutexGuard<'static, ()>,
}

impl Typist {
    /// A surface over `doc` in a window of `w` x `h` with the theme of `style`.
    pub fn new(style: &str, doc: &Doc, w: f32, h: f32) -> Self {
        let theme = own_theme();
        warm_up(style);
        let surface = match StyleId::from_name(style) {
            Some(id) => RichSurface::with_style(doc, id),
            None => RichSurface::new(doc),
        };
        let root = div().w(w).h(h).p_px(PAD).child(surface.view());
        let mut t = Self {
            surface,
            tree: RenderTree::from_element(&root),
            size: (w, h),
            router: EventRouter::new(),
            _theme: theme,
        };
        t.settle();
        t
    }

    /// Build again what depends on a changed signal and lay it out, as the window does in its next frames.
    pub fn settle(&mut self) {
        let (w, h) = self.size;
        self.tree.compute_layout(w, h);
        for _ in 0..4 {
            let rebuilt = self.tree.process_pending_subtree_rebuilds();
            self.tree.compute_layout(w, h);
            if !rebuilt {
                break;
            }
        }
    }

    /// A click (move, press, release) at `(x, y)` of the surface.
    pub fn click(&mut self, x: f32, y: f32) {
        let (ax, ay) = (PAD + x, PAD + y);
        for events in [
            self.router.on_mouse_move(&self.tree, ax, ay),
            self.router
                .on_mouse_down(&self.tree, ax, ay, MouseButton::Left),
            self.router
                .on_mouse_up(&self.tree, ax, ay, MouseButton::Left),
        ] {
            for (n, e) in events {
                // As the window runner does: the local position is the one inside the node the handler belongs to.
                let (bx, by, bw, bh) = self
                    .router
                    .get_node_bounds(n)
                    .unwrap_or((0.0, 0.0, 0.0, 0.0));
                self.tree.dispatch_event_full(
                    n,
                    e,
                    ax,
                    ay,
                    ax - bx,
                    ay - by,
                    bx,
                    by,
                    bw,
                    bh,
                    0.0,
                    0.0,
                    1.0,
                    false,
                    false,
                    false,
                    false,
                );
            }
        }
        self.settle();
    }

    /// A key by its code.
    pub fn key(&mut self, code: u32) {
        self.tree
            .broadcast_key_event(KEY_DOWN, code, false, false, false, false);
        self.settle();
    }

    /// Several presses of the same key.
    pub fn keys(&mut self, code: u32, times: usize) {
        for _ in 0..times {
            self.key(code);
        }
    }

    /// Type `text`: Enter for a line break, the others as characters, one key at a time.
    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\n' {
                self.tree
                    .broadcast_key_event(KEY_DOWN, ENTER, false, false, false, false);
            } else {
                self.tree
                    .broadcast_text_input_event(c, false, false, false, false);
            }
            self.settle();
        }
    }

    /// The text of the page as Markdown.
    pub fn markdown(&self) -> String {
        self.surface.live_document().to_markdown()
    }

    /// Click on the box of the task item whose text is the leaf at `path`.
    pub fn click_box(&mut self, path: &[usize]) {
        let g = self.surface.geometry();
        let (x, y, side) = g
            .markers
            .iter()
            .find_map(|m| match m {
                fs_components_rich::Marker::Task {
                    x,
                    y,
                    side,
                    path: p,
                    ..
                } if p == path => Some((*x, *y, *side)),
                _ => None,
            })
            .expect("a box for that task");
        self.click(x + side / 2.0, y + side / 2.0);
    }

    /// Draw what the person sees now to `file`: the surface with the caret, on the ground of the theme.
    pub fn picture(&self, file: &Path) {
        let (w, h) = self.size;
        let surface = Arc::clone(&self.surface);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let canvas = Canvas::new(w as u32, h as u32, css()).with_window_styles();
        draw_to_png(
            &canvas,
            move |w, h| {
                let bg = blinc_theme::ThemeState::get().color(blinc_theme::ColorToken::Background);
                div().w(w).h(h).bg(bg).p_px(PAD).child(surface.view())
            },
            || {},
            file,
        )
        .expect("the picture is drawn");
    }
}

//! The state of the writing surface: the session with its cursor, the focus, the width, and what keys and clicks do to them.
//!
//! Pattern: Observer — the surface holds a counter signal that is counted up after every change; the element
//! ([`crate::view`]) depends on it and is built again from the state. The keys and the clicks come through here, from the handlers
//! of the element and from the tests alike, so the test of a key is the test of the key the window gives.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::Signal;
use blinc_core::reactive::signal;
use blinc_layout::widgets::text_input::{decrement_focus_count, increment_focus_count};
use fs_components_editor::rich::{Cursor, Key, Session};
use fs_components_editor::{Doc, Op};
use fs_components_styles::StyleId;

use crate::geometry::Geometry;
use crate::look::Look;
use crate::place::place;
use crate::query::Landing;

/// The width the surface assumes until the layout has told its real width.
const FIRST_WIDTH: f32 = 720.0;

struct Inner {
    session: Session,
    focused: bool,
    width: f32,
    /// The x the person is after while going up and down (none after any other key).
    want_x: Option<f32>,
}

impl Inner {
    fn geometry(&self, style: StyleId) -> Geometry {
        let width = if self.width > 0.0 {
            self.width
        } else {
            FIRST_WIDTH
        };
        place(self.session.editor(), &Look::of(style), width)
    }

    /// A key that depends on how the lines are set (wrapped lines count): Up, Down, Home and End by visual line.
    fn visual(&mut self, key: Key, style: StyleId) -> bool {
        let c = self.session.cursor().clone();
        let g = self.geometry(style);
        let (landing, keep) = match key {
            Key::Up | Key::Down => (g.step(&c.path, c.at, key == Key::Down, self.want_x), true),
            Key::Home | Key::End => (
                g.line_edge(&c.path, c.at, key == Key::End)
                    .map(|at| Landing {
                        path: c.path.clone(),
                        at,
                        x: 0.0,
                    }),
                false,
            ),
            _ => return false,
        };
        if let Some(l) = landing {
            self.session.place(&l.path, l.at);
            self.want_x = keep.then_some(l.x);
        }
        true
    }
}

/// A writing surface: what a person writes on, and the state behind it.
///
/// The rule for a host that embeds it: **store [`RichSurface::save_form`] (or the operations of [`RichSurface::take_save_ops`])**.
/// [`RichSurface::document`] and [`RichSurface::take_ops`] are the live form, what was typed as it was typed; stored, a paragraph
/// `## x` would be a heading when loaded again.
pub struct RichSurface {
    inner: Mutex<Inner>,
    style: StyleId,
    pub(crate) version: Signal<u64>,
}

/// The key a platform key code is on the surface (the DOM-style codes Blinc delivers); none for the others.
#[must_use]
pub fn key_of_code(code: u32) -> Option<Key> {
    Some(match code {
        8 => Key::Backspace,
        13 => Key::Enter,
        35 => Key::End,
        36 => Key::Home,
        37 => Key::Left,
        38 => Key::Up,
        39 => Key::Right,
        40 => Key::Down,
        _ => return None,
    })
}

impl Drop for RichSurface {
    /// A surface that goes with the keyboard still in its hands gives it back: the window counts the focused writing surfaces.
    fn drop(&mut self) {
        let inner = self.inner.get_mut().unwrap_or_else(PoisonError::into_inner);
        if inner.focused {
            inner.focused = false;
            decrement_focus_count();
        }
    }
}

impl RichSurface {
    /// A surface over `doc` in the style `Admin`.
    #[must_use]
    pub fn new(doc: &Doc) -> Arc<Self> {
        Self::with_style(doc, StyleId::Admin)
    }

    /// A surface over `doc` for the style the process draws in (the check mark of a task comes from its icon set).
    #[must_use]
    pub fn with_style(doc: &Doc, style: StyleId) -> Arc<Self> {
        Arc::new(Self {
            style,
            inner: Mutex::new(Inner {
                session: Session::new(doc),
                focused: false,
                width: 0.0,
                want_x: None,
            }),
            version: signal(0u64),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn changed(&self) {
        self.version.update(|v| v.wrapping_add(1));
    }

    /// The document to store: what a host saves. Loaded again it gives the same blocks (a paragraph `## x` is stored as `\## x`).
    #[must_use]
    pub fn save_form(&self) -> Doc {
        self.lock().session.save_form()
    }

    /// The operations that turn the document as it was made (or as it was at the last call) into [`RichSurface::save_form`]: what
    /// a host that stores operations takes.
    #[must_use]
    pub fn take_save_ops(&self) -> Vec<Op> {
        self.lock().session.take_save_ops()
    }

    /// The document as it is now, in the **live form**: what was typed is what the source holds.
    ///
    /// # Warning
    ///
    /// Not for storing: a paragraph that reads `## x` is a heading when loaded again. Store [`RichSurface::save_form`].
    #[must_use]
    pub fn document(&self) -> Doc {
        self.lock().session.document()
    }

    /// The operations of the **live form** since the surface was made (or since the last call).
    ///
    /// # Warning
    ///
    /// Not for storing, see [`RichSurface::document`]; store the operations of [`RichSurface::take_save_ops`].
    #[must_use]
    pub fn take_ops(&self) -> Vec<Op> {
        self.lock().session.take_ops()
    }

    /// The cursor.
    #[must_use]
    pub fn cursor(&self) -> Cursor {
        self.lock().session.cursor().clone()
    }

    /// The style the surface is drawn in.
    #[must_use]
    pub fn style(&self) -> StyleId {
        self.style
    }

    /// Does the surface have the keyboard?
    #[must_use]
    pub fn focused(&self) -> bool {
        self.lock().focused
    }

    /// Read the session.
    pub fn with_session<R>(&self, f: impl FnOnce(&Session) -> R) -> R {
        f(&self.lock().session)
    }

    /// The width the layout gave the surface (0 until it has).
    pub(crate) fn set_width(&self, width: f32) -> bool {
        let mut inner = self.lock();
        let moved = (inner.width - width).abs() > 0.5;
        inner.width = width;
        moved
    }

    /// Everything placed, for the width now and the theme now.
    #[must_use]
    pub fn geometry(&self) -> Geometry {
        let inner = self.lock();
        let width = if inner.width > 0.0 {
            inner.width
        } else {
            FIRST_WIDTH
        };
        place(inner.session.editor(), &Look::of(self.style), width)
    }

    /// Give the keyboard to the surface (or take it away); counted for the window, which delivers keys to text only while a text
    /// widget has the focus.
    pub fn set_focus(&self, focused: bool) {
        {
            let mut inner = self.lock();
            if inner.focused == focused {
                return;
            }
            inner.focused = focused;
        }
        if focused {
            increment_focus_count();
        } else {
            decrement_focus_count();
        }
        self.changed();
    }

    /// One key (the surface has to have the focus: a key that comes without it is not for it).
    pub fn press(&self, key: Key) {
        {
            let mut inner = self.lock();
            if !inner.focused {
                return;
            }
            if !inner.visual(key, self.style) {
                inner.want_x = None;
                inner.session.key(key);
            }
        }
        self.changed();
    }

    /// A press of the mouse at `(x, y)` of the surface: the box of a task is ticked or unticked, any other point puts the cursor
    /// in the text nearest to it; either way the surface has the keyboard.
    pub fn click(&self, x: f32, y: f32) {
        self.set_focus(true);
        let g = self.geometry();
        {
            let mut inner = self.lock();
            if let Some(path) = g.task_at(x, y) {
                inner.session.click_task(path);
            } else if let Some((path, at)) = g.hit(x, y) {
                inner.session.place(&path, at);
            }
        }
        self.changed();
    }
}

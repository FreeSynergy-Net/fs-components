//! The real board element, laid out and driven like a window drives it, without opening one.
//!
//! Tests: a helper over Blinc's tree and router; the pointer carries its local position as a window does (the pressed element
//! gets the move and the release wherever the pointer is).
#![allow(dead_code, clippy::pedantic)]

use std::cell::Cell;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use blinc_core::{BlincContextState, HookState};
use blinc_layout::css_parser::Stylesheet;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use blinc_layout::tree::LayoutNodeId;
use fs_components_workbenches::board::{BoardBench, Card, Column, MemoryBoard};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// How long a test waits for the other tests that hold the lock (they draw for milliseconds; a minute means a hang).
pub const SERIAL_WAIT: Duration = Duration::from_secs(120);

thread_local! {
    /// This thread (= this test) holds the lock already.
    static HOLDS: Cell<bool> = const { Cell::new(false) };
}

/// The lock of the tests that share Blinc's global state; released (and the thread's mark cleared) when dropped.
pub struct Serial {
    _guard: MutexGuard<'static, ()>,
}

impl Drop for Serial {
    fn drop(&mut self) {
        HOLDS.with(|h| h.set(false));
    }
}

/// Takes the lock within `wait`. Panics with a plain text, never hangs: (1) the same test holds it already (a second `Rig` or
/// `serial()` while the first is alive would wait for itself for ever: `drop` the first), (2) another test holds it for longer
/// than `wait`.
pub fn serial_within(wait: Duration) -> Serial {
    assert!(
        !HOLDS.with(Cell::get),
        "this test holds the serial lock already (a second Rig or serial() while the first is alive would wait for itself): drop(rig) first"
    );
    let start = Instant::now();
    loop {
        match ONE_AT_A_TIME.try_lock() {
            Ok(guard) => return mark(guard),
            Err(TryLockError::Poisoned(p)) => return mark(p.into_inner()),
            Err(TryLockError::WouldBlock) => {
                assert!(
                    start.elapsed() < wait,
                    "the serial lock of the board tests was not free for {wait:?}: another test holds it (a hang?)"
                );
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}

fn mark(guard: MutexGuard<'static, ()>) -> Serial {
    HOLDS.with(|h| h.set(true));
    Serial { _guard: guard }
}

pub fn serial() -> Serial {
    serial_within(SERIAL_WAIT)
}

pub fn column(key: &str, title: &str, cards: &[(&str, &str)]) -> Column {
    let mut c = Column::new(key, title);
    c.cards = cards.iter().map(|(k, t)| Card::new(k, t)).collect();
    c
}

pub fn sample() -> MemoryBoard {
    let mut doing = column("doing", "Doing", &[("d", "Call the landlord")]);
    doing.limit = Some(1);
    MemoryBoard::new(vec![
        column(
            "todo",
            "To do",
            &[
                ("a", "Write the letter"),
                ("b", "Buy stamps"),
                ("c", "Water the plants"),
            ],
        ),
        doing,
        column("done", "Done", &[]),
    ])
}

pub struct Rig {
    pub bench: BoardBench,
    pub tree: RenderTree,
    scope: BuildScope,
    router: EventRouter,
    size: (f32, f32),
    _serial: Serial,
}

fn tree_of(bench: &BoardBench, scope: &mut BuildScope, (w, h): (f32, f32)) -> RenderTree {
    let bundle = fs_components_render::bundle(&fs_components_workbenches::board::styles());
    let scheme = if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        blinc_theme::ColorScheme::Dark
    } else {
        blinc_theme::ColorScheme::Light
    };
    blinc_theme::ThemeState::init(bundle.clone(), scheme);
    if !BlincContextState::is_initialized() {
        BlincContextState::init_with_callback(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            Arc::new(|ids: &[blinc_core::reactive::SignalId]| {
                blinc_layout::check_stateful_deps(ids);
            }),
        );
    }
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    blinc_layout::init_text_measurer();
    let pass = scope.begin();
    blinc_layout::key::reset_call_counters();
    let mut tree = RenderTree::from_element(&bench.build(w, h));
    let vars = blinc_theme::ThemeState::get().to_css_variable_map();
    let css = bundle.css_sources.join("\n");
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).expect("stylesheet"));
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    scope.finish(pass);
    tree
}

impl Rig {
    pub fn new(bench: BoardBench, w: f32, h: f32) -> Self {
        let serial = serial();
        let mut scope = BuildScope::new();
        let tree = tree_of(&bench, &mut scope, (w, h));
        Self {
            bench,
            tree,
            scope,
            router: EventRouter::new(),
            size: (w, h),
            _serial: serial,
        }
    }

    /// The window builds everything again.
    pub fn rebuild(&mut self) {
        self.tree = tree_of(&self.bench, &mut self.scope, self.size);
    }

    /// A frame: live areas that changed draw again.
    pub fn frame(&mut self) {
        let updates = self.tree.take_pending_property_updates();
        self.tree.apply_partial_property_updates(updates);
        self.tree.process_pending_subtree_rebuilds_routed(None);
        self.tree.compute_layout(self.size.0, self.size.1);
    }

    pub fn has(&self, id: &str) -> bool {
        self.tree.query_by_id(id).is_some()
    }

    pub fn bounds(&self, id: &str) -> (f32, f32, f32, f32) {
        let node = self
            .tree
            .query_by_id(id)
            .unwrap_or_else(|| panic!("no element {id}"));
        let b = self.tree.get_absolute_bounds(node).expect("bounds");
        (b.x, b.y, b.width, b.height)
    }

    pub fn centre(&self, id: &str) -> (f32, f32) {
        let (x, y, w, h) = self.bounds(id);
        (x + w / 2.0, y + h / 2.0)
    }

    fn send(&mut self, events: Vec<(LayoutNodeId, u32)>, x: f32, y: f32) {
        for (node, event) in events {
            match self.tree.get_absolute_bounds(node) {
                Some(b) => self.tree.dispatch_event_with_local(
                    node,
                    event,
                    x,
                    y,
                    x - b.x,
                    y - b.y,
                    b.x,
                    b.y,
                    b.width,
                    b.height,
                ),
                None => self.tree.dispatch_event(node, event, x, y),
            }
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        let ev = self.router.on_mouse_move(&self.tree, x, y);
        self.send(ev, x, y);
    }

    pub fn down(&mut self, x: f32, y: f32) {
        self.move_to(x, y);
        let ev = self
            .router
            .on_mouse_down(&self.tree, x, y, MouseButton::Left);
        self.send(ev, x, y);
    }

    pub fn up(&mut self, x: f32, y: f32) {
        let ev = self.router.on_mouse_up(&self.tree, x, y, MouseButton::Left);
        self.send(ev, x, y);
    }

    pub fn click(&mut self, id: &str) {
        let (x, y) = self.centre(id);
        self.down(x, y);
        self.up(x, y);
        self.frame();
    }

    /// Press on `from`, move in steps to the point, let go.
    pub fn drag_to(&mut self, from: &str, to: (f32, f32)) {
        let (x, y) = self.centre(from);
        self.down(x, y);
        let steps = 6;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.move_to(x + (to.0 - x) * t, y + (to.1 - y) * t);
        }
        self.up(to.0, to.1);
        self.frame();
    }

    pub fn key(&mut self, code: u32, shift: bool) {
        if self
            .tree
            .handle_navigation_key(&mut self.router, code, shift)
        {
            return;
        }
        if let Some((node, event)) = self.router.on_key_down(code) {
            self.tree.dispatch_event(node, event, 0.0, 0.0);
        }
        self.tree
            .broadcast_key_event(KEY_DOWN, code, shift, false, false, false);
        self.frame();
    }

    /// Tab until the board has the keyboard.
    pub fn focus_board(&mut self, id: &str) -> bool {
        let want = self.tree.query_by_id(id);
        for _ in 0..40 {
            self.key(9, false);
            if want.is_some() && self.router.focused() == want {
                return true;
            }
        }
        false
    }
}

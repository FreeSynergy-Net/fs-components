//! Test support: a grid over a few items, and a log of what the host heard.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic, dead_code)]

use std::sync::{Arc, Mutex};

use fs_components_calendar_grid::*;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;

pub const W: f32 = 900.0;
pub const H: f32 = 640.0;

#[derive(Default)]
pub struct Log(pub Mutex<Vec<String>>);

impl GridEvents for Log {
    fn select_slot(&self, s: Slot) {
        self.0
            .lock()
            .unwrap()
            .push(format!("slot {:?} {:?}", s.day.ymd(), s.minute));
    }
    fn select_item(&self, key: &str) {
        self.0.lock().unwrap().push(format!("item {key}"));
    }
    fn navigated(&self, nav: Nav, a: Day) {
        self.0
            .lock()
            .unwrap()
            .push(format!("nav {nav:?} {:?}", a.ymd()));
    }
}

/// The day of a date that exists.
pub fn d(y: i32, m: u8, dd: u8) -> Day {
    Day::new(y, m, dd).expect("a real date")
}

/// A wall-clock moment `minute` minutes into `day` (below 1440).
pub fn at(day: Day, minute: u16) -> Moment {
    Moment { day, minute }
}

/// A day as the tuple the tests compare with.
pub trait Ymd {
    fn ymd(self) -> (i32, u8, u8);
}

impl Ymd for Day {
    fn ymd(self) -> (i32, u8, u8) {
        (self.year(), self.month(), self.day())
    }
}

pub fn grid(style: StyleId) -> (CalendarGrid, Arc<Log>) {
    let items = vec![
        GridItem::all_day("trip", "Trip", d(2026, 11, 9), d(2026, 11, 12)),
        GridItem::timed(
            "sync",
            "Sync",
            at(d(2026, 11, 10), 600),
            at(d(2026, 11, 10), 660),
        ),
        GridItem::timed(
            "call",
            "Call",
            at(d(2026, 11, 10), 630),
            at(d(2026, 11, 10), 690),
        ),
    ];
    let mut st = GridState::new(Mode::Month, d(2026, 11, 10));
    st.today = Some(d(2026, 11, 10));
    st.now = Some(at(d(2026, 11, 10), 615));
    let log = Arc::new(Log::default());
    let g = CalendarGrid::new(
        st,
        Arc::new(VecSource(items)),
        Arc::new(KeysAsText),
        Look::new(style, false),
    )
    .events(log.clone());
    (g, log)
}

pub fn last(log: &Log) -> String {
    log.0.lock().unwrap().last().cloned().unwrap_or_default()
}

/// The texts drawn inside the element with this id (itself and everything below it), in tree order.
pub fn texts_under(tree: &blinc_layout::prelude::RenderTree, id: &str) -> Vec<String> {
    fn walk(
        tree: &blinc_layout::prelude::RenderTree,
        node: blinc_layout::tree::LayoutNodeId,
        out: &mut Vec<String>,
    ) {
        if let Some(n) = tree.get_render_node(node)
            && let blinc_layout::renderer::ElementType::Text(t) = &n.element_type
        {
            out.push(t.content.clone());
        }
        for child in tree.layout().children(node) {
            walk(tree, child, out);
        }
    }
    let mut out = Vec::new();
    if let Some(node) = tree.query_by_id(id) {
        walk(tree, node, &mut out);
    }
    out
}

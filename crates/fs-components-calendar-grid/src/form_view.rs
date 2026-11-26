//! The month grid as a [`FormView`] of the form `bars-in-time`: the rows come through `read_bars` (the one adapter of the calendar
//! components) as items of the grid; a row that cannot be read is named and counted as a partial notice above the grid.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, a [`CalendarGrid`] over a [`VecSource`] out). The view of the
//! block is the one the place asks for (`ctx.place.view`: month, week, day or agenda; nothing asked = the month, the contract default);
//! the variant is the one the place asks for (`compact`: the small month of a side bar; nothing asked = the full grid).
//! A slot is only a cursor here: the form has no `create`, so a slot choice goes nowhere; an item choice only marks the item.

use std::sync::{Arc, Mutex, PoisonError};

pub use crate::calendar_core::partial_notice;
use crate::calendar_core::{BarEntry, BarsRead, Day, read_bars, today};
use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::keys::Key;
use fs_components_fields::{Look, Say};

use crate::place::{mode_of, variant_of, week_numbers_of, week_start_of};
use crate::{CalendarGrid, GridItem, GridState, VecSource};

/// The widget this view is (`widget` of the block `calendar-grid`).
pub const WIDGET: &str = "CalendarGrid";

/// The height the partial notice takes from the grid (one line and its padding).
const NOTICE_HEIGHT: f32 = 48.0;

/// The view of the block `calendar-grid`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        let day = today(&ctx);
        self.mount_on(&input, ctx, day)
    }
}

impl View {
    /// [`FormView::mount`] with the day the month opens on (and marks as today); `mount` takes the clock (UTC: the context knows no
    /// clock and no zone yet, `DrawCtx` C2-V1).
    #[must_use]
    pub fn mount_on(&self, input: &FormInput, ctx: DrawCtx, today: Day) -> Box<dyn Mounted> {
        let read = read_bars(input);
        let mut state = GridState::new(mode_of(&ctx.place), today);
        state.today = Some(today);
        state.week_numbers = week_numbers_of(&ctx.place);
        state.variant = variant_of(&ctx.place);
        state.week_start = week_start_of(&ctx.place, &*ctx.dates);
        let grid = make(&read, &ctx, state);
        Box::new(MountedCalendar(Mutex::new(Held {
            ctx,
            grid,
            invalid: read.invalid,
            total: input.rows.len(),
        })))
    }
}

/// The item of the grid for an entry: an all-day entry ends the day before its exclusive end.
fn item_of(entry: &BarEntry) -> GridItem {
    let made = if entry.all_day {
        GridItem::all_day(
            &entry.key,
            &entry.label,
            entry.start.day,
            entry.end.day.add_days(-1),
        )
    } else {
        GridItem::timed(&entry.key, &entry.label, entry.start, entry.end)
    };
    match entry.color {
        Some(slot) => made.slot(slot),
        None => made,
    }
}

/// The grid over `read` in `state` (a refresh passes the state of the reader, so mode, month and choice stay).
fn make(read: &BarsRead, ctx: &DrawCtx, state: GridState) -> CalendarGrid {
    let items: Vec<GridItem> = read.entries.iter().map(item_of).collect();
    let say = Arc::clone(&ctx.say);
    let say: Arc<dyn Say> = Arc::new(move |key: &str| say(key));
    CalendarGrid::new(
        state,
        Arc::new(VecSource(items)),
        say,
        Look::new(ctx.look.0, ctx.look.1),
    )
    .localised(&ctx.locale, Arc::clone(&ctx.dates))
}

/// The context, the grid and what could not be read.
struct Held {
    ctx: DrawCtx,
    grid: CalendarGrid,
    invalid: Vec<String>,
    total: usize,
}

struct MountedCalendar(Mutex<Held>);

impl MountedCalendar {
    fn held(&self) -> std::sync::MutexGuard<'_, Held> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Mounted for MountedCalendar {
    fn build(&self, width: f32, height: f32) -> Div {
        let held = self.held();
        let mut root = div().w(width).h(height).flex_col();
        let mut grid_height = height;
        if !held.invalid.is_empty() {
            let notice =
                partial_notice(&held.ctx, &held.invalid, held.total).id("calendar-partial");
            root = root.child(notice.build());
            grid_height = (height - NOTICE_HEIGHT).max(0.0);
        }
        root.child(held.grid.build(width, grid_height))
    }

    fn refresh(&self, input: FormInput) {
        let read = read_bars(&input);
        let mut held = self.held();
        let grid = make(&read, &held.ctx, held.grid.snapshot());
        held.grid = grid;
        held.invalid = read.invalid;
        held.total = input.rows.len();
    }

    fn key(&self, key: Key) -> bool {
        let grid = self.held().grid.clone();
        grid.key(key)
    }
}

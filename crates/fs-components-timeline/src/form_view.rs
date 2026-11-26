//! The timeline as a [`FormView`] of the form `bars-in-time`: the rows come through `read_bars` (the one reader of the calendar
//! components, in `fs-components-calendar-grid`) and become [`BarRecord`]s; a row that cannot be read is named and counted as a partial
//! notice above the axis (`partial_notice`, shared with `calendar-grid`).
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, a [`Timeline`] over a swappable source out). The timeline only
//! shows the plan: the form's `move` is not used. The place gives no zoom yet (`DrawCtx.place`, C2-V1), so the standard zoom (weeks) holds.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_calendar_grid::calendar_core::{Day, partial_notice, read_bars, today};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::keys::Key;
use fs_components_fields::{Look, Say};

use crate::Timeline;
use crate::bars::{BarRecord, BarsSource};
use crate::model::{Dependency, Item};
use crate::source::TimelineSource;
use crate::state::TimelineState;

/// The widget this view is (`widget` of the block `timeline`).
pub const WIDGET: &str = "Timeline";

/// The height the partial notice takes from the timeline (one line and its padding).
const NOTICE_HEIGHT: f32 = 48.0;

/// The view of the block `timeline`.
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
    /// [`FormView::mount`] with the day of the today line; `mount` takes `ctx.clock` (UTC; the context knows no zone yet).
    #[must_use]
    pub fn mount_on(&self, input: &FormInput, ctx: DrawCtx, today: Day) -> Box<dyn Mounted> {
        let read = read_bars(input);
        let source = Arc::new(Latest::new(
            read.entries.iter().map(BarRecord::from_entry).collect(),
        ));
        let say = Arc::clone(&ctx.say);
        let say: Arc<dyn Say> = Arc::new(move |key: &str| say(key));
        let timeline = Timeline::new(
            TimelineState::new(Some(today)),
            source.clone(),
            say,
            Look::new(ctx.look.0, ctx.look.1),
        )
        .localised(&ctx.locale, Arc::clone(&ctx.dates));
        Box::new(MountedTimeline {
            held: Mutex::new(Held {
                ctx,
                invalid: read.invalid,
                total: input.rows.len(),
            }),
            source,
            timeline,
        })
    }
}

/// The records of the last reading, behind the one source the timeline keeps (a refresh swaps them).
struct Latest(Mutex<BarsSource>);

impl Latest {
    fn new(records: Vec<BarRecord>) -> Self {
        Self(Mutex::new(BarsSource::new(records)))
    }

    fn set(&self, records: Vec<BarRecord>) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = BarsSource::new(records);
    }
}

impl TimelineSource for Latest {
    fn items(&self) -> Vec<Item> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .items()
    }

    fn dependencies(&self) -> Vec<Dependency> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .dependencies()
    }
}

/// The context and what could not be read.
struct Held {
    ctx: DrawCtx,
    invalid: Vec<String>,
    total: usize,
}

struct MountedTimeline {
    held: Mutex<Held>,
    source: Arc<Latest>,
    timeline: Timeline,
}

impl Mounted for MountedTimeline {
    fn build(&self, width: f32, height: f32) -> Div {
        let held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let mut root = div().w(width).h(height).flex_col();
        let mut body = height;
        if !held.invalid.is_empty() {
            let notice =
                partial_notice(&held.ctx, &held.invalid, held.total).id("timeline-partial");
            root = root.child(notice.build());
            body = (height - NOTICE_HEIGHT).max(0.0);
        }
        root.child(self.timeline.build(width, body))
    }

    fn refresh(&self, input: FormInput) {
        let read = read_bars(&input);
        self.source
            .set(read.entries.iter().map(BarRecord::from_entry).collect());
        {
            let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
            held.invalid = read.invalid;
            held.total = input.rows.len();
        }
        // A choice whose record is gone is dropped; zoom and the other choice stay.
        let items = self.source.items();
        let mut state = self.timeline.lock();
        if state
            .selected
            .as_deref()
            .is_some_and(|k| !items.iter().any(|i| i.key == k))
        {
            state.selected = None;
        }
        drop(state);
        self.timeline.refresh();
    }

    fn key(&self, key: Key) -> bool {
        self.timeline.key(key)
    }
}

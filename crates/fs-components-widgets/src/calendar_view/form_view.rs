//! The agenda as a [`FormView`] of the form `bars-in-time`: the rows come through `read_bars` (the one adapter of the calendar
//! components) as items of [`CalendarView`]; a row that cannot be read is named and counted as a partial notice above the list.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, a [`CalendarView`] of the range agenda out). The agenda is the
//! default of the block `calendar-view` in the contract; the place gives no view or variant yet (`DrawCtx.place`, C2-V1), so none
//! is read. The form has no `open`: a choice (click or key) only marks the item.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::keys::Key;

use super::bars_form::{BarEntry, BarsRead, read_bars};
use super::day::{Day, Moment};
use super::form_support::{partial_notice, today_utc};
use super::layout::{span_of, step_from};
use super::model::{CalendarItem, CalendarNames, Picked, ViewRange};
use super::position::CalendarPosition;
use super::view::{AGENDA_DAYS, CalendarView};

/// The widget this view is (`widget` of the block `calendar-view`).
pub const WIDGET: &str = "CalendarView";

/// The height the partial notice takes from the list (one line and its padding).
const NOTICE_HEIGHT: f32 = 48.0;

/// The view of the block `calendar-view`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        self.mount_on(&input, ctx, today_utc())
    }
}

impl View {
    /// [`FormView::mount`] with the day the agenda starts on (and marks as today); `mount` takes the clock (UTC, see
    /// [`today_utc`]).
    #[must_use]
    pub fn mount_on(&self, input: &FormInput, ctx: DrawCtx, today: Day) -> Box<dyn Mounted> {
        let read = read_bars(input);
        Box::new(MountedAgenda {
            held: Mutex::new(Held {
                names: Arc::new(names(&ctx)),
                items: Arc::new(read.entries.iter().map(BarEntry::to_item).collect()),
                invalid: read.invalid,
                total: input.rows.len(),
                ctx,
                anchor: today,
                today,
            }),
            chosen: Arc::new(Mutex::new(None)),
            position: CalendarPosition::new(),
        })
    }
}

/// The words of the list, each one a key of the host (`calendar.weekday.N` from Monday, `calendar.month.N` from 1).
fn names(ctx: &DrawCtx) -> CalendarNames {
    let say = |key: String| (ctx.say)(&key);
    CalendarNames::new(
        std::array::from_fn(|i| say(format!("calendar.weekday.{i}"))),
        std::array::from_fn(|i| say(format!("calendar.month.{}", i + 1))),
        say("calendar.all-day".to_owned()),
        say("calendar.empty".to_owned()),
    )
}

/// What `refresh` replaces.
struct Held {
    ctx: DrawCtx,
    names: Arc<CalendarNames>,
    items: Arc<Vec<CalendarItem>>,
    invalid: Vec<String>,
    total: usize,
    anchor: Day,
    today: Day,
}

impl Held {
    /// The keys of the items in the shown span, in the order of the list.
    fn shown(&self) -> Vec<String> {
        let (first, last) = span_of(ViewRange::Agenda, self.anchor, 0, AGENDA_DAYS);
        let (from, to) = (Moment::start_of(first), Moment::start_of(last.add_days(1)));
        let mut items: Vec<&CalendarItem> = self
            .items
            .iter()
            .filter(|i| i.end > from && i.start < to)
            .collect();
        items.sort_by(|a, b| (a.start, &a.key).cmp(&(b.start, &b.key)));
        items.into_iter().map(|i| i.key.clone()).collect()
    }
}

struct MountedAgenda {
    held: Mutex<Held>,
    chosen: Arc<Mutex<Option<String>>>,
    position: CalendarPosition,
}

impl MountedAgenda {
    fn held(&self) -> std::sync::MutexGuard<'_, Held> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn chosen(&self) -> std::sync::MutexGuard<'_, Option<String>> {
        self.chosen.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The choice `step` places on (a negative step goes back); from nothing the first (or, backwards, the last).
    fn walk(&self, step: isize) -> bool {
        let keys = self.held().shown();
        if keys.is_empty() {
            return false;
        }
        let mut chosen = self.chosen();
        let at = chosen
            .as_ref()
            .and_then(|c| keys.iter().position(|k| k == c));
        let next = match at {
            Some(i) => i.saturating_add_signed(step).min(keys.len() - 1),
            None if step < 0 => keys.len() - 1,
            None => 0,
        };
        *chosen = Some(keys[next].clone());
        true
    }
}

impl Mounted for MountedAgenda {
    fn build(&self, width: f32, height: f32) -> Div {
        let held = self.held();
        let mut root = div().w(width).h(height).flex_col();
        let mut list_height = height;
        if !held.invalid.is_empty() {
            let notice =
                partial_notice(&held.ctx, &held.invalid, held.total).id("calendar-partial");
            root = root.child(notice.build());
            list_height = (height - NOTICE_HEIGHT).max(0.0);
        }
        let chosen = Arc::clone(&self.chosen);
        let view = CalendarView::new(
            ViewRange::Agenda,
            held.anchor,
            Arc::clone(&held.names),
            Arc::clone(&held.items),
        )
        .today(held.today)
        .named("agenda")
        .position(&self.position)
        .selected_item(self.chosen().clone())
        .on_pick(Arc::new(move |picked| {
            if let Picked::Item(key) = picked {
                *chosen.lock().unwrap_or_else(PoisonError::into_inner) = Some(key);
            }
        }));
        root.child(view.build(width, list_height))
    }

    fn refresh(&self, input: FormInput) {
        let read: BarsRead = read_bars(&input);
        let mut held = self.held();
        held.items = Arc::new(read.entries.iter().map(BarEntry::to_item).collect());
        held.invalid = read.invalid;
        held.total = input.rows.len();
        // A choice the module no longer has is gone; one it still has stays.
        let mut chosen = self.chosen();
        if chosen
            .as_ref()
            .is_some_and(|c| !held.items.iter().any(|i| &i.key == c))
        {
            *chosen = None;
        }
    }

    fn key(&self, key: Key) -> bool {
        match key {
            Key::Down => self.walk(1),
            Key::Up => self.walk(-1),
            Key::Home => self.walk(isize::MIN / 2),
            Key::End => self.walk(isize::MAX / 2),
            Key::PageDown | Key::PageUp => {
                let mut held = self.held();
                held.anchor = step_from(
                    ViewRange::Agenda,
                    held.anchor,
                    key == Key::PageDown,
                    AGENDA_DAYS,
                );
                *self.chosen() = None;
                true
            }
            Key::Enter | Key::Space | Key::Activate => self.chosen().is_some(),
            _ => false,
        }
    }
}

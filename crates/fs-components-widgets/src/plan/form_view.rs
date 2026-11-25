//! The block `task-row` as a [`FormView`] of the form `checklist@^1.1.0`: the rows become a list of task lines under the
//! heading of their group; a tick (box click or Space) goes out as the command `toggle` (writes `done`), an open (click on the
//! line or Enter) as the command `open`.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, [`TaskLine`]s out, the two requests out as writes through
//! the port). The state a reader made (the keyboard cursor) stays over `refresh` where its row still exists. A command the
//! host does not allow is not offered: a box that may not be ticked is drawn locked, and `open` without the right does nothing.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::scroll::scroll;
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::{WriteArg, WriteValue};
use fs_components_core::keys::Key;

use super::checklist_form::{OPEN, Rows, TOGGLE, read_rows};
use super::events::{Events, Say, redraw};
use super::line::Cursor;

/// The widget this view is (`widget` of the block `task-row`).
pub const WIDGET: &str = "TaskRow";

/// The view of the block `task-row`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        let rows = rows_of(&input, &ctx);
        Box::new(MountedRows(Arc::new(Mutex::new(Held {
            ctx,
            rows,
            cursor: None,
        }))))
    }
}

fn rows_of(input: &FormInput, ctx: &DrawCtx) -> Rows {
    read_rows(input, &|k| (ctx.say)(k), &|key| {
        ctx.write.can(TOGGLE, Some(key))
    })
}

struct Held {
    ctx: DrawCtx,
    rows: Rows,
    /// The key of the line the keyboard cursor is on.
    cursor: Option<String>,
}

#[derive(Clone)]
struct MountedRows(Arc<Mutex<Held>>);

impl MountedRows {
    fn held(&self) -> MutexGuard<'_, Held> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Ask the module to tick (or untick) `key`; an accepted write shows what the module now says.
    fn toggle(&self, key: &str) {
        let (write, done, reread) = {
            let held = self.held();
            let Some(done) = held.rows.is_done(key) else {
                return;
            };
            (held.ctx.write.clone(), done, held.ctx.reread.clone())
        };
        if !write.can(TOGGLE, Some(key)) {
            return;
        }
        let arg = WriteArg {
            name: "done".to_owned(),
            value: WriteValue::Flag(!done),
        };
        // a refusal changes nothing here: the module's next answer says what stands
        if write.send(TOGGLE, Some(key), &[arg]).is_ok() {
            self.refresh(reread());
        }
    }

    fn open(&self, key: &str) {
        let write = self.held().ctx.write.clone();
        if write.can(OPEN, Some(key)) {
            let _ = write.send(OPEN, Some(key), &[]);
        }
    }

    /// The cursor `step` lines on (from nothing: the first, or backwards the last).
    fn walk(&self, step: isize) -> bool {
        let mut held = self.held();
        let keys = held.rows.keys();
        if keys.is_empty() {
            return false;
        }
        let at = held
            .cursor
            .as_ref()
            .and_then(|c| keys.iter().position(|k| k == c));
        let next = match at {
            Some(i) => i.saturating_add_signed(step).min(keys.len() - 1),
            None if step < 0 => keys.len() - 1,
            None => 0,
        };
        held.cursor = Some(keys[next].clone());
        true
    }

    fn jump(&self, last: bool) -> bool {
        let mut held = self.held();
        let keys = held.rows.keys();
        let key = if last { keys.last() } else { keys.first() };
        let Some(key) = key.cloned() else {
            return false;
        };
        held.cursor = Some(key);
        true
    }

    fn events(&self) -> Events {
        let (a, b) = (self.clone(), self.clone());
        let tick: Say = Arc::new(move |k: &str| a.toggle(k));
        let open: Say = Arc::new(move |k: &str| b.open(k));
        Events {
            tick: Some(tick),
            open: Some(open),
            ..Events::default()
        }
    }
}

impl Mounted for MountedRows {
    fn build(&self, width: f32, height: f32) -> Div {
        let events = self.events();
        let mut sections = self.held().rows.sections.clone();
        let cursor = self.held().cursor.clone();
        let mut page = div()
            .class("fs-plan-page")
            .flex_col()
            .w(width)
            .flex_shrink_0();
        for section in &mut sections {
            for line in &mut section.lines {
                if cursor.as_deref() == Some(line.key.as_str()) {
                    line.cursor = Cursor::On;
                }
            }
            page = page.child(section.build("tasks", &events));
        }
        div()
            .class("fs-plan-root")
            .w(width)
            .h(height)
            .child(scroll().w_full().h_full().content(page))
    }

    fn refresh(&self, input: FormInput) {
        let mut held = self.held();
        held.rows = rows_of(&input, &held.ctx);
        // a cursor on a row the module no longer has is gone
        if held
            .cursor
            .as_ref()
            .is_some_and(|c| !held.rows.keys().contains(c))
        {
            held.cursor = None;
        }
        drop(held);
        redraw();
    }

    fn key(&self, key: Key) -> bool {
        match key {
            Key::Down => self.walk(1),
            Key::Up => self.walk(-1),
            Key::Home => self.jump(false),
            Key::End => self.jump(true),
            Key::Space | Key::Enter => {
                let Some(cursor) = self.held().cursor.clone() else {
                    return false;
                };
                if key == Key::Space {
                    self.toggle(&cursor);
                } else {
                    self.open(&cursor);
                }
                true
            }
            _ => false,
        }
    }
}

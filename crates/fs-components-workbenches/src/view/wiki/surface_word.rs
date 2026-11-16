//! A double click on a word of the surface marks the word.
//!
//! Pattern: Adapter — Blinc's `text_area` has no word marking (its own handler says so; `text_input` has one), so the surface
//! counts the clicks and asks Blinc's own word boundaries (`text_edit::word_at_position`, the ones `text_input` uses).

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use blinc_layout::widgets::text_area::{SharedTextAreaState, TextPosition, refresh_text_area};
use blinc_layout::widgets::text_edit::word_at_position;

/// Two clicks on the same place within this time are a double click.
const DOUBLE: Duration = Duration::from_millis(400);

/// The last click: when and where (the cursor the first click put there).
#[derive(Default)]
pub(super) struct Clicks(Mutex<Option<(Instant, TextPosition)>>);

impl Clicks {
    /// A click has just put the cursor of `state`; when it is the second one on that place, the word under the cursor is
    /// marked (the cursor behind it, the mark in front of it).
    pub(super) fn clicked(&self, state: &SharedTextAreaState) {
        let mut area = state.lock().unwrap_or_else(PoisonError::into_inner);
        let (now, at) = (Instant::now(), area.cursor);
        let mut last = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let second = last
            .take()
            .is_some_and(|(then, was)| was == at && now.duration_since(then) <= DOUBLE);
        if !second {
            *last = Some((now, at));
            return;
        }
        drop(last);
        let Some(line) = area.lines.get(at.line) else {
            return;
        };
        let (from, to) = word_at_position(line, at.column);
        if from == to {
            return;
        }
        area.selection_start = Some(TextPosition::new(at.line, from));
        area.cursor = TextPosition::new(at.line, to);
        drop(area);
        refresh_text_area(state);
    }
}

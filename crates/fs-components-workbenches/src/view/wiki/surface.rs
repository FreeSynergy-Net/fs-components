//! The writing surface of the block editor: one text area for the whole page, no box around a block.
//!
//! Pattern: Observer — the surface keeps the text of the page in one text area (so a switch from one block to the next is
//! only the cursor moving: there is no second field whose state could be lost), and tells the owner after every change.
//! Enter in a paragraph puts the next block in (a gap, the block itself stays unseen), a list goes on, `/` opens the block
//! menu under the cursor, Backspace at the start of a block joins it to the one above (`surface_text`).

use std::sync::{Arc, Mutex, PoisonError};

use blinc_core::reactive::{Signal, signal};
use blinc_layout::widgets::text_area::{
    SharedTextAreaState, TextPosition, focus_text_area, refresh_text_area,
};
use fs_components_primitives::FieldData;
use fs_components_widgets::SlashLabels;

use super::surface_menu::Open;
use super::surface_text::{
    after_backspace, after_enter, block_at, block_texts, entries, is_gap, spans,
};

/// The way the whole text of the page goes back to the editor's owner.
pub(super) type Report = Arc<dyn Fn(&str) + Send + Sync>;

/// The text area of one edit and everything that belongs to it.
pub(super) struct Surface {
    pub(super) state: SharedTextAreaState,
    _keep: FieldData,
    /// Counted up when the area has to be built again (its width or its number of lines changed).
    pub(super) grow: Signal<u64>,
    /// The text after the last change: what "one line break more" is measured against.
    last: Mutex<String>,
    pub(super) menu: Mutex<Option<Open>>,
    pub(super) menu_signal: Signal<u64>,
    pub(super) words: Mutex<SlashLabels>,
    announce_to: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    /// The column (in characters) the person is after on the lines; an arrow key over the gap between blocks keeps it.
    want: Mutex<usize>,
}

fn locked<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Surface {
    /// A surface holding `text`.
    pub fn new(text: &str) -> Arc<Self> {
        let keep = FieldData::new();
        let state = crate::view::new_area(text, &keep);
        let now = locked(&state).value();
        Arc::new(Self {
            state,
            _keep: keep,
            grow: signal(0u64),
            last: Mutex::new(now),
            menu: Mutex::new(None),
            menu_signal: signal(0u64),
            words: Mutex::new(SlashLabels::default()),
            announce_to: Mutex::new(None),
            want: Mutex::new(0),
        })
    }

    /// The text of the whole surface.
    pub fn text(&self) -> String {
        locked(&self.state).value()
    }

    /// The cursor as a byte offset in [`Surface::text`].
    pub fn cursor(&self) -> usize {
        let d = locked(&self.state);
        let before: usize = d
            .lines
            .iter()
            .take(d.cursor.line)
            .map(|l| l.len() + 1)
            .sum();
        let line = d.lines.get(d.cursor.line).map_or("", String::as_str);
        before
            + line
                .chars()
                .take(d.cursor.column)
                .map(char::len_utf8)
                .sum::<usize>()
    }

    /// Put `text` into the area with the cursor at byte `at`, and draw the area again.
    pub fn set(&self, text: &str, at: usize) {
        {
            let mut d = locked(&self.state);
            // Every line, also an empty last one (`set_value` drops it, and a line break at the end is what Enter makes).
            d.set_value(text);
            d.lines = text.split('\n').map(str::to_owned).collect();
            let before = &text[..at.min(text.len())];
            let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count());
            d.cursor = TextPosition::new(before.matches('\n').count(), column);
            d.compute_visual_lines();
        }
        refresh_text_area(&self.state);
        self.note_column();
    }

    /// Keep the text now as the one the next change is measured against.
    pub fn remember(&self) {
        *locked(&self.last) = self.text();
    }

    /// Tell the owner the whole text now.
    pub fn announce(&self) {
        let to = locked(&self.announce_to).clone();
        if let Some(to) = to {
            to();
        }
    }

    /// Who hears the whole text after a change.
    pub fn announce_to(&self, to: Arc<dyn Fn() + Send + Sync>) {
        *locked(&self.announce_to) = Some(to);
    }

    /// The text changed (the area made the change, or an edit of the contract did): what Enter and Backspace mean for blocks,
    /// and the block menu.
    pub fn on_changed(&self) {
        self.note_column();
        let before = locked(&self.last).clone();
        let (now, cursor) = (self.text(), self.cursor());
        // Enter is exactly one line break more, at the cursor; anything else is typing, pasting or deleting.
        let is_enter = cursor > 0
            && now.is_char_boundary(cursor - 1)
            && now.as_bytes().get(cursor - 1) == Some(&b'\n')
            && format!("{}{}", &now[..cursor - 1], &now[cursor..]) == before;
        if is_enter {
            if let Some(kind) = self.marked() {
                self.choose(kind, &before);
                return;
            }
            let (text, at) = after_enter(&before, cursor - 1);
            if text != now || at != cursor {
                self.set(&text, at);
            }
        } else if let Some((text, at)) = after_backspace(&before, &now, cursor) {
            self.set(&text, at);
        }
        self.refresh_menu();
        self.remember();
    }

    /// An arrow key moved the cursor up or down (`down` says which way): when it came to rest on the blank line that only
    /// separates two blocks, it goes on to the block, so one key press goes from one block to the next as in Notion (the gap
    /// is no stop; an empty block made by Enter is). `select` keeps a selection growing.
    pub fn hop_gap(&self, down: bool, select: bool) {
        let text = self.text();
        if !is_gap(&text, self.cursor()) {
            return;
        }
        {
            let mut d = locked(&self.state);
            if down {
                d.move_down(select);
            } else {
                d.move_up(select);
            }
            let wanted = *locked(&self.want);
            let len = d.lines.get(d.cursor.line).map_or(0, |l| l.chars().count());
            d.cursor.column = wanted.min(len);
        }
        refresh_text_area(&self.state);
    }

    /// Remember the column of the cursor as the one the person is after (not on a blank line: that one has no column to keep).
    pub fn note_column(&self) {
        let column = locked(&self.state).cursor.column;
        let text = self.text();
        if !is_gap(&text, self.cursor()) {
            *locked(&self.want) = column;
        }
    }

    /// Where the cursor is: the block and the bytes into it; none on the gap between two blocks.
    pub fn cursor_in_block(&self) -> Option<(usize, usize)> {
        let (text, at) = (self.text(), self.cursor());
        if is_gap(&text, at) {
            return None;
        }
        let i = block_at(&text, at);
        let start = entries(&text).get(i).map_or(0, |e| e.start);
        Some((i, at.saturating_sub(start)))
    }

    /// The text that is selected (several blocks may be in it), if any.
    pub fn selected_text(&self) -> Option<String> {
        locked(&self.state).selected_text()
    }

    /// The blocks of the surface, the empty one at the end if the person is about to write there.
    pub fn block_texts(&self) -> Vec<String> {
        block_texts(&self.text())
    }

    /// The block that has the keyboard, if the area has it.
    pub fn focused_block(&self) -> Option<usize> {
        let focused = locked(&self.state).is_focused();
        focused.then(|| block_at(&self.text(), self.cursor()))
    }

    /// Give the keyboard to the surface with the cursor `column` bytes into block `i` (the end of the text for the empty
    /// block at the end).
    pub fn put_cursor(&self, i: usize, column: usize) {
        let text = self.text();
        let found = spans(&text);
        let at = found
            .get(i)
            .map_or(text.len(), |&(s, e)| (s + column).min(e));
        self.set(&text, at);
        self.remember();
        focus_text_area(&self.state);
    }

    /// Press Enter in block `i` of the document (blocks that are not blank), `at` bytes into it: what the keyboard does,
    /// without the keyboard (the contract's edits go through here, so tests and window share the one code path).
    pub fn enter_at(&self, i: usize, at: usize) {
        let text = self.text();
        let Some(&(start, end)) = spans(&text).get(i) else {
            return;
        };
        let place = (start + at).min(end);
        self.set(
            &format!("{}\n{}", &text[..place], &text[place..]),
            place + 1,
        );
        self.on_changed();
    }

    /// Press Backspace in block `i` of the document, `at` bytes into it, as the keyboard does: one character goes, at the start of
    /// a block the line break that is left of the two between the blocks, and the change decides what that means for blocks.
    pub fn backspace_at(&self, i: usize, at: usize) {
        let text = self.text();
        let Some(&(start, _)) = spans(&text).get(i) else {
            return;
        };
        let place = (start + at).min(text.len());
        let Some((from, _)) = text[..place].char_indices().next_back() else {
            return;
        };
        self.remember();
        self.set(&format!("{}{}", &text[..from], &text[place..]), from);
        self.on_changed();
    }

    /// Tick or untick the task item on the line `at` is on in block `i` of the document (a click on its box).
    pub fn toggle_task_at(&self, i: usize, at: usize) {
        let text = self.text();
        let Some(&(start, end)) = spans(&text).get(i) else {
            return;
        };
        if let Some(now) = fs_components_editor::toggle_task_line(&text[start..end], at) {
            let cursor = self.cursor();
            self.set(&format!("{}{now}{}", &text[..start], &text[end..]), cursor);
            self.remember();
        }
    }

    /// Press Enter where the cursor is (what the keyboard does).
    pub fn enter_at_cursor(&self) {
        let (text, at) = (self.text(), self.cursor());
        self.set(&format!("{}\n{}", &text[..at], &text[at..]), at + 1);
        self.on_changed();
    }

    /// Type `text` into block `i` of the blocks of [`Surface::block_texts`], `at` bytes into it: a block that is not a
    /// block with text is the empty place at the end of the surface.
    pub fn type_in_block(&self, i: usize, at: usize, text: &str) {
        let all = self.text();
        let place = entries(&all)
            .get(i)
            .map_or(all.len(), |e| (e.start + at).min(e.end));
        self.set(
            &format!("{}{text}{}", &all[..place], &all[place..]),
            place + text.len(),
        );
        self.on_changed();
    }

    /// Type `text` (no line break) into block `i` of the document, `at` bytes into it.
    pub fn type_at(&self, i: usize, at: usize, text: &str) {
        let all = self.text();
        let Some(&(start, end)) = spans(&all).get(i) else {
            return;
        };
        let place = (start + at).min(end);
        self.set(
            &format!("{}{text}{}", &all[..place], &all[place..]),
            place + text.len(),
        );
        self.on_changed();
    }
}

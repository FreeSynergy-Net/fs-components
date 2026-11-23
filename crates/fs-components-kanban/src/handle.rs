//! The handle of a board: shared state, source, events, and the redraw.
//!
//! Pattern: Facade + Observer — one clonable handle over [`BoardState`]. Every gesture, button and
//! key ends in one method here; a finished move goes to the host as a request, its answer becomes
//! the notice; the live area draws again (only the board, never per frame).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_components_fields::{Look, Say};
use fs_components_widgets::Key;
use fs_components_widgets::live::Live;

use crate::model::{Column, Operation};
use crate::quick_add::Drafts;
use crate::source::{Answer, BoardEvents, BoardSource, NoEvents};
use crate::state::{BoardState, Step};

/// A kanban board. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct KanbanBoard {
    state: Arc<Mutex<BoardState>>,
    pub(crate) source: Arc<dyn BoardSource>,
    events: Arc<dyn BoardEvents>,
    pub(crate) say: Arc<dyn Say>,
    pub(crate) look: Look,
    pub(crate) live: Live,
    swallow: Arc<AtomicBool>,
    pub(crate) drafts: Drafts,
}

impl KanbanBoard {
    /// A board over `source`, with words from `say`, in `look`.
    pub fn new(source: Arc<dyn BoardSource>, say: Arc<dyn Say>, look: Look) -> Self {
        Self {
            state: Arc::default(),
            source,
            events: Arc::new(NoEvents),
            say,
            look,
            live: Live::new(),
            swallow: Arc::default(),
            drafts: Drafts::default(),
        }
    }

    /// Tell `events` of the reader's requests.
    #[must_use]
    pub fn events(mut self, events: Arc<dyn BoardEvents>) -> Self {
        self.events = events;
        self
    }

    /// A copy of the state.
    pub fn snapshot(&self) -> BoardState {
        self.lock().clone()
    }

    fn lock(&self) -> MutexGuard<'_, BoardState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn can_move(&self) -> bool {
        self.source.supports(Operation::Move)
    }

    /// Run `f` on the state with the columns, then carry out what it came to.
    fn act(&self, f: impl FnOnce(&mut BoardState, &[Column], bool) -> Step) -> Step {
        let cols = self.source.columns();
        // Ask the host before taking the lock: `supports` may call back into the board.
        let can_move = self.can_move();
        let step = f(&mut self.lock(), &cols, can_move);
        match &step {
            Step::Select(key) => self.events.select_card(key),
            Step::Move(m) => {
                if let Answer::Refused(code) =
                    self.events.move_card_at(&m.key, &m.from, &m.to, m.index)
                {
                    self.lock().notice = Some(code);
                }
            }
            Step::Ignored => return step,
            _ => {}
        }
        self.live.refresh();
        step
    }

    /// Choose a card.
    pub fn select(&self, key: &str) -> Step {
        self.act(|s, _, _| s.select(key))
    }

    /// Pick a card up: the start of a frontend's gesture.
    pub fn grab(&self, key: &str) -> Step {
        self.act(|s, c, m| s.grab(c, m, key))
    }

    /// The held card is over `column`.
    pub fn hover(&self, column: &str) -> Step {
        self.act(|s, c, _| s.hover(c, column))
    }

    /// The held card is over `column`, in front of the card at `before` (`None` = at the end): a frontend that knows the card under
    /// the pointer.
    pub fn hover_at(&self, column: &str, before: Option<usize>) -> Step {
        self.act(|s, c, _| s.hover_at(c, column, before))
    }

    /// Put the held card down.
    pub fn drop_held(&self) -> Step {
        self.act(|s, _, m| s.drop_held(m))
    }

    /// Give the held card up.
    pub fn cancel(&self) -> Step {
        self.act(|s, _, _| s.cancel())
    }

    /// Hold over `column` and put down there: a click on a column while a card is held.
    pub fn drop_on(&self, column: &str) -> Step {
        self.act(|s, c, m| s.drop_at(c, m, column))
    }

    /// Move a card one column left (-1) or right (1): the move buttons.
    pub fn step_over(&self, key: &str, dir: i32) -> Step {
        self.act(|s, c, m| s.step_over(c, m, key, dir))
    }

    /// Move a card one place up (-1) or down (1) in its column: Alt+Up and Alt+Down, mapped by the frontend.
    pub fn reorder(&self, key: &str, dir: i32) -> Step {
        self.act(|s, c, m| s.reorder(c, m, key, dir))
    }

    /// A key press; true when the board used it.
    pub fn key(&self, key: Key) -> bool {
        self.act(|s, c, m| s.key(c, m, key)) != Step::Ignored
    }

    /// The reader asked for an operation (`column` for rename and limit). Unsupported ones refuse.
    pub fn request(&self, op: Operation, column: Option<&str>) {
        if !self.source.supports(op) {
            self.lock().notice = Some("locked".into());
            self.live.refresh();
            return;
        }
        match (op, column) {
            (Operation::AddColumn, _) => self.events.add_column(),
            (Operation::Rename, Some(c)) => self.events.rename_column(c),
            (Operation::Limit, Some(c)) => self.events.set_limit(c),
            _ => {}
        }
    }

    /// Add a card titled `title` to `column` (the quick-add row). False, and nothing is asked of the host, while the host does not
    /// support [`Operation::AddCard`] (the notice says `locked`) or the title is only blanks; a refusal keeps the typed text and
    /// shows its reason.
    pub fn add_card(&self, column: &str, title: &str) -> bool {
        if !self.source.supports(Operation::AddCard) {
            self.lock().notice = Some("locked".into());
            self.live.refresh();
            return false;
        }
        let title = title.trim();
        if title.is_empty() {
            return false;
        }
        let accepted = match self.events.add_card(column, title) {
            Answer::Accepted => {
                self.drafts.clear(column);
                self.lock().notice = None;
                true
            }
            Answer::Refused(code) => {
                self.lock().notice = Some(code);
                false
            }
        };
        self.live.refresh();
        accepted
    }

    /// Add the card typed in the quick-add row of `column`: the Enter key and the button.
    pub fn submit_quick(&self, column: &str) -> bool {
        self.add_card(column, &self.drafts.text(column))
    }

    /// What is typed in the quick-add row of `column`.
    pub fn quick_text(&self, column: &str) -> String {
        self.drafts.text(column)
    }

    /// Give the keyboard to the quick-add field of `column` (on the next frame).
    pub fn focus_quick(&self, column: &str) {
        self.drafts.focus(column);
    }

    /// Put `text` in the quick-add row of `column` (a host that pre-fills it; tests).
    pub fn set_quick_text(&self, column: &str, text: &str) {
        self.drafts.set(column, text);
    }

    /// A child has handled the click that is bubbling up to its column.
    pub(crate) fn swallow(&self) {
        self.swallow.store(true, Ordering::Relaxed);
    }

    /// Whether a child handled the click now reaching the column; clears the mark.
    pub(crate) fn swallowed(&self) -> bool {
        self.swallow.swap(false, Ordering::Relaxed)
    }

    /// Read the source again and draw: the host changed its data.
    pub fn refresh(&self) {
        self.live.refresh();
    }

    /// The element of `width` x `height` pixels. The keyboard binding sits on a stable outer box;
    /// inside it a live area is built again after each change, nothing else.
    pub fn build(
        &self,
        width: f32,
        height: f32,
    ) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        let (me, keys) = (self.clone(), self.clone());
        let inner = self
            .live
            .region("board", move || crate::view::board(&me, width, height));
        crate::view::bind_keys(
            blinc_layout::div::div().id("kb-root").w(width).h(height),
            Arc::new(move |k| keys.key(k)),
        )
        .child(inner)
    }
}

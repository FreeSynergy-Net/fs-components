//! The board workbench as a thing a host places: one clonable handle over the state, the program's source and events, and the
//! redraws.
//!
//! Pattern: Facade + Observer (+ Template method for the view: head, columns, frame, line under the board; Strategy for what
//! the program decides). Every gesture, key and command ends in a method here; a finished move is a request to the program, its
//! [`Answer`] becomes the line under the board (and the question or the undo). The workbench never writes, ranks or stores:
//! after an answer it reads the source again. The lock is never held while the program is called or elements are built.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::selector::ScrollRef;
use fs_components_fields::Look;
use fs_components_styles::StyleId;
use fs_components_widgets::keys::KeyEvent;
use fs_components_widgets::live::Live;
use fs_components_widgets::plan::Entry;

use super::detail::DetailContent;
use super::drag::Moved;
use super::labels::Labels;
use super::model::{ContextTarget, FilterState, MoveRequest, NewCard};
use super::scroll;
use super::source::{BoardEvents, BoardSource, NoEvents};
use super::state::{Action, BoardState};

/// Called when the reader asks for a context menu: what it is for, where, and the command ids to offer. The shell opens the menu
/// (`AppWidget::open_context_menu`) and sends the chosen id back through [`BoardBench::run`].
pub type ContextHook = Arc<dyn Fn(&ContextTarget, (f32, f32), &[String]) + Send + Sync>;

/// A board workbench. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct BoardBench {
    pub(super) id: String,
    pub(super) state: Arc<Mutex<BoardState>>,
    pub(super) source: Arc<dyn BoardSource>,
    pub(super) events: Arc<dyn BoardEvents>,
    pub(super) detail: Option<Arc<dyn DetailContent>>,
    pub(super) labels: Labels,
    pub(super) live: Live,
    pub(super) drag_live: Live,
    pub(super) entries: Arc<Mutex<HashMap<String, Entry>>>,
    pub(super) many: Arc<Mutex<Option<Vec<NewCard>>>>,
    pub(super) filter: Arc<Mutex<FilterState>>,
    pub(super) scroll: ScrollRef,
    pub(super) hook: Option<ContextHook>,
    pub(super) claimed: Arc<AtomicBool>,
    pub(super) title: Option<String>,
    pub(super) look: Look,
}

impl BoardBench {
    /// A board over `source`, with words from `labels`; it hears nothing until [`BoardBench::events`].
    pub fn new(source: Arc<dyn BoardSource>, labels: Labels) -> Self {
        let state = BoardState::new(source.as_ref());
        Self {
            id: "fs-board".into(),
            state: Arc::new(Mutex::new(state)),
            source,
            events: Arc::new(NoEvents),
            detail: None,
            labels,
            live: Live::new(),
            drag_live: Live::new(),
            entries: Arc::default(),
            many: Arc::default(),
            filter: Arc::default(),
            scroll: fs_components_primitives::scroll_ref(),
            hook: None,
            claimed: Arc::default(),
            title: None,
            look: Look::new(StyleId::Admin, false),
        }
    }

    /// The element id prefix (cards are `<id>-card-<key>`, column heads `<id>-head-<key>`, quick entries `<id>-quick-<key>`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Tell `events` what the reader does.
    #[must_use]
    pub fn events(mut self, events: Arc<dyn BoardEvents>) -> Self {
        self.events = events;
        self
    }

    /// Put the program's content into the frame at the side.
    #[must_use]
    pub fn detail(mut self, detail: Arc<dyn DetailContent>) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Hear of requests for a context menu (right click on a card, a column head or the empty board).
    #[must_use]
    pub fn on_context(mut self, hook: ContextHook) -> Self {
        self.hook = Some(hook);
        self
    }

    /// The look the facts on the cards are drawn in (the field components take their style and light or dark from it); the board
    /// itself takes its colours from the style sheet of the window.
    #[must_use]
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    /// The large heading above the board (the name of the board shown).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, BoardState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A copy of the state (what the view draws from).
    pub fn snapshot(&self) -> BoardState {
        self.lock().clone()
    }

    /// Run `f` on the state without changing it.
    pub fn read<T>(&self, f: impl FnOnce(&BoardState) -> T) -> T {
        f(&self.lock())
    }

    pub(super) fn refresh(&self) {
        self.live.refresh();
    }

    // ---- data -------------------------------------------------------------------------------------------------------------

    /// Read the source again and draw.
    pub fn reload(&self) {
        self.lock().load(self.source.as_ref());
        self.refresh();
    }

    /// Reload only if the source says its data changed (call it when the program's feed says so); true if it did.
    pub fn sync(&self) -> bool {
        let stale = self.lock().is_stale(self.source.as_ref());
        if stale {
            self.reload();
        }
        stale
    }

    /// The filter was changed by the reader: remembered here and told to the program (which filters; `columns()` comes back shorter).
    pub fn set_filter(&self, filter: &FilterState) {
        filter.clone_into(&mut self.filter.lock().unwrap_or_else(PoisonError::into_inner));
        self.events.filter_changed(filter);
        self.reload();
    }

    /// The filter as it is now.
    pub fn filter(&self) -> FilterState {
        self.filter
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    // ---- the pointer ------------------------------------------------------------------------------------------------------

    /// The button went down on `card` (window coordinates).
    pub fn press(&self, card: &str, window: (f32, f32)) {
        self.press_card(card, window);
    }

    /// The pointer moved while the button is down on `card`: `window` is the pointer in the window, `local` it in the element
    /// of the card. Draws only the layer of the carried card (the cards stay, so Blinc keeps the gesture alive).
    pub fn drag(&self, card: &str, window: (f32, f32), local: (f32, f32)) {
        let (moved, scroll_step) = {
            let mut state = self.lock();
            let Some((x, y, ..)) = state.geometry().card_rect(card) else {
                return;
            };
            let at = (x + local.0, y + local.1);
            let moved = state.pointer(window, at, self.source.as_ref());
            (moved, self.scroll_step(at))
        };
        if moved == Moved::Nothing {
            return;
        }
        if scroll_step != (0.0, 0.0) {
            self.scroll.scroll_by(scroll_step.0, scroll_step.1);
        }
        self.drag_live.refresh();
    }

    /// Near the edge of the viewport the board scrolls: the step for a pointer at `at` (in the columns row).
    fn scroll_step(&self, at: (f32, f32)) -> (f32, f32) {
        let Some((w, h)) = self.scroll.viewport_size() else {
            return (0.0, 0.0);
        };
        let (ox, oy) = self.scroll.offset();
        // The offset is the distance scrolled; its sign is Blinc's (the magnitude is what the viewport's start is).
        let (ox, oy) = (ox.abs(), oy.abs());
        (
            scroll::step(at.0, ox, ox + w),
            scroll::step(at.1, oy, oy + h),
        )
    }

    /// The button went up.
    pub fn release(&self) {
        self.sync_rights();
        let action = self.lock().release(self.source.as_ref());
        self.perform(action);
        self.drag_live.refresh();
    }

    // ---- the keyboard -----------------------------------------------------------------------------------------------------

    /// A key press. True if the board used it (the caller may stop it travelling on).
    pub fn press_key(&self, event: &KeyEvent) -> bool {
        self.sync_rights();
        let action = self.lock().key(event, self.source.as_ref());
        let used = action != Action::None;
        self.perform(action);
        used
    }

    // ---- doing ------------------------------------------------------------------------------------------------------------

    pub(super) fn perform(&self, action: Action) {
        match action {
            Action::None => {}
            Action::Redraw => self.refresh(),
            Action::Open(card) => self.open(&card),
            Action::Move(request, undo) => self.move_to(&request, Some(undo)),
            Action::QuickOpen(column) => {
                self.lock().set_quick(Some(&column));
                self.refresh();
            }
            Action::Undo => self.undo(),
        }
    }

    /// Open a card in the frame at the side, and tell the program.
    pub fn open(&self, card: &str) {
        self.lock().set_open(Some(card));
        self.events.opened(card);
        self.refresh();
    }

    /// Close the frame at the side.
    pub fn close_detail(&self) {
        self.lock().set_open(None);
        self.refresh();
    }

    /// Ask the program to carry out a move (the card is already at its new place on the screen) and keep or set back by its answer.
    fn move_to(&self, request: &MoveRequest, undo: Option<MoveRequest>) {
        let answer = self.events.moved(request);
        self.lock().answered(
            request,
            answer,
            undo,
            &self.labels.moved,
            self.source.as_ref(),
        );
        self.refresh();
    }

    /// The reader answered the program's question (an agent holds the card: move it anyway?).
    pub fn answer_question(&self, yes: bool) {
        let asked = self.lock().take_question();
        if let (true, Some((mut request, _))) = (yes, asked) {
            request.confirmed = true;
            self.move_to(&request, None);
        } else {
            self.refresh();
        }
    }

    /// Take the last move back (a move of the same kind as any other: the program writes the counter move).
    pub fn undo(&self) {
        let undo = self.lock().notice().and_then(|n| n.undo.clone());
        match undo {
            Some(request) => {
                self.lock().clear_notice();
                self.move_to(&request, None);
            }
            None => self.refresh(),
        }
    }

    // ---- the foot of a column ---------------------------------------------------------------------------------------------

    // ---- a column's short menu --------------------------------------------------------------------------------------------
}

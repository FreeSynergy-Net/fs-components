//! The board workbench as a thing a host places: one clonable handle over the state, the program's source and events, and the
//! redraws.
//!
//! Pattern: Facade + Observer (+ Template method for the view: head, columns, frame, line under the board; Strategy for what
//! the program decides). Every gesture, key and command ends in a method here; a finished move is a request to the program, its
//! [`Answer`] becomes the line under the board (and the question or the undo). The workbench never writes, ranks or stores:
//! after an answer it reads the source again. `load` asks the program (`columns`, `locked`, `generation`, `supports`) with the state lock held; gestures ask `supports` without it (see Known limits in DESIGN.md).

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::selector::ScrollRef;
use fs_components_fields::Look;
use fs_components_styles::StyleId;
use fs_components_widgets::keys::KeyEvent;
use fs_components_widgets::live::Live;
use fs_components_widgets::plan::Entry;

use crate::detail::DetailContent;
use crate::labels::Labels;
use crate::requests::{Answer, ContextTarget, FilterState, MoveRequest, NewCard};
use crate::source::{BoardEvents, BoardSource, NoEvents};
use crate::state::{Action, BoardState};

/// Called when the reader asks for a context menu: what it is for, where, and the command ids to offer. The shell opens the menu
/// (`AppWidget::open_context_menu`) and sends the chosen id back through [`BoardBench::run`].
pub type ContextHook = Arc<dyn Fn(&ContextTarget, (f32, f32), &[String]) + Send + Sync>;

/// A board workbench. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct BoardBench {
    pub(crate) id: String,
    pub(crate) state: Arc<Mutex<BoardState>>,
    pub(crate) source: Arc<dyn BoardSource>,
    pub(crate) events: Arc<dyn BoardEvents>,
    pub(crate) detail: Option<Arc<dyn DetailContent>>,
    pub(crate) labels: Labels,
    pub(crate) live: Live,
    pub(crate) drag_live: Live,
    pub(crate) entries: Arc<Mutex<HashMap<String, Entry>>>,
    pub(crate) many: Arc<Mutex<Option<Vec<NewCard>>>>,
    pub(crate) filter: Arc<Mutex<FilterState>>,
    pub(crate) scroll: ScrollRef,
    pub(crate) hook: Option<ContextHook>,
    pub(crate) claimed: Arc<AtomicBool>,
    pub(crate) title: Option<String>,
    pub(crate) look: Look,
    /// The card components drawn last, by element id (what the layer asks while one is carried).
    pub(crate) cards: Arc<Mutex<HashMap<String, fs_components_widgets::card::Card>>>,
    /// The card being carried: its element id and itself.
    pub(crate) active: Arc<Mutex<Option<(String, fs_components_widgets::card::Card)>>>,
    /// The reason of the place the carried card is over, if it refuses (becomes the notice when the card is let go there).
    pub(crate) pending: Arc<Mutex<Option<crate::requests::Reason>>>,
    /// The measured places of the board as drawn last.
    pub(crate) places: Arc<Mutex<Option<Arc<fs_components_widgets::card::MeasuredTargets>>>>,
    /// The grid (its cursor and its open lanes stay over every redraw of the board), with the shape it was made for.
    pub(crate) grid: Arc<Mutex<Option<(String, fs_components_lists::grid::Grid)>>>,
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
            cards: Arc::default(),
            active: Arc::default(),
            pending: Arc::default(),
            places: Arc::default(),
            grid: Arc::default(),
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

    pub(crate) fn lock(&self) -> MutexGuard<'_, BoardState> {
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

    /// Scroll the board to this offset (what the wheel or a host does).
    pub fn scroll_to_offset(&self, x: f32, y: f32) {
        self.scroll.set_scroll_offset(x, y);
    }

    /// How far the board is scrolled now (Blinc's own numbers).
    pub fn scroll_offset(&self) -> (f32, f32) {
        self.scroll.offset()
    }

    /// Draw the layer of the carried card (the line where it would land, the card at the pointer) again. It is made from measured
    /// places, so it is right only after a layout: a host that draws once (a picture) calls this after the layout.
    pub fn redraw_layer(&self) {
        self.drag_live.refresh();
    }

    pub(crate) fn refresh(&self) {
        self.live.refresh();
    }

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

    pub(crate) fn perform(&self, action: Action) {
        match action {
            Action::None => {}
            Action::Redraw => self.refresh(),
            Action::Open(card) => self.open(&card),
            Action::Move(request, undo) => {
                self.move_to(&request, Some(undo));
            }
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
    pub(crate) fn move_to(&self, request: &MoveRequest, undo: Option<MoveRequest>) -> Answer {
        let answer = self.events.moved(request);
        self.lock().answered(
            request,
            answer.clone(),
            undo,
            &self.labels.moved,
            self.source.as_ref(),
        );
        self.refresh();
        answer
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

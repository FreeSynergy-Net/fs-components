//! The board workbench as a thing a host places: one clonable handle over the state, the program's source and events, and the
//! redraws.
//!
//! Pattern: Facade + Observer (+ Template method for the view: head, columns, frame, line under the board; Strategy for what
//! the program decides). Every gesture, key and command ends in a method here; a finished move is a request to the program, its
//! [`Answer`] becomes the line under the board (and the question or the undo). The workbench never writes, ranks or stores:
//! after an answer it reads the source again. The lock is never held while the program is called or elements are built.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::selector::ScrollRef;
use fs_components_widgets::keys::KeyEvent;
use fs_components_widgets::live::Live;
use fs_components_widgets::plan::Entry;

use super::commands;
use super::detail::DetailContent;
use super::drag::Moved;
use super::labels::Labels;
use super::model::{Answer, ColumnAction, ContextTarget, FilterState, MoveRequest, NewCard};
use super::quick::{Quick, submit};
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
    state: Arc<Mutex<BoardState>>,
    pub(super) source: Arc<dyn BoardSource>,
    events: Arc<dyn BoardEvents>,
    pub(super) detail: Option<Arc<dyn DetailContent>>,
    pub(super) labels: Labels,
    pub(super) live: Live,
    pub(super) drag_live: Live,
    entries: Arc<Mutex<HashMap<String, Entry>>>,
    many: Arc<Mutex<Option<Vec<NewCard>>>>,
    filter: Arc<Mutex<FilterState>>,
    pub(super) scroll: ScrollRef,
    hook: Option<ContextHook>,
    claimed: Arc<AtomicBool>,
    pub(super) title: Option<String>,
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

    /// The large heading above the board (the name of the board shown).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The reader asked for a context menu at `at` (window coordinates): the board's lines first, then the program's.
    pub fn context(&self, target: &ContextTarget, at: (f32, f32)) {
        self.ask_context(target, at, true);
    }

    /// Like [`BoardBench::context`] for a press on a button (the column's "..." button): no right click is on its way to the board.
    pub fn context_by_click(&self, target: &ContextTarget, at: (f32, f32)) {
        self.ask_context(target, at, false);
    }

    fn ask_context(&self, target: &ContextTarget, at: (f32, f32), right_click: bool) {
        if !self.source.supports().context {
            return;
        }
        // A right click bubbles from the card to the board: the inner one claims it, the board's own handler then stands aside.
        match target {
            ContextTarget::Board if self.claimed.swap(false, Ordering::Relaxed) => return,
            ContextTarget::Board => {}
            _ if right_click => self.claimed.store(true, Ordering::Relaxed),
            _ => {}
        }
        let ids = match target {
            ContextTarget::Card(card) => self.card_menu(card, at),
            ContextTarget::Column(column) => {
                let mut ids = vec![commands::NEW.to_owned()];
                ids.extend(self.events.context(target, at));
                let _ = column;
                ids
            }
            ContextTarget::Board => self.events.context(target, at),
        };
        if let Some(hook) = &self.hook {
            hook(target, at, &ids);
        }
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

    fn refresh(&self) {
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
        self.lock().press(card, window);
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
        let action = self.lock().release(self.source.as_ref());
        self.perform(action);
        self.drag_live.refresh();
    }

    // ---- the keyboard -----------------------------------------------------------------------------------------------------

    /// A key press. True if the board used it (the caller may stop it travelling on).
    pub fn press_key(&self, event: &KeyEvent) -> bool {
        let action = self.lock().key(event, self.source.as_ref());
        let used = action != Action::None;
        self.perform(action);
        used
    }

    // ---- doing ------------------------------------------------------------------------------------------------------------

    fn perform(&self, action: Action) {
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

    /// Run a command id of the board (from a context menu line or a shortcut) on `card`: `board.open`, `board.up`, `board.down`,
    /// `board.top`, `board.bottom`, `board.to:<column>`, `board.undo`. False for an id that is not the board's.
    pub fn run(&self, id: &str, card: &str) -> bool {
        if id == commands::UNDO {
            self.undo();
            return true;
        }
        if id == commands::OPEN {
            self.open(card);
            return true;
        }
        let action = {
            let mut state = self.lock();
            let Some(at) = state.lanes().iter().enumerate().find_map(|(l, lane)| {
                lane.items
                    .iter()
                    .position(|i| i.key == card)
                    .map(|i| (l, i, lane.items.len()))
            }) else {
                return false;
            };
            let column = state.lanes()[at.0].key.clone();
            let source = Arc::clone(&self.source);
            match id {
                commands::UP => state.place(
                    card,
                    &column,
                    at.1.saturating_sub(1),
                    source.as_ref(),
                    false,
                ),
                commands::DOWN => state.place(
                    card,
                    &column,
                    (at.1 + 1).min(at.2 - 1),
                    source.as_ref(),
                    false,
                ),
                commands::TOP => state.place(card, &column, 0, source.as_ref(), false),
                commands::BOTTOM => state.place(card, &column, at.2 - 1, source.as_ref(), false),
                other => match commands::column_of(other) {
                    Some(to) => {
                        let end = state
                            .lanes()
                            .iter()
                            .find(|l| l.key == to)
                            .map_or(0, |l| l.items.len());
                        state.place(card, to, end, source.as_ref(), false)
                    }
                    None => return false,
                },
            }
        };
        self.perform(action);
        true
    }

    /// The lines of the menu of `card` (ids of the command model), as far as the program adds its own: the board's own come first.
    pub fn card_menu(&self, card: &str, at: (f32, f32)) -> Vec<String> {
        let target = ContextTarget::Card(card.to_owned());
        let own = {
            let state = self.lock();
            let Some((lane, index, len)) =
                state.lanes().iter().enumerate().find_map(|(l, lane)| {
                    lane.items
                        .iter()
                        .position(|i| i.key == card)
                        .map(|i| (l, i, lane.items.len()))
                })
            else {
                return Vec::new();
            };
            let others: Vec<String> = state
                .lanes()
                .iter()
                .enumerate()
                .filter(|(l, _)| *l != lane)
                .map(|(_, l)| l.key.clone())
                .collect();
            commands::card_lines(index, len, &others)
        };
        let mut ids: Vec<String> = own
            .into_iter()
            .filter(|l| l.enabled)
            .map(|l| l.id)
            .collect();
        ids.extend(self.events.context(&target, at));
        ids
    }

    // ---- the foot of a column ---------------------------------------------------------------------------------------------

    /// The quick entry of `column` (one per column, kept, so what is typed survives a redraw).
    pub(super) fn entry(&self, column: &str) -> Entry {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(column.to_owned())
            .or_insert_with(|| Entry::new(&self.labels.quick_hint))
            .clone()
    }

    /// Open the quick entry at the foot of `column`.
    pub fn open_quick(&self, column: &str) {
        self.lock().set_quick(Some(column));
        self.refresh();
    }

    /// Close the quick entry.
    pub fn close_quick(&self) {
        self.lock().set_quick(None);
        self.refresh();
    }

    /// Enter was pressed in the quick entry of `column` with `text` (Shift: at the top). One line makes one card, several ask first.
    /// If the program refuses, the text is put back into the entry and the reason stands under it: nothing typed is lost.
    pub fn quick_submit(&self, column: &str, text: &str, top: bool) {
        match submit(column, text, top) {
            Quick::Nothing => {}
            Quick::Create(cards) => self.create(column, text, &cards),
            Quick::Confirm(cards) => {
                *self.many.lock().unwrap_or_else(PoisonError::into_inner) = Some(cards);
                self.entry(column).set(text);
                self.refresh();
            }
        }
    }

    fn create(&self, column: &str, text: &str, cards: &[NewCard]) {
        let entry = self.entry(column);
        for card in cards {
            if let Answer::Refused(reason) = self.events.new_card(card) {
                entry.set(text);
                entry.set_notice(Some(&reason.text));
                self.refresh();
                return;
            }
        }
        entry.set_notice(None);
        self.lock().load(self.source.as_ref());
        self.refresh();
    }

    /// The reader answered "Create N cards?".
    pub fn answer_many(&self, yes: bool) {
        let cards = self
            .many
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let (true, Some(cards)) = (yes, cards) {
            if let Some(first) = cards.first() {
                let column = first.column.clone();
                let text = cards
                    .iter()
                    .map(|c| c.title.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                self.entry(&column).clear();
                self.create(&column, &text, &cards);
            }
        } else {
            self.refresh();
        }
    }

    /// The cards waiting for the reader's yes ("Create N cards?").
    pub fn pending_many(&self) -> Option<Vec<NewCard>> {
        self.many
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    // ---- a column's short menu --------------------------------------------------------------------------------------------

    /// Something chosen in a column's short menu: the program does it (it owns names, limits and the table).
    pub fn column_action(&self, column: &str, action: ColumnAction) {
        self.events.column_action(column, action);
    }
}

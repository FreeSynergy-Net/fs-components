//! The people workbench as a thing a program can place in its own window.
//!
//! Pattern: Facade + Observer — `PeopleWidget` holds the shared state and builds the element. A
//! search draws the list area (the list and its More button) again through a signal, so the search
//! field keeps the keyboard;
//! every other change asks the window for a full rebuild, because `cn` elements built inside a live
//! area get new keys and their old callbacks keep running (see the Blinc notes in `DESIGN.md`).
//!
//! No lock is held while elements are built: `on_state` runs its closure at once, so a guard alive in
//! the same statement would deadlock the interface thread. Views draw from a snapshot.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_core::{BlincContextState, Signal};
use blinc_layout::div::div;
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful_with_key;
use fs_components_commands::{Resolved, Target, TextKey, Trigger};
use fs_components_primitives::{FieldSize, SearchField};
use fs_components_widgets::Element;

use super::commands::PeopleRequest;
use super::labels::PeopleLabels;
use super::list_head::list_head;
use super::shown::Variant;
use super::source::PeopleSource;
use super::state::PeopleState;
use super::view::{Areas, detail, frame, list};
use crate::input_slots::InputSlots;
use crate::page_scroll::PageScroll;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Asked when a context menu should open: how it was asked for, what it acts on, and the lines to offer (ids of the command
/// model, [`PeopleState::person_menu`] and its kin). The shell opens the menu (`AppWidget::open_context_menu`), reads the words with
/// [`PeopleWidget::say`] and sends the chosen id back through [`PeopleWidget::run`].
pub type ContextHook = Arc<dyn Fn(&Trigger, &Target, &[Resolved]) + Send + Sync>;

/// Asked when the workbench cannot do what was chosen itself (write a mail, export, merge): the program does it.
pub type RequestHook = Arc<dyn Fn(&PeopleRequest) + Send + Sync>;

/// The people workbench, ready to be built into a layout. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct PeopleWidget {
    id: u64,
    pub(super) state: Arc<Mutex<PeopleState>>,
    field: SearchField,
    list_scroll: PageScroll,
    slots: InputSlots,
    page: PageScroll,
    pub(super) claimed: Arc<AtomicBool>,
    pub(super) context: Arc<Mutex<Option<ContextHook>>>,
    pub(super) request: Arc<Mutex<Option<RequestHook>>>,
}

impl PeopleWidget {
    /// The whole workbench over `source`.
    pub fn new(source: Arc<dyn PeopleSource>, labels: PeopleLabels) -> Self {
        Self::with_variant(source, labels, Variant::Full)
    }

    /// The compact variant (ADR 0037): list and detail, no editing, whatever the source can do.
    pub fn compact(source: Arc<dyn PeopleSource>, labels: PeopleLabels) -> Self {
        Self::with_variant(source, labels, Variant::Compact)
    }

    fn with_variant(source: Arc<dyn PeopleSource>, labels: PeopleLabels, variant: Variant) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let placeholder = labels.search.clone();
        let state = Arc::new(Mutex::new(PeopleState::new(source, labels, variant)));
        let field = {
            let state = Arc::clone(&state);
            SearchField::new(&placeholder)
                .size(FieldSize::Small)
                .on_change(Arc::new(move |term| {
                    locked(&state).search(term);
                    bump_list(id);
                }))
        };
        Self {
            id,
            state,
            field,
            list_scroll: PageScroll::default(),
            slots: InputSlots::default(),
            page: PageScroll::default(),
            claimed: Arc::new(AtomicBool::new(false)),
            context: Arc::new(Mutex::new(None)),
            request: Arc::new(Mutex::new(None)),
        }
    }

    /// Let the program open the context menus (right click, menu key, Shift+F10, the three dots and the three lines). Without a hook
    /// the workbench hangs its own panel on the two buttons and opens no menu on a right click.
    #[must_use]
    pub fn on_context(self, hook: ContextHook) -> Self {
        *lock_slot(&self.context) = Some(hook);
        self
    }

    /// Let the program do what the workbench cannot: write a mail, export, merge.
    #[must_use]
    pub fn on_request(self, hook: RequestHook) -> Self {
        *lock_slot(&self.request) = Some(hook);
        self
    }

    /// The words of a key of the command model of the lines this workbench offers (for the shell that draws the menu).
    pub fn say(&self, key: &TextKey) -> String {
        let state = locked(&self.state).clone();
        state.labels().say(key, &state)
    }

    /// Run `f` with the state, for a program's menus and commands. A closure, so the lock cannot be
    /// held across calls. After a change made this way call [`PeopleWidget::refresh`]. Do not call
    /// other methods of this widget from inside `f`.
    pub fn with_state<T>(&self, f: impl FnOnce(&mut PeopleState) -> T) -> T {
        f(&mut locked(&self.state))
    }

    /// The style sheet to add to the theme of the window.
    pub fn styles(&self) -> String {
        super::view::styles()
    }

    /// Draw again after the state was changed through [`PeopleWidget::with_state`].
    pub fn refresh(&self) {
        rebuild();
    }

    /// The element of the workbench, filling `width` x `height`. Call it inside the program's build
    /// function, not inside a live area.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let snapshot = locked(&self.state).clone();
        let actions = self.actions();
        let name = format!("fs-people-{}", self.id);
        let own_menus = lock_slot(&self.context).is_none();
        let head = list_head(&snapshot, &name, &actions, own_menus);
        let search = div()
            .id(format!("{name}-search"))
            .class("fs-people-search-field")
            .flex_1()
            .child(self.field.build());
        let live = {
            let (state, scroll, actions, name) = (
                Arc::clone(&self.state),
                self.list_scroll.clone(),
                actions.clone(),
                name.clone(),
            );
            stateful_with_key::<NoState>(format!("{name}-list-area"))
                .deps([list_signal(self.id).id()])
                .on_state(move |_| {
                    let snapshot = locked(&state).clone();
                    list(&snapshot, &name, &scroll, &actions)
                })
        };
        let person = detail(&snapshot, &name, &actions, &self.slots, &self.page, own_menus);
        frame(
            Areas {
                head,
                search,
                list: live,
                detail: person,
            },
            width,
            height,
        )
    }

    /// The name element ids of this widget begin with: `<name>-<row key>`, `<name>-save`, ...
    pub fn element_name(&self) -> String {
        format!("fs-people-{}", self.id)
    }
}

pub(super) fn lock_slot<T>(slot: &Mutex<T>) -> MutexGuard<'_, T> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(super) fn locked(state: &Mutex<PeopleState>) -> MutexGuard<'_, PeopleState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

fn list_signal(id: u64) -> Signal<u64> {
    BlincContextState::get().use_signal_keyed(&format!("fs-people-{id}-list"), || 0u64)
}

fn bump_list(id: u64) {
    if BlincContextState::is_initialized() {
        list_signal(id).update(|v| v + 1);
    }
}

/// Ask the window to build everything again. Before the runtime is up there is nothing to draw.
pub(super) fn rebuild() {
    if BlincContextState::is_initialized() {
        BlincContextState::get().request_rebuild();
    }
}

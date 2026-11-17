//! The handle of a person picker.
//!
//! Pattern: Facade + Observer, as `share-dialog/handle`: a clonable handle over the [`Picking`] state; every step answers with
//! an [`Outcome`], tells the callback and draws again. The search field keeps its text outside the part that is drawn again.

use std::sync::{Arc, Mutex, PoisonError};

use fs_components_primitives::SearchField;

use super::bindings::{Bindings, Command};
use super::labels::Labels;
use super::model::{Outcome, PersonRef, Refusal, Selection};
use super::source::CandidateSource;
use super::state::Picking;
use crate::keys::KeyEvent;
use crate::live::Live;

/// Called with what every step did.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

struct Inner {
    state: Picking,
    on_outcome: Option<OnOutcome>,
}

/// The shared part: state, callback and the redraw handle.
#[derive(Clone)]
pub(super) struct Core {
    shared: Arc<Mutex<Inner>>,
    pub live: Live,
}

impl Core {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn act(&self, f: impl FnOnce(&mut Picking) -> Outcome) -> Outcome {
        let (outcome, handler) = {
            let mut inner = self.lock();
            (f(&mut inner.state), inner.on_outcome.clone())
        };
        if let (true, Some(handler)) = (outcome != Outcome::Ignored, handler) {
            handler(&outcome);
        }
        self.live.refresh();
        outcome
    }

    pub fn read<T>(&self, f: impl FnOnce(&Picking) -> T) -> T {
        f(&self.lock().state)
    }
}

/// A person picker. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct PersonPicker {
    pub(super) core: Core,
    pub(super) id: String,
    pub(super) bindings: Option<Bindings>,
    pub(super) labels: Labels,
    pub(super) allow_me: bool,
    pub(super) field: SearchField,
}

impl PersonPicker {
    /// A picker over `source`; it asks the source for the first candidates at once.
    pub fn new(source: Arc<dyn CandidateSource>) -> Self {
        let core = Core {
            shared: Arc::new(Mutex::new(Inner {
                state: Picking::new(source, 20),
                on_outcome: None,
            })),
            live: Live::new(),
        };
        let labels = Labels::default();
        Self {
            field: Self::field_for(&core, &labels),
            core,
            id: "fs-pick".into(),
            bindings: None,
            labels,
            allow_me: true,
        }
    }

    fn field_for(core: &Core, labels: &Labels) -> SearchField {
        let core = core.clone();
        SearchField::new(&labels.search).on_change(Arc::new(move |text| {
            core.act(|s| s.set_query(text));
        }))
    }

    /// The element id (rows are `<id>-row-<reference>`, their click part `<id>-pick-<reference>`, and `<id>-me`, `<id>-confirm`, `<id>-cancel`, `<id>-notice`,
    /// `<id>-error`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Take keys while the picker has the keyboard, by these bindings.
    #[must_use]
    pub fn keys(mut self, bindings: Bindings) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.field = Self::field_for(&self.core, &labels);
        self.labels = labels;
        self
    }

    /// Show the "Assign to me" button (on by default; a place that does not assign turns it off).
    #[must_use]
    pub fn allow_me(mut self, allow: bool) -> Self {
        self.allow_me = allow;
        self
    }

    /// At most this many candidates asked of the source per query (1 to 50; 20 by default).
    #[must_use]
    pub fn limit(self, limit: usize) -> Self {
        {
            let mut inner = self.core.lock();
            inner.state.limit = limit.clamp(1, 50);
            inner.state.refresh();
        }
        self
    }

    /// One person or several (several by default).
    #[must_use]
    pub fn selection(self, selection: Selection) -> Self {
        self.core.lock().state.set_selection(selection);
        self
    }

    /// Start with these people chosen.
    #[must_use]
    pub fn choose(self, people: Vec<PersonRef>) -> Self {
        self.core.lock().state.choose(people);
        self
    }

    /// Confirm only with somebody chosen.
    #[must_use]
    pub fn require_choice(self, require: bool) -> Self {
        self.core.lock().state.require_choice(require);
        self
    }

    /// Hear of every step.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.core.lock().on_outcome = Some(handler);
        self
    }

    /// A copy of the references chosen so far.
    pub fn chosen(&self) -> Vec<PersonRef> {
        self.core.read(|s| s.chosen().to_vec())
    }

    /// The reference under the cursor.
    pub fn cursor(&self) -> Option<PersonRef> {
        self.core.read(|s| s.cursor().cloned())
    }

    /// Ask the source again (the program calls this when a pending answer has arrived).
    pub fn refresh(&self) -> Outcome {
        self.core.act(Picking::refresh)
    }

    /// Narrow the list (the search field does this by itself when typed into).
    pub fn set_query(&self, text: &str) -> Outcome {
        self.core.act(|s| s.set_query(text))
    }

    /// Choose or unchoose.
    pub fn toggle(&self, person: &PersonRef) -> Outcome {
        self.core.act(|s| s.toggle(person))
    }

    /// "Assign to me".
    pub fn choose_me(&self) -> Outcome {
        self.core.act(Picking::choose_me)
    }

    /// Done: reports [`Outcome::Confirmed`] to the callback.
    pub fn confirm(&self) -> Outcome {
        self.core.act(|s| s.confirm())
    }

    /// Cancel: reports [`Outcome::Cancelled`].
    pub fn cancel(&self) -> Outcome {
        self.core.act(|_| Outcome::Cancelled)
    }

    /// Do one step.
    pub fn command(&self, command: Command) -> Outcome {
        self.core.act(|s| match command {
            Command::Up => s.move_cursor(-1),
            Command::Down => s.move_cursor(1),
            Command::Toggle => match s.cursor().cloned() {
                Some(p) => s.toggle(&p),
                None => Outcome::Refused(Refusal::Empty),
            },
            Command::Me => s.choose_me(),
            Command::Confirm => s.confirm(),
            _ => Outcome::Cancelled,
        })
    }

    /// A key press: through the bindings if there are any ([`Outcome::Ignored`] otherwise).
    pub fn press(&self, event: &KeyEvent) -> Outcome {
        match self.bindings.as_ref().and_then(|b| b.command(event)) {
            Some(command) => self.command(command),
            None => Outcome::Ignored,
        }
    }
}

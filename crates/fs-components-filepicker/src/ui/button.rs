//! `FilePickButton`: a button that asks a chooser for a path and shows what came back.
//!
//! Pattern: Command (`activate` is what the click, Enter and Space call) + Observer (`Live` draws it again when the answer
//! arrives). States: idle ("Nothing chosen yet"), waiting (button off, "The dialog is open"), chosen (the paths), failed
//! (the problem as a sentence, never silent). A cancel changes nothing. The chooser runs in a worker thread
//! ([`crate::worker::pick_in_worker`]); the answer arrives as an event, `on_picked`. Keyboard: the button is a normal
//! focusable button, the window turns Enter and Space into a click.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use fs_components_widgets::Element;
use fs_components_widgets::live::Live;

use crate::chooser::FileChooser;
use crate::labels::Labels;
use crate::problem::{ChooserError, Problem};
use crate::request::{OpenRequest, Picked};
use crate::worker::pick_in_worker;

/// What the button shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Nothing chosen yet.
    Idle,
    /// The dialog is open.
    Waiting,
    /// These paths were chosen.
    Chosen(Vec<PathBuf>),
    /// The dialog could not be shown or the answer could not be used.
    Failed(Problem),
}

type OnPicked = Arc<dyn Fn(&Picked) + Send + Sync>;

/// The button. Clones share the phase and the callback.
#[derive(Clone)]
pub struct FilePickButton {
    id: String,
    label: String,
    labels: Labels,
    request: OpenRequest,
    chooser: Arc<dyn FileChooser>,
    phase: Arc<Mutex<Phase>>,
    live: Live,
    on_picked: Option<OnPicked>,
}

impl FilePickButton {
    /// A button for `request`; the word on it is the standard one for the mode (`labels`), the answer comes from `chooser`.
    pub fn new(labels: &Labels, chooser: Arc<dyn FileChooser>, request: OpenRequest) -> Self {
        Self {
            id: "file-pick".to_owned(),
            label: labels.button(request.mode).to_owned(),
            labels: labels.clone(),
            request,
            chooser,
            phase: Arc::new(Mutex::new(Phase::Idle)),
            live: Live::new(),
            on_picked: None,
        }
    }

    /// The prefix of the ids of the parts (several buttons in one window): `{id}-button`, `{id}-state`.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Another word on the button (an MF2 text resolved by the caller).
    #[must_use]
    pub fn label(mut self, label: &str) -> Self {
        label.clone_into(&mut self.label);
        self
    }

    /// Call `handler` with the choice (on the worker thread) when the person chose.
    #[must_use]
    pub fn on_picked(mut self, handler: impl Fn(&Picked) + Send + Sync + 'static) -> Self {
        self.on_picked = Some(Arc::new(handler));
        self
    }

    fn lock(&self) -> MutexGuard<'_, Phase> {
        self.phase.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// What the button shows now.
    pub fn phase(&self) -> Phase {
        self.lock().clone()
    }

    /// Start the dialog. Returns at once; false (nothing happens) while a dialog is already open.
    pub fn activate(&self) -> bool {
        let before = {
            let mut phase = self.lock();
            if *phase == Phase::Waiting {
                return false;
            }
            std::mem::replace(&mut *phase, Phase::Waiting)
        };
        self.live.refresh_now();
        let (state, live, on_picked) = (Arc::clone(&self.phase), self.live, self.on_picked.clone());
        pick_in_worker(Arc::clone(&self.chooser), self.request.clone(), move |answer| {
            let next = match answer {
                Ok(picked) => {
                    if let Some(handler) = &on_picked {
                        handler(&picked);
                    }
                    Phase::Chosen(picked.paths().to_vec())
                }
                // a cancel changes nothing: back to what was there
                Err(ChooserError::Cancelled) => before,
                Err(ChooserError::Problem(p)) => Phase::Failed(p),
            };
            *state.lock().unwrap_or_else(PoisonError::into_inner) = next;
            live.refresh_now();
        });
        true
    }

    fn state_line(&self) -> (String, &'static str) {
        match self.phase() {
            Phase::Idle => (self.labels.nothing.clone(), "fs-fp-state"),
            Phase::Waiting => (self.labels.waiting.clone(), "fs-fp-state"),
            Phase::Chosen(paths) => (
                paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                "fs-fp-state",
            ),
            Phase::Failed(p) => (self.labels.problem(&p), "fs-fp-problem"),
        }
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.region("file-pick", move || this.render())
    }

    fn render(&self) -> blinc_layout::div::Div {
        let click = self.clone();
        let waiting = self.phase() == Phase::Waiting;
        let button_id = format!("{}-button", self.id);
        let (line, class) = self.state_line();
        let mut root = div()
            .id(&self.id)
            .class("fs-fp")
            .child(
                div().class("fs-fp-row").child(
                    ButtonBuilder::with_key(button_id.clone(), self.label.clone())
                        .variant(ButtonVariant::Outline)
                        .disabled(waiting)
                        .on_click(move |_| {
                            click.activate();
                        })
                        .build_component()
                        .id(&button_id),
                ),
            )
            .child(
                div()
                    .id(format!("{}-state", self.id))
                    .child(text(line).class(class)),
            );
        if self.chooser.is_provisional() {
            root = root.child(
                div()
                    .id(format!("{}-provisional", self.id))
                    .child(text(self.labels.provisional.clone()).class("fs-fp-provisional")),
            );
        }
        root
    }
}

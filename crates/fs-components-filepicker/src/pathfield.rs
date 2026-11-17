//! The fallback chooser: a typed path, checked, and shown as provisional.
//!
//! Pattern: Strategy (the same trait as the system dialog) + Mediator (the question goes to a [`PathPrompt`] that the Blinc view
//! shows). Used where there is no system dialog (Windows, no portal). Never a permanent solution: the interface marks it
//! with the word `provisional` of the labels, and `is_provisional` is true.

use std::sync::mpsc;

use crate::chooser::FileChooser;
use crate::problem::ChooserError;
use crate::prompt::PathPrompt;
use crate::request::{OpenRequest, Picked};

/// The path field as a chooser. `open` blocks (in a worker thread) until the view has taken a path or cancelled.
#[derive(Clone, Default)]
pub struct PathFieldChooser {
    prompt: PathPrompt,
}

impl PathFieldChooser {
    /// A chooser over `prompt` (the view draws the same prompt).
    pub fn new(prompt: &PathPrompt) -> Self {
        Self {
            prompt: prompt.clone(),
        }
    }

    /// The prompt, for the view.
    pub fn prompt(&self) -> &PathPrompt {
        &self.prompt
    }
}

impl FileChooser for PathFieldChooser {
    fn open(&self, request: OpenRequest) -> Result<Picked, ChooserError> {
        let (reply, answer) = mpsc::channel();
        self.prompt.park(request, reply);
        // a dropped sender (the prompt was replaced and the old one answered, or all clones are gone) is a cancel
        answer.recv().unwrap_or(Err(ChooserError::Cancelled))
    }

    fn is_provisional(&self) -> bool {
        true
    }
}

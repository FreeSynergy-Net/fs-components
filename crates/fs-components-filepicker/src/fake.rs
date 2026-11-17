//! A chooser for tests: answers from a script, remembers what it was asked and on which thread.
//!
//! Pattern: Test double (Fake + Spy). The answers are the only way a test makes a [`Picked`]; an empty script answers with a
//! problem (`io`, `fake: no answer scripted`), never silently with a cancel.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::ThreadId;

use crate::chooser::FileChooser;
use crate::problem::{ChooserError, Problem, ProblemCode};
use crate::request::{OpenRequest, Picked};

/// What the fake answers next.
#[derive(Clone, Debug)]
pub enum Answer {
    /// These paths.
    Paths(Vec<PathBuf>),
    /// A cancel.
    Cancel,
    /// This error.
    Fail(ChooserError),
}

/// A gate: a call of the fake waits at it until the test opens it (shows that the caller does not wait for the dialog).
#[derive(Clone, Default)]
pub struct Gate(Arc<(Mutex<bool>, Condvar)>);

impl Gate {
    /// Let the waiting calls go on (and every later one).
    pub fn open(&self) {
        *self.0.0.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.0.1.notify_all();
    }

    fn wait(&self) {
        let mut open = self.0.0.lock().unwrap_or_else(PoisonError::into_inner);
        while !*open {
            open = self.0.1.wait(open).unwrap_or_else(PoisonError::into_inner);
        }
    }
}

#[derive(Default)]
struct Log {
    script: VecDeque<Answer>,
    requests: Vec<OpenRequest>,
    threads: Vec<ThreadId>,
}

/// The fake. Clones share one script and one log.
#[derive(Clone, Default)]
pub struct FakeChooser {
    log: Arc<Mutex<Log>>,
    gate: Option<Gate>,
    provisional: bool,
}

impl FakeChooser {
    /// A fake with an empty script.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue the next answer.
    #[must_use]
    pub fn answer(self, answer: Answer) -> Self {
        self.lock().script.push_back(answer);
        self
    }

    /// Wait at `gate` before answering.
    #[must_use]
    pub fn gated(mut self, gate: &Gate) -> Self {
        self.gate = Some(gate.clone());
        self
    }

    /// Behave as the provisional stand-in (shows the marker).
    #[must_use]
    pub fn provisional(mut self, provisional: bool) -> Self {
        self.provisional = provisional;
        self
    }

    /// The requests so far.
    pub fn requests(&self) -> Vec<OpenRequest> {
        self.lock().requests.clone()
    }

    /// The threads the calls ran on.
    pub fn threads(&self) -> Vec<ThreadId> {
        self.lock().threads.clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Log> {
        self.log.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl FileChooser for FakeChooser {
    fn open(&self, request: OpenRequest) -> Result<Picked, ChooserError> {
        {
            let mut log = self.lock();
            log.requests.push(request);
            log.threads.push(std::thread::current().id());
        }
        if let Some(gate) = &self.gate {
            gate.wait();
        }
        let next = self.lock().script.pop_front();
        match next {
            Some(Answer::Paths(paths)) => Picked::from_dialog(paths).ok_or_else(|| {
                Problem::new(ProblemCode::Io)
                    .with("detail", "fake: empty list of paths")
                    .into()
            }),
            Some(Answer::Cancel) => Err(ChooserError::Cancelled),
            Some(Answer::Fail(error)) => Err(error),
            None => Err(Problem::new(ProblemCode::Io)
                .with("detail", "fake: no answer scripted")
                .into()),
        }
    }

    fn is_provisional(&self) -> bool {
        self.provisional
    }
}

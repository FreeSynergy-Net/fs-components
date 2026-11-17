//! The state behind the path field: what is asked, the last problem, the way back to the waiting caller.
//!
//! Pattern: Observer + Mediator. A worker thread (the caller of `open`) parks a request here and waits on a channel; the view
//! (Blinc, in the window's thread) shows it, and `submit` or `cancel` answers the worker. The state has no Blinc, so every rule is tested
//! without a window. A wrong path stays pending with its problem shown; nothing is accepted that [`validate_path`] refused.

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::problem::{ChooserError, Problem};
use crate::request::{OpenRequest, Picked};
use crate::validate::validate_path;

type Reply = Sender<Result<Picked, ChooserError>>;
type Notify = Arc<dyn Fn() + Send + Sync>;

struct Pending {
    request: OpenRequest,
    reply: Reply,
}

#[derive(Default)]
struct State {
    pending: Option<Pending>,
    problem: Option<Problem>,
    notify: Option<Notify>,
}

/// The shared state of the path field. Clones share it.
#[derive(Clone, Default)]
pub struct PathPrompt {
    state: Arc<Mutex<State>>,
}

impl PathPrompt {
    /// An empty prompt.
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Call `handler` whenever the state changes (the view draws again). It runs on the thread that changed the state.
    pub fn on_change(&self, handler: impl Fn() + Send + Sync + 'static) {
        self.lock().notify = Some(Arc::new(handler));
    }

    fn changed(&self) {
        let notify = self.lock().notify.clone();
        if let Some(notify) = notify {
            notify();
        }
    }

    /// Park `request`; a request that waited before gets a cancel (one question at a time).
    pub(crate) fn park(&self, request: OpenRequest, reply: Reply) {
        let old = self.lock().pending.replace(Pending { request, reply });
        if let Some(old) = old {
            let _ = old.reply.send(Err(ChooserError::Cancelled));
        }
        self.lock().problem = None;
        self.changed();
    }

    /// The request that waits for an answer.
    pub fn pending(&self) -> Option<OpenRequest> {
        self.lock().pending.as_ref().map(|p| p.request.clone())
    }

    /// The problem of the last typed path, if it was refused.
    pub fn problem(&self) -> Option<Problem> {
        self.lock().problem.clone()
    }

    /// Take the typed path. A refused path leaves the question open with the problem set and returns it.
    ///
    /// # Errors
    /// The problem of [`validate_path`]; also `io` when nothing is asked.
    pub fn submit(&self, text: &str) -> Result<(), Problem> {
        let Some(mode) = self.pending().map(|r| r.mode) else {
            return Err(Problem::new(crate::problem::ProblemCode::Io)
                .with("path", text)
                .with("detail", "no question is open"));
        };
        match validate_path(text, mode) {
            Ok(path) => {
                let taken = self.lock().pending.take();
                if let Some(p) = taken {
                    let picked = Picked::from_dialog(vec![path]).ok_or_else(|| {
                        Problem::new(crate::problem::ProblemCode::PathMissing).with("path", text)
                    })?;
                    let _ = p.reply.send(Ok(picked));
                }
                self.lock().problem = None;
                self.changed();
                Ok(())
            }
            Err(problem) => {
                self.lock().problem = Some(problem.clone());
                self.changed();
                Err(problem)
            }
        }
    }

    /// Leave without a path: the waiting caller gets a cancel.
    pub fn cancel(&self) {
        let taken = self.lock().pending.take();
        if let Some(p) = taken {
            let _ = p.reply.send(Err(ChooserError::Cancelled));
        }
        self.lock().problem = None;
        self.changed();
    }
}

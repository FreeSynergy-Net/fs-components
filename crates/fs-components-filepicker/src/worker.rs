//! The one way to call a chooser from an interface: in a worker thread, the answer by callback.
//!
//! Pattern: Active object (a thread per question; the caller returns at once). `open` of a chooser blocks, and a blocked
//! window thread is a frozen window (the lesson of fs-people): so no part of this crate calls `open` on the caller's thread.

use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;

use crate::chooser::FileChooser;
use crate::problem::{ChooserError, Problem, ProblemCode};
use crate::request::{OpenRequest, Picked};

/// Ask `chooser` in a new thread and hand the answer to `deliver` (on that thread). Returns at once. A panic of the chooser is
/// delivered as a problem (`io`), never lost; when no thread can be started `deliver` gets the problem on the calling thread.
pub fn pick_in_worker<F>(
    chooser: Arc<dyn FileChooser>,
    request: OpenRequest,
    deliver: F,
) -> Option<JoinHandle<()>>
where
    F: FnOnce(Result<Picked, ChooserError>) + Send + 'static,
{
    let slot = Arc::new(Mutex::new(Some(deliver)));
    let inner = Arc::clone(&slot);
    let started = std::thread::Builder::new()
        .name("fs-filepicker".to_owned())
        .spawn(move || {
            let answer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                chooser.open(request)
            }))
            .unwrap_or_else(|_| Err(failure("the chooser panicked")));
            if let Some(deliver) = inner.lock().unwrap_or_else(PoisonError::into_inner).take() {
                deliver(answer);
            }
        });
    match started {
        Ok(handle) => Some(handle),
        Err(e) => {
            let taken = slot.lock().unwrap_or_else(PoisonError::into_inner).take();
            if let Some(deliver) = taken {
                deliver(Err(failure(&format!("no thread: {e}"))));
            }
            None
        }
    }
}

fn failure(detail: &str) -> ChooserError {
    Problem::new(ProblemCode::Io)
        .with("path", "")
        .with("detail", detail)
        .into()
}

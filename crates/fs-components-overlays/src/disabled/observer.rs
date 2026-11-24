//! The hint "first time something unfinished is drawn".
//!
//! Pattern: Observer - one listener per process, told once when the first `Why::Unsupported` control is drawn (ENTWURF-unfertig-tooltip 2.4).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

type Listener = Arc<dyn Fn() + Send + Sync>;

static LISTENER: Mutex<Option<Listener>> = Mutex::new(None);
static TOLD: AtomicBool = AtomicBool::new(false);

/// Called once per process, the first time an `Unsupported` control is drawn. Default: nobody listens.
/// A later call replaces the listener; a listener set after the first draw hears the next draw once.
pub fn on_first_unsupported(f: Arc<dyn Fn() + Send + Sync>) {
    *LISTENER.lock().unwrap_or_else(PoisonError::into_inner) = Some(f);
}

/// An unsupported control was drawn: tell the listener, if there is one, and only once.
pub(super) fn unsupported_drawn() {
    let listener = LISTENER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    if let Some(f) = listener
        && !TOLD.swap(true, Ordering::SeqCst)
    {
        f();
    }
}

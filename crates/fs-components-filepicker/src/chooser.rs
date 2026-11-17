//! The trait every chooser implements.
//!
//! Pattern: Strategy (portal, path field, fake and the fallback chain are interchangeable behind it). `open` BLOCKS until the
//! person has answered: it belongs in a worker thread, never in the window's thread (the lesson of the freeze in fs-people);
//! [`crate::worker::pick_in_worker`] is the way the Blinc parts call it.

use crate::problem::ChooserError;
use crate::request::{OpenRequest, Picked};

/// Something that lets the person choose a path.
pub trait FileChooser: Send + Sync {
    /// Ask the person. Blocks until they chose or cancelled. A cancel is `Err(ChooserError::Cancelled)`, not a problem.
    ///
    /// # Errors
    /// [`ChooserError::Cancelled`] when the person cancelled, [`ChooserError::Problem`] when no dialog could be shown or the
    /// answer could not be used.
    fn open(&self, request: OpenRequest) -> Result<Picked, ChooserError>;

    /// Whether this chooser is a provisional replacement for the system dialog (the path field): the interface marks it.
    fn is_provisional(&self) -> bool {
        false
    }
}

impl<T: FileChooser + ?Sized> FileChooser for std::sync::Arc<T> {
    fn open(&self, request: OpenRequest) -> Result<Picked, ChooserError> {
        (**self).open(request)
    }

    fn is_provisional(&self) -> bool {
        (**self).is_provisional()
    }
}

//! Which chooser to use: the system dialog if there is one, else the marked path field.
//!
//! Pattern: Chain of responsibility (portal first, path field when the portal says "no dialog") + Factory
//! (`default_chooser`). The decision is made twice: before (is a session bus there at all?) and at the call (the portal
//! may still not start in time): both end in the path field and the interface can ask `used_fallback` to show the marker.
//! A refusal by the portal (`portal-denied`) is NOT worked around: it is shown.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::chooser::FileChooser;
use crate::pathfield::PathFieldChooser;
use crate::problem::ChooserError;
use crate::prompt::PathPrompt;
use crate::request::{OpenRequest, Picked};

/// A chooser that tries `primary` and, when that has no dialog to show, asks `fallback`.
pub struct FallbackChooser {
    primary: Arc<dyn FileChooser>,
    fallback: Arc<dyn FileChooser>,
    used_fallback: AtomicBool,
}

impl FallbackChooser {
    /// `primary` first, `fallback` when `primary` answers `portal-unavailable` or `portal-timeout`.
    pub fn new(primary: Arc<dyn FileChooser>, fallback: Arc<dyn FileChooser>) -> Self {
        Self {
            primary,
            fallback,
            used_fallback: AtomicBool::new(false),
        }
    }

    /// Whether a call so far went to the fallback (the interface shows the provisional marker then).
    pub fn used_fallback(&self) -> bool {
        self.used_fallback.load(Ordering::Relaxed)
    }
}

impl FileChooser for FallbackChooser {
    fn open(&self, request: OpenRequest) -> Result<Picked, ChooserError> {
        match self.primary.open(request.clone()) {
            Err(ChooserError::Problem(p)) if p.code.means_no_dialog() => {
                self.used_fallback.store(true, Ordering::Relaxed);
                self.fallback.open(request)
            }
            other => other,
        }
    }

    fn is_provisional(&self) -> bool {
        self.used_fallback() || self.primary.is_provisional()
    }
}

/// Whether a session bus can be there: `DBUS_SESSION_BUS_ADDRESS` is set, or the socket `$XDG_RUNTIME_DIR/bus` exists. This only
/// looks; it never connects (so it never touches the session of a person that is not ours to touch). `env` reads a variable.
pub fn session_bus_present(
    env: &dyn Fn(&str) -> Option<String>,
    exists: &dyn Fn(&Path) -> bool,
) -> bool {
    if env("DBUS_SESSION_BUS_ADDRESS").is_some_and(|v| !v.trim().is_empty()) {
        return true;
    }
    env("XDG_RUNTIME_DIR").is_some_and(|dir| exists(&Path::new(&dir).join("bus")))
}

/// The chooser of this system. With the feature `portal` on Linux and a session bus in sight: the portal, with the path
/// field behind it. Otherwise only the path field (`prompt` is what the view draws).
pub fn default_chooser(prompt: &PathPrompt) -> Arc<dyn FileChooser> {
    let field: Arc<dyn FileChooser> = Arc::new(PathFieldChooser::new(prompt));
    #[cfg(all(feature = "portal", target_os = "linux"))]
    {
        let present = session_bus_present(&|k| std::env::var(k).ok(), &|p| p.exists());
        if present {
            return Arc::new(FallbackChooser::new(
                Arc::new(crate::portal::PortalChooser::new()),
                field,
            ));
        }
    }
    field
}

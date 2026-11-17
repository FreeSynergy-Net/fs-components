//! The system dialog: the `org.freedesktop.portal.FileChooser` of xdg-desktop-portal, over the pinned crate `ashpd`.
//!
//! Pattern: Adapter (ashpd behind [`FileChooser`]) + Timeout (the start of the portal, 5 s). Linux only (target cfg, feature
//! `portal`). `open` blocks the calling thread on a small runtime of its own, so it must run in a worker thread
//! ([`crate::worker`]). The parent window is not passed yet (xdg-foreign handle): the dialog is modal to nothing; a known gap,
//! noted in DESIGN. NOT proven in a headless test: only the translation and the error mapping are; see `tests/portal_live.rs`.

use std::path::PathBuf;
use std::time::Duration;

use ashpd::desktop::ResponseError;
use ashpd::desktop::file_chooser::{FileChooserProxy, FileFilter, SelectedFiles};

use crate::chooser::FileChooser;
use crate::filters::{PortalFilter, to_portal};
use crate::portal_uri::path_of_uri;
use crate::problem::{ChooserError, Problem, ProblemCode};
use crate::request::{Mode, OpenRequest, Picked};

/// How long the portal may take to start (connect to the bus, find the `FileChooser`). The dialog itself has no limit.
pub const START_LIMIT: Duration = Duration::from_secs(5);

/// The chooser over the portal.
#[derive(Clone, Debug)]
pub struct PortalChooser {
    start_limit: Duration,
}

impl Default for PortalChooser {
    fn default() -> Self {
        Self {
            start_limit: START_LIMIT,
        }
    }
}

impl PortalChooser {
    /// A chooser with the standard start limit.
    pub fn new() -> Self {
        Self::default()
    }

    /// Another start limit (tests).
    #[must_use]
    pub fn start_limit(mut self, limit: Duration) -> Self {
        self.start_limit = limit;
        self
    }
}

/// How a D-Bus error reads for the fallback: the dialog is not there, the portal refused, or something else went wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reading {
    /// The service is missing, the name has no owner, the bus is not reachable: the path field may take over.
    Missing,
    /// A refusal (`AccessDenied`, a failed authentication): shown, never worked around.
    Denied,
    /// Anything else: shown as an error with its text, never worked around.
    Other,
}

fn read_dbus_name(name: &str) -> Reading {
    match name.strip_prefix("org.freedesktop.DBus.Error.") {
        Some(
            "ServiceUnknown" | "NameHasNoOwner" | "NoReply" | "Timeout" | "TimedOut" | "NoServer"
            | "Disconnected" | "BadAddress" | "UnknownMethod" | "UnknownObject"
            | "UnknownInterface",
        ) => Reading::Missing,
        Some("AccessDenied" | "AuthFailed") => Reading::Denied,
        _ => Reading::Other,
    }
}

fn read_fdo(error: &ashpd::zbus::fdo::Error) -> Reading {
    use ashpd::zbus::fdo::Error as F;
    match error {
        F::ServiceUnknown(_)
        | F::NameHasNoOwner(_)
        | F::NoReply(_)
        | F::Timeout(_)
        | F::NoServer(_)
        | F::Disconnected(_)
        | F::BadAddress(_)
        | F::UnknownMethod(_)
        | F::UnknownObject(_)
        | F::UnknownInterface(_) => Reading::Missing,
        F::AccessDenied(_) | F::AuthFailed(_) => Reading::Denied,
        F::ZBus(e) => read_zbus(e),
        _ => Reading::Other,
    }
}

fn read_zbus(error: &ashpd::zbus::Error) -> Reading {
    use ashpd::zbus::Error as Z;
    match error {
        Z::FDO(e) => read_fdo(e),
        Z::MethodError(name, _, _) => read_dbus_name(name.as_str()),
        // no bus to talk to: the address is bad, the socket is not there, the handshake did not come about
        Z::Address(_)
        | Z::Connection(..)
        | Z::InputOutput(_)
        | Z::Handshake(_)
        | Z::InterfaceNotFound => Reading::Missing,
        _ => Reading::Other,
    }
}

fn from_reading(reading: Reading, detail: String) -> ChooserError {
    match reading {
        Reading::Missing => ChooserError::problem(ProblemCode::PortalUnavailable),
        Reading::Denied => Problem::new(ProblemCode::PortalDenied)
            .with("detail", detail)
            .into(),
        Reading::Other => Problem::new(ProblemCode::PortalFailed)
            .with("detail", detail)
            .into(),
    }
}

/// An ashpd error as our answer. A cancel is [`ChooserError::Cancelled`]. Only "there is no dialog" falls back to the path
/// field: no bus, no service, a name without an owner, no portal frontend, an old interface, no answer (`portal-unavailable`).
/// A refusal (portal `NotAllowed`, a D-Bus `AccessDenied`) is `portal-denied`, a failed file operation is `io`, and every other
/// error is `portal-failed`: all three are shown with their text and are never a reason for the path field.
pub fn map_error(error: &ashpd::Error) -> ChooserError {
    use ashpd::{Error as E, PortalError as P};
    let detail = error.to_string();
    match error {
        E::Response(ResponseError::Cancelled) | E::Portal(P::Cancelled(_)) => {
            ChooserError::Cancelled
        }
        E::Response(_) | E::Portal(P::NotAllowed(_) | P::InvalidArgument(_)) => {
            from_reading(Reading::Denied, detail)
        }
        E::IO(_) | E::NulTerminated(_) | E::Uri(_) => Problem::new(ProblemCode::Io)
            .with("path", "")
            .with("detail", detail)
            .into(),
        E::Zbus(e) | E::Portal(P::ZBus(e)) => from_reading(read_zbus(e), detail),
        // no portal frontend, no answer, an old interface
        E::NoResponse | E::PortalNotFound(_) | E::RequiresVersion(..) => {
            from_reading(Reading::Missing, detail)
        }
        // everything else (a portal that ran and failed, a broken answer, not found, exists, window destroyed, bad app id, ...)
        // is an error to show
        _ => from_reading(Reading::Other, detail),
    }
}

fn file_filter(filter: &PortalFilter) -> FileFilter {
    let mut out = FileFilter::new(&filter.label);
    for glob in &filter.globs {
        out = out.glob(glob);
    }
    for media in &filter.media_types {
        out = out.mimetype(media);
    }
    out
}

/// The files of the answer as paths; a URI that is not a local `file:` one is a problem (`io`), not skipped.
fn paths_of(files: &SelectedFiles) -> Result<Vec<PathBuf>, ChooserError> {
    files
        .uris()
        .iter()
        .map(|uri| {
            path_of_uri(uri.as_str()).ok_or_else(|| {
                Problem::new(ProblemCode::Io)
                    .with("path", uri.as_str())
                    .with("detail", "not a local file URI")
                    .into()
            })
        })
        .collect()
}

async fn ask(request: OpenRequest) -> Result<SelectedFiles, ashpd::Error> {
    let filters: Vec<FileFilter> = to_portal(&request.filters)
        .iter()
        .map(file_filter)
        .collect();
    let dir = request.initial_dir.as_deref();
    let sent = match request.mode {
        Mode::SaveFile => {
            SelectedFiles::save_file()
                .title(request.title.as_str())
                .modal(true)
                .filters(filters)
                .current_name(request.suggested_name.as_deref())
                .current_folder::<&std::path::Path>(dir)?
                .send()
                .await?
        }
        mode => {
            SelectedFiles::open_file()
                .title(request.title.as_str())
                .modal(true)
                .directory(mode == Mode::Folder)
                .multiple(mode == Mode::OpenFiles)
                .filters(if mode == Mode::Folder {
                    Vec::new()
                } else {
                    filters
                })
                .current_folder::<&std::path::Path>(dir)?
                .send()
                .await?
        }
    };
    sent.response()
}

impl FileChooser for PortalChooser {
    fn open(&self, request: OpenRequest) -> Result<Picked, ChooserError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                ChooserError::from(
                    Problem::new(ProblemCode::Io)
                        .with("path", "")
                        .with("detail", e.to_string()),
                )
            })?;
        let limit = self.start_limit;
        runtime.block_on(async move {
            // the start: connect and find the interface, within the limit; the dialog itself has none
            match tokio::time::timeout(limit, FileChooserProxy::new()).await {
                Err(_) => {
                    return Err(Problem::new(ProblemCode::PortalTimeout)
                        .with("limit-ms", limit.as_millis().to_string())
                        .into());
                }
                Ok(Err(e)) => return Err(map_error(&e)),
                Ok(Ok(_proxy)) => {}
            }
            let files = ask(request).await.map_err(|e| map_error(&e))?;
            let paths = paths_of(&files)?;
            // a dialog that returns no file is a cancel
            Picked::from_dialog(paths).ok_or(ChooserError::Cancelled)
        })
    }
}

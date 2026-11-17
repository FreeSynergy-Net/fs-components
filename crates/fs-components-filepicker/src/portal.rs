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

/// An ashpd error as our answer. A cancel is [`ChooserError::Cancelled`]; no bus or no portal is `portal-unavailable`; a refusal
/// is `portal-denied`; a failed file operation is `io`.
pub fn map_error(error: &ashpd::Error) -> ChooserError {
    use ashpd::{Error as E, PortalError as P};
    let detail = error.to_string();
    match error {
        E::Response(ResponseError::Cancelled) | E::Portal(P::Cancelled(_)) => {
            ChooserError::Cancelled
        }
        E::Response(_) | E::Portal(P::NotAllowed(_) | P::InvalidArgument(_)) => {
            Problem::new(ProblemCode::PortalDenied)
                .with("detail", detail)
                .into()
        }
        E::IO(_) | E::NulTerminated(_) | E::Uri(_) => Problem::new(ProblemCode::Io)
            .with("path", "")
            .with("detail", detail)
            .into(),
        // no bus, no portal frontend, no answer, an old interface and everything we do not know: there is no dialog
        _ => ChooserError::problem(ProblemCode::PortalUnavailable),
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

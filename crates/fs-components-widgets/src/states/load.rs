//! `Load` and `Surface`: what a list, a grid or a page shows for each of its four states besides "ready".
//!
//! Pattern: State (the enum) + Strategy (the caller's `ready` closure draws the content). One place decides which surface goes
//! with which state, so every program shows empty, loading, failed and partial the same way:
//! - `Loading`: the [`Skeleton`] (the content keeps its place, nothing jumps when it arrives);
//! - `Empty`: the [`EmptyState`] the caller configured (words and action are the caller's);
//! - `Failed(reason)`: an [`Alert`] with the reason and "Try again" (the retry is the caller's);
//! - `Partial`: the content AND a [`PartialNotice`] above it (what is there is shown and usable);
//! - `Ready`: the content.

use std::sync::Arc;

use blinc_layout::div::{Div, div};

use super::alert::Alert;
use super::empty::EmptyState;
use super::labels::Labels;
use super::partial::PartialNotice;
use super::skeleton::Skeleton;

/// The state of something that is fetched.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Load {
    /// On its way.
    Loading,
    /// Fetched, and there is nothing.
    Empty,
    /// Could not be fetched; the reason in the person's words.
    Failed(String),
    /// Some of it arrived; `missing` names what did not, out of `total`.
    Partial {
        /// The names of what could not be shown.
        missing: Vec<String>,
        /// How many there should have been.
        total: usize,
    },
    /// All of it is there.
    Ready,
}

/// The surfaces for one place (a list, a grid): its empty surface, its retry and its placeholder.
#[derive(Clone)]
pub struct Surface {
    id: String,
    labels: Labels,
    empty: EmptyState,
    retry: Option<Arc<dyn Fn() + Send + Sync>>,
    rows: usize,
}

impl Surface {
    /// Surfaces with `empty` as the empty one and three placeholder rows.
    pub fn new(labels: &Labels, empty: EmptyState) -> Self {
        Self {
            id: "state".to_owned(),
            labels: labels.clone(),
            empty,
            retry: None,
            rows: 3,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What "Try again" does (failed and partial).
    #[must_use]
    pub fn on_retry(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.retry = Some(Arc::new(handler));
        self
    }

    /// How many placeholder rows the loading surface shows.
    #[must_use]
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// The element for `load`; `ready` draws the content for `Ready` and `Partial` (called only then).
    pub fn build(&self, load: &Load, ready: impl FnOnce() -> Div) -> Div {
        let id = &self.id;
        match load {
            Load::Loading => div().id(id).child(
                Skeleton::new(&self.labels.loading)
                    .id(&format!("{id}-skeleton"))
                    .rows(self.rows)
                    .render(),
            ),
            Load::Empty => div()
                .id(id)
                .child(self.empty.clone().id(&format!("{id}-empty")).render()),
            Load::Failed(reason) => {
                let retry = self.retry.clone();
                let alert = Alert::new(super::Severity::Error, reason)
                    .id(&format!("{id}-failed"))
                    .title(&self.labels.failed);
                let alert = match retry {
                    Some(act) => alert.action(&self.labels.retry, move || act()),
                    None => alert,
                };
                div().id(id).child(alert.render())
            }
            Load::Partial { missing, total } => {
                let mut notice =
                    PartialNotice::new(&self.labels, missing, *total).id(&format!("{id}-partial"));
                if let Some(act) = self.retry.clone() {
                    notice = notice.retry(&self.labels, move || act());
                }
                div().id(id).child(notice.render()).child(ready())
            }
            Load::Ready => div().id(id).child(ready()),
        }
    }
}

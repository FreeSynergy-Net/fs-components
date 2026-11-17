//! A source for tests and for programs that have no directory yet: typed, scripted, in memory.
//!
//! Pattern: Test double (Fake). It filters its list by case-insensitive text in the released name and detail and keeps
//! the rank of the list; what it releases is what it was given (a person without a name stays unnamed).

use std::sync::{Arc, Mutex, PoisonError};

use super::model::{Candidate, CandidatePage, ErrorCode, PersonRef, PickerError};
use super::source::CandidateSource;

/// How the fake answers.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Script {
    /// Answer from the list.
    Answer,
    /// Answer "not there yet" until [`FakeSource::set_script`] changes it.
    Pending,
    /// Fail with this error on every call.
    Fail(PickerError),
}

struct Inner {
    people: Vec<Candidate>,
    me: Result<Option<PersonRef>, PickerError>,
    script: Script,
    asked: Vec<(String, usize)>,
}

/// An in-memory [`CandidateSource`]. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct FakeSource {
    inner: Arc<Mutex<Inner>>,
}

impl FakeSource {
    /// A source offering `people`, without an own card.
    pub fn new(people: Vec<Candidate>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                people,
                me: Ok(None),
                script: Script::Answer,
                asked: Vec::new(),
            })),
        }
    }

    /// A source that has no directory: every call fails with [`ErrorCode::SourceMissing`].
    pub fn missing(what: &str) -> Self {
        let fake = Self::new(Vec::new());
        fake.set_script(Script::Fail(PickerError::new(
            ErrorCode::SourceMissing,
            what,
        )));
        fake
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The own card the asker has.
    #[must_use]
    pub fn with_me(self, me: PersonRef) -> Self {
        self.lock().me = Ok(Some(me));
        self
    }

    /// `me()` fails with this.
    #[must_use]
    pub fn with_me_error(self, error: PickerError) -> Self {
        self.lock().me = Err(error);
        self
    }

    /// Change how it answers.
    pub fn set_script(&self, script: Script) {
        self.lock().script = script;
    }

    /// The queries and limits asked so far.
    pub fn asked(&self) -> Vec<(String, usize)> {
        self.lock().asked.clone()
    }
}

impl CandidateSource for FakeSource {
    fn search(&self, query: &str, limit: usize) -> Result<Option<CandidatePage>, PickerError> {
        let mut inner = self.lock();
        inner.asked.push((query.to_string(), limit));
        match &inner.script {
            Script::Pending => return Ok(None),
            Script::Fail(e) => return Err(e.clone()),
            Script::Answer => {}
        }
        let needle = query.trim().to_lowercase();
        let hits: Vec<Candidate> = inner
            .people
            .iter()
            .filter(|c| {
                needle.is_empty()
                    || [c.display.as_deref(), c.detail.as_deref()]
                        .into_iter()
                        .flatten()
                        .any(|t| t.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect();
        let more = hits.len() > limit;
        Ok(Some(CandidatePage {
            items: hits.into_iter().take(limit).collect(),
            more,
        }))
    }

    fn me(&self) -> Result<Option<PersonRef>, PickerError> {
        self.lock().me.clone()
    }

    fn display_of(&self, person: &PersonRef) -> Option<String> {
        self.lock()
            .people
            .iter()
            .find(|c| &c.reference == person)
            .and_then(|c| c.display.clone())
    }
}

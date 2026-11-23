//! The facade [`Notifications`]: the one place every message, failure and question of a program goes through (ADR 0106 point 2).
//!
//! Pattern: Facade over Port ([`Outlet`]) + Observer with a lifetime (ADR 0065 point 4): each notification lives from `tell` to its
//! close (expiry, answer, `close`, end of its scope), every observer lives as long as its [`Subscription`]. Time is handed in
//! (`now_ms`), so nothing here reads a clock and every test is exact.
//!
//! Safety rule (ADR 0106 point 8): an answer or a close that comes from OUTSIDE ([`Notifications::answered`],
//! [`Notifications::dismissed`]) counts only for an id this crate issued and an action the notification has. Anything else is
//! dropped, logged ([`Notifications::rejected`]) and never run.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use fs_types::coded::Coded;

use crate::codes::{NotificationCode, keys};
use crate::events::{Event, Observer, Open};
use crate::message::{ActionId, NoteId};
use crate::notification::{CloseReason, Notification};
use crate::outlet::{Form, Outlet, Situation};
use crate::policy::Policy;

/// How many dropped answers are kept for [`Notifications::rejected`].
pub const REJECTED_KEPT: usize = 100;

/// The lifetime of an observer: dropping it ends the observation.
#[must_use = "dropping the subscription ends the observation"]
pub struct Subscription {
    inner: Weak<Inner>,
    token: u64,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            lock(&inner.state)
                .observers
                .retain(|(t, _)| *t != self.token);
        }
    }
}

pub(crate) struct State {
    pub(crate) policy: Policy,
    pub(crate) situation: Situation,
    pub(crate) next_id: u64,
    pub(crate) next_token: u64,
    pub(crate) open: Vec<Open>,
    pub(crate) observers: Vec<(u64, Arc<dyn Observer>)>,
    pub(crate) rejected: Vec<Coded<NotificationCode>>,
}

pub(crate) struct Inner {
    pub(crate) outlets: Vec<Arc<dyn Outlet>>,
    pub(crate) state: Mutex<State>,
}

pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The facade. Cheap to clone: every clone is the same set of notifications.
#[derive(Clone)]
pub struct Notifications {
    pub(crate) inner: Arc<Inner>,
}

impl Notifications {
    /// With the rule and the outlets (a window frontend, a system adapter ...).
    #[must_use]
    pub fn new(policy: Policy, outlets: Vec<Arc<dyn Outlet>>) -> Self {
        Self {
            inner: Arc::new(Inner {
                outlets,
                state: Mutex::new(State {
                    policy,
                    situation: Situation::default(),
                    next_id: 1,
                    next_token: 1,
                    open: Vec::new(),
                    observers: Vec::new(),
                    rejected: Vec::new(),
                }),
            }),
        }
    }

    /// Takes a new rule (a setting changed; see [`Policy::from_settings`]). Open notifications keep their form.
    pub fn set_policy(&self, policy: Policy) {
        lock(&self.inner.state).policy = policy;
    }

    /// The host tells where the window is and whether it is calm.
    pub fn set_situation(&self, situation: Situation) {
        lock(&self.inner.state).situation = situation;
    }

    /// Listens until the returned [`Subscription`] is dropped.
    pub fn subscribe(&self, observer: Arc<dyn Observer>) -> Subscription {
        let mut st = lock(&self.inner.state);
        let token = st.next_token;
        st.next_token += 1;
        st.observers.push((token, observer));
        Subscription {
            inner: Arc::downgrade(&self.inner),
            token,
        }
    }

    /// A message.
    ///
    /// # Errors
    /// `wrong-kind` for a question, `no-form` / `no-outlet` / `show-failed` when it could be shown nowhere.
    pub fn tell(
        &self,
        notification: Notification,
        now_ms: u64,
    ) -> Result<NoteId, Coded<NotificationCode>> {
        if notification.is_question() {
            return Err(wrong_kind("is-a-question"));
        }
        self.show(notification, now_ms)
    }

    /// A question: the answer comes as [`Event::Answered`].
    ///
    /// # Errors
    /// `wrong-kind` for a message, else as [`Notifications::tell`].
    pub fn ask(
        &self,
        notification: Notification,
        now_ms: u64,
    ) -> Result<NoteId, Coded<NotificationCode>> {
        if !notification.is_question() {
            return Err(wrong_kind("not-a-question"));
        }
        self.show(notification, now_ms)
    }

    /// The program closes it (the cause is gone, the failure is fixed).
    ///
    /// # Errors
    /// `unknown-id` when it is not open.
    pub fn close(&self, id: &NoteId) -> Result<(), Coded<NotificationCode>> {
        self.end(id, CloseReason::Closed)
    }

    /// The person closed it in an outlet. An id that is not open is dropped and logged.
    ///
    /// # Errors
    /// `unknown-id`.
    pub fn dismissed(&self, id: &NoteId) -> Result<(), Coded<NotificationCode>> {
        match self.end(id, CloseReason::Dismissed) {
            Err(e) => Err(self.reject(e)),
            ok => ok,
        }
    }

    /// The person chose an action in an outlet. Counts only for an open id and an action it has; else it is dropped, logged, and
    /// nothing runs. After the answer the notification is closed (`dismissed`).
    ///
    /// # Errors
    /// `unknown-id` or `unknown-action` (both also in [`Notifications::rejected`]).
    pub fn answered(&self, id: &NoteId, action: &ActionId) -> Result<(), Coded<NotificationCode>> {
        // checking and taking it out of `open` is ONE step under the lock: a second answer, a close or a tick finds nothing
        let taken = {
            let mut st = lock(&self.inner.state);
            match st.open.iter().position(|o| &o.id == id) {
                None => Err(unknown_id(id)),
                Some(at)
                    if !st.open[at]
                        .notification
                        .actions
                        .iter()
                        .any(|a| &a.id == action) =>
                {
                    Err(Coded::new(NotificationCode::UnknownAction)
                        .with_text(keys::ID, id.as_str())
                        .with_text(keys::ACTION, action.as_str()))
                }
                Some(at) => Ok(st.open.remove(at)),
            }
        };
        let gone = taken.map_err(|e| self.reject(e))?;
        self.close_forms(&gone.forms, id);
        self.notify(&[
            Event::Answered {
                id: id.clone(),
                action: action.clone(),
            },
            Event::Closed {
                id: id.clone(),
                reason: CloseReason::Dismissed,
            },
        ]);
        Ok(())
    }

    /// Closes what ran out of time. Call it on the beat of the shell (`fs.shell.refresh-ms`). Returns the ids it closed.
    #[allow(clippy::must_use_candidate)] // the host may ignore the ids; the closing is the point
    pub fn tick(&self, now_ms: u64) -> Vec<NoteId> {
        let due: Vec<NoteId> = lock(&self.inner.state)
            .open
            .iter()
            .filter(|o| o.deadline_ms.is_some_and(|d| d <= now_ms))
            .map(|o| o.id.clone())
            .collect();
        for id in &due {
            let _ = self.end(id, CloseReason::Expired);
        }
        due
    }

    /// The person left an occasion (a view): everything that belongs to it closes (ADR 0065 point 4).
    #[allow(clippy::must_use_candidate)] // the caller may ignore the ids; the closing is the point
    pub fn leave_scope(&self, scope: &crate::message::Scope) -> Vec<NoteId> {
        let gone: Vec<NoteId> = lock(&self.inner.state)
            .open
            .iter()
            .filter(|o| o.notification.scope.as_ref() == Some(scope))
            .map(|o| o.id.clone())
            .collect();
        for id in &gone {
            let _ = self.end(id, CloseReason::Closed);
        }
        gone
    }

    /// What is open now, in the order it was shown.
    #[must_use]
    pub fn open(&self) -> Vec<Open> {
        lock(&self.inner.state).open.clone()
    }

    /// The answers and closes that were dropped (the newest [`REJECTED_KEPT`]): the log of ADR 0106 point 8.
    #[must_use]
    pub fn rejected(&self) -> Vec<Coded<NotificationCode>> {
        lock(&self.inner.state).rejected.clone()
    }

    fn reject(&self, code: Coded<NotificationCode>) -> Coded<NotificationCode> {
        let mut st = lock(&self.inner.state);
        st.rejected.push(code.clone());
        let over = st.rejected.len().saturating_sub(REJECTED_KEPT);
        st.rejected.drain(..over);
        code
    }

    pub(crate) fn notify(&self, events: &[Event]) {
        let observers: Vec<Arc<dyn Observer>> = lock(&self.inner.state)
            .observers
            .iter()
            .map(|(_, o)| Arc::clone(o))
            .collect();
        for e in events {
            for o in &observers {
                o.on_event(e);
            }
        }
    }

    fn end(&self, id: &NoteId, reason: CloseReason) -> Result<(), Coded<NotificationCode>> {
        let gone = {
            let mut st = lock(&self.inner.state);
            let at = st
                .open
                .iter()
                .position(|o| &o.id == id)
                .ok_or_else(|| unknown_id(id))?;
            st.open.remove(at)
        };
        self.close_forms(&gone.forms, id);
        self.notify(&[Event::Closed {
            id: id.clone(),
            reason,
        }]);
        Ok(())
    }

    /// Tells every outlet of these forms to take the notification away (outside every lock).
    pub(crate) fn close_forms(&self, forms: &[Form], id: &NoteId) {
        for form in forms {
            for outlet in self
                .inner
                .outlets
                .iter()
                .filter(|o| o.forms().contains(form))
            {
                outlet.close(*form, id);
            }
        }
    }
}

pub(crate) fn unknown_id(id: &NoteId) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::UnknownId).with_text(keys::ID, id.as_str())
}

fn wrong_kind(cause: &str) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::WrongKind).with_text(keys::CAUSE, cause)
}

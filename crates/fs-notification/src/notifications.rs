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
use crate::message::{ActionId, NoteId};
use crate::notification::{CloseReason, Notification};
use crate::outlet::{Form, Outlet, Situation};
use crate::policy::Policy;

/// How many dropped answers are kept for [`Notifications::rejected`].
pub const REJECTED_KEPT: usize = 100;

/// What happened to a notification; observers get these.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Shown (or replaced, with the same id) in these forms.
    Shown {
        /// Its id.
        id: NoteId,
        /// The forms it was shown in.
        forms: Vec<Form>,
    },
    /// The person chose an action.
    Answered {
        /// Its id.
        id: NoteId,
        /// The action.
        action: ActionId,
    },
    /// Gone.
    Closed {
        /// Its id.
        id: NoteId,
        /// Why.
        reason: CloseReason,
    },
}

/// Somebody who wants to hear what happens.
pub trait Observer: Send + Sync {
    /// Called after the change, outside every lock: an observer may call the facade again.
    fn on_event(&self, event: &Event);
}

impl<F: Fn(&Event) + Send + Sync> Observer for F {
    fn on_event(&self, event: &Event) {
        self(event);
    }
}

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

/// An open notification, as the facade keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Open {
    /// Its id.
    pub id: NoteId,
    /// What was shown.
    pub notification: Notification,
    /// The forms it was shown in.
    pub forms: Vec<Form>,
    /// When it expires (ms of the clock handed in), or `None` for until it is closed.
    pub deadline_ms: Option<u64>,
}

struct State {
    policy: Policy,
    situation: Situation,
    next_id: u64,
    next_token: u64,
    open: Vec<Open>,
    observers: Vec<(u64, Arc<dyn Observer>)>,
    rejected: Vec<Coded<NotificationCode>>,
}

struct Inner {
    outlets: Vec<Arc<dyn Outlet>>,
    state: Mutex<State>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The facade. Cheap to clone: every clone is the same set of notifications.
#[derive(Clone)]
pub struct Notifications {
    inner: Arc<Inner>,
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
        let known = {
            let st = lock(&self.inner.state);
            st.open
                .iter()
                .find(|o| &o.id == id)
                .map(|o| o.notification.actions.iter().any(|a| &a.id == action))
        };
        match known {
            None => Err(self.reject(unknown_id(id))),
            Some(false) => Err(self.reject(
                Coded::new(NotificationCode::UnknownAction)
                    .with_text(keys::ID, id.as_str())
                    .with_text(keys::ACTION, action.as_str()),
            )),
            Some(true) => {
                self.notify(&[Event::Answered {
                    id: id.clone(),
                    action: action.clone(),
                }]);
                self.end(id, CloseReason::Dismissed)
            }
        }
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

    fn notify(&self, events: &[Event]) {
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
        for form in &gone.forms {
            for outlet in self
                .inner
                .outlets
                .iter()
                .filter(|o| o.forms().contains(form))
            {
                outlet.close(*form, id);
            }
        }
        self.notify(&[Event::Closed {
            id: id.clone(),
            reason,
        }]);
        Ok(())
    }

    fn show(
        &self,
        notification: Notification,
        now_ms: u64,
    ) -> Result<NoteId, Coded<NotificationCode>> {
        let (chosen, id, lifetime) = {
            let mut st = lock(&self.inner.state);
            let chosen = st.policy.choose(&notification, &st.situation);
            // only an id that is open here may be replaced; an id from outside that was never issued is ignored
            let id = match &notification.replaces {
                Some(r) if st.open.iter().any(|o| &o.id == r) => r.clone(),
                _ => {
                    let id = NoteId::issued(st.next_id);
                    st.next_id += 1;
                    id
                }
            };
            let lifetime = st.policy.lifetime_ms(&notification);
            (chosen, id, lifetime)
        };
        if chosen.is_empty() {
            return Err(Coded::new(NotificationCode::NoForm));
        }
        let shown = self.deliver(&chosen, &id, &notification)?;
        let open = Open {
            id: id.clone(),
            notification,
            forms: shown.clone(),
            deadline_ms: lifetime.map(|ms| now_ms.saturating_add(ms)),
        };
        {
            let mut st = lock(&self.inner.state);
            match st.open.iter().position(|o| o.id == id) {
                Some(at) => st.open[at] = open,
                None => st.open.push(open),
            }
        }
        self.notify(&[Event::Shown {
            id: id.clone(),
            forms: shown,
        }]);
        Ok(id)
    }

    /// Hands it to the outlets, form by form. A modal that no outlet can show falls back to a toast. Returns the forms that took it.
    fn deliver(
        &self,
        chosen: &[Form],
        id: &NoteId,
        notification: &Notification,
    ) -> Result<Vec<Form>, Coded<NotificationCode>> {
        let mut shown = Vec::new();
        let mut failure = None;
        let mut want: Vec<Form> = chosen.to_vec();
        if want.contains(&Form::Modal)
            && !self.has_form(Form::Modal)
            && !want.contains(&Form::Toast)
        {
            want.insert(0, Form::Toast);
        }
        for form in want {
            if form == Form::Modal && !self.has_form(Form::Modal) {
                continue;
            }
            let view = if form == Form::System {
                notification.redacted()
            } else {
                notification.clone()
            };
            let mut took = false;
            for outlet in self
                .inner
                .outlets
                .iter()
                .filter(|o| o.forms().contains(&form))
            {
                match outlet.show(form, id, &view) {
                    Ok(()) => took = true,
                    Err(e) => failure = Some(e),
                }
            }
            if took {
                shown.push(form);
            }
        }
        if shown.is_empty() {
            return Err(failure.unwrap_or_else(|| {
                let names: Vec<&str> = chosen.iter().map(|f| f.as_str()).collect();
                Coded::new(NotificationCode::NoOutlet).with_text(keys::FORMS, names.join("+"))
            }));
        }
        Ok(shown)
    }

    fn has_form(&self, form: Form) -> bool {
        self.inner.outlets.iter().any(|o| o.forms().contains(&form))
    }
}

fn unknown_id(id: &NoteId) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::UnknownId).with_text(keys::ID, id.as_str())
}

fn wrong_kind(cause: &str) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::WrongKind).with_text(keys::CAUSE, cause)
}

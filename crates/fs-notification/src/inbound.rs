//! What comes from OUTSIDE: the answer and the close of a person in an outlet, and the log of what was dropped (ADR 0106 point 8).
//!
//! Pattern: Guard. Checking an id and an action and taking the notification out of `open` is ONE step under the lock; only then
//! the outlets are told and the observers hear it, so a second answer, a `close` or a `tick` finds nothing.

use fs_types::coded::Coded;

use crate::codes::{NotificationCode, keys};
use crate::events::{Event, Outcome};
use crate::message::{ActionId, NoteId};
use crate::notification::CloseReason;
use crate::notifications::{Notifications, REJECTED_KEPT, lock, unknown_id};

impl Notifications {
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

    /// The person chose an action in an outlet, without the box "remember": the same as `answered_with(id, action, false)`.
    ///
    /// # Errors
    /// `unknown-id` or `unknown-action` (both also in [`Notifications::rejected`]).
    pub fn answered(&self, id: &NoteId, action: &ActionId) -> Result<(), Coded<NotificationCode>> {
        self.answered_with(id, action, false)
    }

    /// The person chose an action in an outlet. Counts only for an open id and an action it has; else it is dropped, logged, and
    /// nothing runs. After the answer the notification is closed (`dismissed`). With `remember` the answer is also written as the
    /// value of the setting of the question ([`crate::Remember`]), but only when its action is a word of the list (`cancel` never).
    ///
    /// # Errors
    /// `unknown-id` or `unknown-action` (both also in [`Notifications::rejected`]); then nothing counted. `remember-invalid`
    /// (`cause` = `not-offered`: the question offered no box; a mistake of the caller, no line) or `remember-failed` (`cause` = the
    /// code of `fs-settings`, e.g. `locked`; a visible warning line says so): the answer counted once all the same, nothing was
    /// written.
    pub fn answered_with(
        &self,
        id: &NoteId,
        action: &ActionId,
        remember: bool,
    ) -> Result<(), Coded<NotificationCode>> {
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
                Some(at) => {
                    let gone = st.open.remove(at);
                    st.record(
                        id,
                        Outcome {
                            reason: CloseReason::Dismissed,
                            answer: Some(action.clone()),
                        },
                    );
                    Ok(gone)
                }
            }
        };
        let gone = taken.map_err(|e| self.reject(e))?;
        self.close_forms(&gone.forms, id);
        // writing happens outside every lock, after the notification is out of `open`: a second answer finds nothing
        let written = remember.then(|| self.remember_answer(&gone, action));
        self.notify(&[Event::Answered {
            id: id.clone(),
            action: action.clone(),
        }]);
        if let Some(w) = &written {
            self.tell_remembered(id, w);
        }
        self.notify(&[Event::Closed {
            id: id.clone(),
            reason: CloseReason::Dismissed,
        }]);
        written.map_or(Ok(()), |w| w.map(|_| ()))
    }

    /// How a notification ended, by its id: the reason and, for a question, the action the person chose (the newest
    /// [`ENDED_KEPT`] are kept). `None` while it is open and for an id that never was one. The same fact comes as
    /// [`Event::Answered`] and [`Event::Closed`] to whoever subscribed; nothing here blocks and waits.
    #[must_use]
    pub fn outcome(&self, id: &NoteId) -> Option<Outcome> {
        lock(&self.inner.state)
            .ended
            .iter()
            .rev()
            .find(|(i, _)| i == id)
            .map(|(_, o)| o.clone())
    }

    /// The answers and closes that were dropped (the newest [`REJECTED_KEPT`]): the log of ADR 0106 point 8.
    #[must_use]
    pub fn rejected(&self) -> Vec<Coded<NotificationCode>> {
        lock(&self.inner.state).rejected.clone()
    }

    pub(crate) fn reject(&self, code: Coded<NotificationCode>) -> Coded<NotificationCode> {
        let mut st = lock(&self.inner.state);
        st.rejected.push(code.clone());
        let over = st.rejected.len().saturating_sub(REJECTED_KEPT);
        st.rejected.drain(..over);
        code
    }
}

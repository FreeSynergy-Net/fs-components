//! Remembering at the facade: the host hands over the way to write ([`Notifications::with_remembers`]), a question that carries a
//! [`crate::Remember`] is checked when it is asked, and an answer with the box set ([`Notifications::answered_with`]) writes the
//! setting and says so in one line afterwards.
//!
//! Pattern: Command (the write is one `value.set` through [`crate::Remembers`]) behind the Facade; no window code ever writes.

use std::sync::Arc;

use fs_settings::SettingKey;
use fs_types::coded::{Coded, ErrorCode, Param};

use crate::codes::{NotificationCode, keys};
use crate::events::{Event, Open};
use crate::message::{ActionId, Category, Message, NoteId};
use crate::notification::{Action, Notification};
use crate::notifications::{Notifications, lock};
use crate::remember::{Offer, Remembers};

/// The text key of the yellow line when no settings are connected.
const UNAVAILABLE_NOTICE: &str = "notification-notice-remember-unavailable";
/// The text key of the line after a remembered answer.
const REMEMBERED_NOTICE: &str = "notification-notice-remembered";
/// The text key of the line after a remembered answer when the names cannot be said (no texts handed over).
const REMEMBERED_PLAIN_NOTICE: &str = "notification-notice-remembered-plain";
/// The text key of the warning when writing failed.
const FAILED_NOTICE: &str = "notification-notice-remember-failed";
/// The id of the button of the lines: the frontend runs `app.settings key=<key>` for it (the command of
/// `fs-components-commands`, `key` is the setting the line is about).
pub const SETTINGS_ACTION: &str = "settings";
/// The category of the lines (warning: yellow).
const WARNING: &str = "warning";

fn invalid(key: &SettingKey, cause: &str) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::RememberInvalid)
        .with_text(keys::KEY, key.as_str())
        .with_text(keys::CAUSE, cause)
}

impl Notifications {
    /// The way answers are remembered (the host passes `SettingsRemember::new(settings)` once settings open). Without this call
    /// [`crate::NoRemember`] holds: no box, one yellow line, every time asked.
    #[must_use]
    pub fn with_remembers(self, remembers: Arc<dyn Remembers>) -> Self {
        lock(&self.inner.state).remembers = remembers;
        self
    }

    /// Checks the box of a question: returns the question as it will be shown (the box dropped when it is not offered), the offer,
    /// and the key to tell the yellow line about (once).
    pub(crate) fn prepare_remember(
        &self,
        notification: Notification,
    ) -> Result<(Notification, Option<Offer>, Option<SettingKey>), Coded<NotificationCode>> {
        let Some(r) = notification.remembering().cloned() else {
            return Ok((notification, None, None));
        };
        let remembers = Arc::clone(&lock(&self.inner.state).remembers);
        match remembers.offers(&r.key) {
            Offer::Yes { where_key, words } => {
                if !notification
                    .actions
                    .iter()
                    .any(|a| words.iter().any(|w| w == a.id.as_str()))
                {
                    return Err(invalid(&r.key, "no-word"));
                }
                Ok((notification, Some(Offer::Yes { where_key, words }), None))
            }
            Offer::Unavailable => {
                let first = !std::mem::replace(&mut lock(&self.inner.state).unavailable_told, true);
                Ok((
                    notification.without_remember(),
                    None,
                    first.then_some(r.key),
                ))
            }
            // locked or human-only: no box, the question is asked every time (never silent: the lock is visible in settings)
            Offer::Locked { .. } | Offer::HumanOnly => {
                Ok((notification.without_remember(), None, None))
            }
            // a mistake of the program (not declared, not a choice): refused when asked, never a false "not connected"
            Offer::Invalid => Err(invalid(&r.key, "not-a-choice")),
        }
    }

    /// The yellow line, once per facade.
    pub(crate) fn tell_unavailable(&self, key: &SettingKey, now_ms: u64) {
        let line = Message::new(UNAVAILABLE_NOTICE)
            .and_then(|m| m.with_text(keys::KEY, key.as_str()))
            .and_then(|m| warning(Notification::tell(m)));
        if let Ok(n) = line {
            let _ = self.tell(n, now_ms);
        }
    }

    /// After an answer with the box set: writes it (outside every lock) and says so. `Ok(None)` when the action is no word of the
    /// list (a `cancel` is never remembered).
    pub(crate) fn remember_answer(
        &self,
        gone: &Open,
        action: &ActionId,
    ) -> Result<Option<(SettingKey, String)>, Coded<NotificationCode>> {
        let Some(r) = gone.notification.remembering() else {
            return Err(
                Coded::new(NotificationCode::RememberInvalid).with_text(keys::CAUSE, "not-offered")
            );
        };
        let Some(Offer::Yes { words, .. }) = &gone.offer else {
            return Err(invalid(&r.key, "not-offered"));
        };
        if !words.iter().any(|w| w == action.as_str()) {
            return Ok(None);
        }
        let remembers = Arc::clone(&lock(&self.inner.state).remembers);
        match remembers.remember(&r.key, action.as_str(), r.target) {
            Ok(()) => Ok(Some((r.key.clone(), action.as_str().to_owned()))),
            Err(e) => Err(Coded::new(NotificationCode::RememberFailed)
                .with_text(keys::KEY, r.key.as_str())
                .with_text(keys::CAUSE, e.code().as_str())),
        }
    }

    /// The events and the line of the result of `remember_answer`, after the answer is out.
    pub(crate) fn tell_remembered(
        &self,
        id: &NoteId,
        result: &Result<Option<(SettingKey, String)>, Coded<NotificationCode>>,
    ) {
        let now = lock(&self.inner.state).now_ms;
        let line = match result {
            Ok(Some((key, value))) => {
                self.notify(&[Event::Remembered {
                    id: id.clone(),
                    key: key.clone(),
                    value: value.clone(),
                }]);
                // names a person reads, never a key or a raw word; without texts the line names neither
                let remembers = Arc::clone(&lock(&self.inner.state).remembers);
                let named = remembers.describe(key, value);
                match named {
                    Some((place, choice)) => Message::new(REMEMBERED_NOTICE)
                        .and_then(|m| m.with_text("choice", choice))
                        .and_then(|m| m.with_text("where", place)),
                    None => Message::new(REMEMBERED_PLAIN_NOTICE),
                }
                .and_then(|m| m.with_text(keys::KEY, key.as_str()))
                .and_then(|m| settings_button(Notification::tell(m)))
            }
            Err(e) if e.code() == NotificationCode::RememberFailed => {
                let text = |name: &str| {
                    e.params()
                        .find(|(k, _)| *k == name)
                        .map(|(_, v)| match v {
                            Param::Text(t) => t.clone(),
                            _ => String::new(),
                        })
                        .unwrap_or_default()
                };
                Message::new(FAILED_NOTICE)
                    .and_then(|m| m.with_text(keys::KEY, text(keys::KEY)))
                    .and_then(|m| m.with_text(keys::CAUSE, text(keys::CAUSE)))
                    .and_then(|m| warning(Notification::tell(m)))
            }
            _ => return,
        };
        if let Ok(n) = line {
            let _ = self.tell(n, now);
        }
    }
}

fn warning(n: Notification) -> Result<Notification, Coded<NotificationCode>> {
    Ok(n.category(Category::new(WARNING)?))
}

fn settings_button(n: Notification) -> Result<Notification, Coded<NotificationCode>> {
    n.action(Action::new(
        ActionId::new(SETTINGS_ACTION)?,
        Message::new("notification-action-settings")?,
    ))
}

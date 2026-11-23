//! The form rule as a setting (ADR 0106 point 4, ADR 0100 E3): which form a notification takes is decided HERE, by one rule, never by
//! the program. The values are settings of the owner `fs` (kind framework), key group `notification`; a person or an admin can
//! change them (ADR: adjustable, not fixed).
//!
//! Pattern: Strategy (`Policy::choose` is a pure function of the notification and the [`Situation`]) + Declares (the declarations).

use fs_settings::{
    Catalog, Declaration, Declares, Effective, GroupId, Kind, Owner, OwnerKind, SettingCode,
    SettingKey, Settings, Value,
};
use fs_types::coded::Coded;

use crate::notification::{Expire, Notification, Urgency};
use crate::outlet::{Form, Situation, Window};

/// `fs.notification.modal`: `calm` (a critical message or a question takes a modal when nothing is being typed) or `never`.
pub const MODAL: &str = "modal";
/// `fs.notification.system`: `away` (the system form too when the window is away, instead of it when there is none), `always`, `never`.
pub const SYSTEM: &str = "system";
/// `fs.notification.expire-ms`: how long a message of urgency low or normal stays, in milliseconds.
pub const EXPIRE_MS: &str = "expire-ms";
/// The category of a failure: it stays until it is closed (ADR 0065 point 5: a failure stays, success goes by itself).
pub const ERROR_CATEGORY: &str = "error";

/// When a modal may be used.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalWhen {
    /// For a critical message or a question, while nothing is being typed.
    Calm,
    /// Never: a toast stands (with the same actions) instead.
    Never,
}

/// When the system form is added.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemWhen {
    /// When the window is away (additionally), or there is none (instead).
    Away,
    /// Always, in addition.
    Always,
    /// Never.
    Never,
}

/// The rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    /// The modal rule.
    pub modal: ModalWhen,
    /// The system rule.
    pub system: SystemWhen,
    /// The time of a message of urgency low or normal (ms).
    pub expire_ms: u64,
}

impl Default for Policy {
    /// The defaults of the declarations (a test keeps them equal).
    fn default() -> Self {
        Self {
            modal: ModalWhen::Calm,
            system: SystemWhen::Away,
            expire_ms: 6000,
        }
    }
}

impl Policy {
    /// The smallest and the largest `expire-ms`.
    pub const EXPIRE_MIN_MS: u64 = 1000;
    /// The largest `expire-ms`.
    pub const EXPIRE_MAX_MS: u64 = 60_000;

    /// The rule from the effective settings; a value that does not fit stays at the default.
    #[must_use]
    pub fn from_settings(settings: &Settings) -> Self {
        let get = |name: &str| {
            NotificationSettings::key(name)
                .ok()
                .and_then(|k| settings.get(&k))
        };
        Self::from_values(
            get(MODAL).as_ref(),
            get(SYSTEM).as_ref(),
            get(EXPIRE_MS).as_ref(),
        )
    }

    /// The rule from the three effective values (each may be missing).
    #[must_use]
    pub fn from_values(
        modal: Option<&Effective>,
        system: Option<&Effective>,
        expire: Option<&Effective>,
    ) -> Self {
        let mut p = Self::default();
        if let Some(Effective {
            value: Value::Choice(c),
            ..
        }) = modal
        {
            p.modal = match c.as_str() {
                "never" => ModalWhen::Never,
                _ => ModalWhen::Calm,
            };
        }
        if let Some(Effective {
            value: Value::Choice(c),
            ..
        }) = system
        {
            p.system = match c.as_str() {
                "always" => SystemWhen::Always,
                "never" => SystemWhen::Never,
                _ => SystemWhen::Away,
            };
        }
        if let Some(Effective {
            value: Value::Whole(ms),
            ..
        }) = expire
            && let Ok(ms) = u64::try_from(*ms)
        {
            p.expire_ms = ms.clamp(Self::EXPIRE_MIN_MS, Self::EXPIRE_MAX_MS);
        }
        p
    }

    /// The forms for this notification in this situation, in the order they are tried. Empty: the rule chose none (no window and the
    /// system form is off).
    ///
    /// - A critical message or a question: [`Form::Modal`] when the modal rule is `calm` and the situation is calm, else [`Form::Toast`].
    /// - Everything else: [`Form::Toast`].
    /// - No window: no window form at all.
    /// - The system form: by the system rule, see [`SystemWhen`].
    #[must_use]
    pub fn choose(&self, notification: &Notification, situation: &Situation) -> Vec<Form> {
        let mut forms = Vec::new();
        if situation.window != Window::Absent {
            let needs_person =
                notification.urgency == Urgency::Critical || notification.is_question();
            if needs_person && self.modal == ModalWhen::Calm && situation.calm {
                forms.push(Form::Modal);
            } else {
                forms.push(Form::Toast);
            }
        }
        let system = match self.system {
            SystemWhen::Never => false,
            SystemWhen::Always => true,
            SystemWhen::Away => situation.window != Window::Front,
        };
        if system {
            forms.push(Form::System);
        }
        forms
    }

    /// The milliseconds a notification stays, or `None` for until it is closed.
    #[must_use]
    pub fn lifetime_ms(&self, notification: &Notification) -> Option<u64> {
        match notification.expire {
            Expire::Never => None,
            Expire::After(ms) => Some(ms),
            // a question stays until it is answered, whatever its urgency
            Expire::Default if notification.is_question() => None,
            // a failure stays until it is closed (ADR 0065 point 5), told as `normal` or not
            Expire::Default
                if notification
                    .category
                    .as_ref()
                    .is_some_and(|c| c.as_str() == ERROR_CATEGORY) =>
            {
                None
            }
            Expire::Default => match notification.urgency {
                Urgency::Critical => None,
                Urgency::Low | Urgency::Normal => Some(self.expire_ms),
            },
        }
    }
}

/// The declarations of the notification rule: `fs.notification.modal`, `.system`, `.expire-ms`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NotificationSettings;

impl NotificationSettings {
    /// The key of `modal`, `system` or `expire-ms`.
    ///
    /// # Errors
    /// `key-invalid` (never for the three constants; a test proves it).
    pub fn key(name: &str) -> Result<SettingKey, Coded<SettingCode>> {
        SettingKey::new(&Owner::new(OwnerKind::Framework, "notification")?, name)
    }

    /// The three declarations as a catalog, to be added to a program's catalog.
    ///
    /// # Errors
    /// `duplicate-key` or `default-invalid` (never; a test proves it).
    pub fn catalog() -> Result<Catalog, Coded<SettingCode>> {
        Catalog::of(Self.declarations()?)
    }
}

impl Declares for NotificationSettings {
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>> {
        let d = Policy::default();
        let group = || GroupId::new("notification");
        let ms = |n: u64| {
            i64::try_from(n).map_err(|_| {
                Coded::new(SettingCode::DefaultInvalid)
                    .with_text("key", "fs.notification.expire-ms")
            })
        };
        Ok(vec![
            Declaration::new(
                Self::key(MODAL)?,
                Kind::Choice {
                    keys: vec!["calm".to_owned(), "never".to_owned()],
                },
                Value::Choice("calm".to_owned()),
                group()?,
            ),
            Declaration::new(
                Self::key(SYSTEM)?,
                Kind::Choice {
                    keys: vec!["away".to_owned(), "always".to_owned(), "never".to_owned()],
                },
                Value::Choice("away".to_owned()),
                group()?,
            ),
            Declaration::new(
                Self::key(EXPIRE_MS)?,
                Kind::Whole {
                    min: ms(Policy::EXPIRE_MIN_MS)?,
                    max: ms(Policy::EXPIRE_MAX_MS)?,
                },
                Value::Whole(ms(d.expire_ms)?),
                group()?,
            ),
        ])
    }
}

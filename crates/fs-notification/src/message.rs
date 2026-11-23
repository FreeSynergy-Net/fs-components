//! Names and the words of a notification: ids, and a text as a CODE with parameters (ADR 0057, ADR 0106 point 1).
//!
//! Pattern: Value objects (checked on creation, never changed). The program does not write a sentence; it names the key of its MF2
//! text and the parameters ([`Message`]), the frontend words it. A [`Message`] is the dynamic form of `fs_types::coded::Code`
//! (whose prefix is a static string and so cannot come from a command or a workflow block).

use fs_types::coded::{Code, Coded, Param};

use crate::codes::{NotificationCode, keys};

/// What was named wrong.
pub(crate) fn invalid(what: &str, value: &str) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::NameInvalid)
        .with_text(keys::WHAT, what)
        .with_text(keys::VALUE, value)
}

/// One word of a name: 1 to 64 bytes of `a-z`, `0-9`, `-`; starts and ends with a letter or digit; no `--`.
pub(crate) fn word_ok(w: &str) -> bool {
    let b = w.as_bytes();
    !b.is_empty()
        && b.len() <= 64
        && b[0].is_ascii_alphanumeric()
        && b[b.len() - 1].is_ascii_alphanumeric()
        && !w.contains("--")
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

macro_rules! name {
    ($(#[$doc:meta])* $ty:ident, $what:literal) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $ty(String);

        impl $ty {
            /// A name of this kind.
            ///
            /// # Errors
            /// `name-invalid` when it is not one word of `a-z`, `0-9`, `-` (1 to 64 bytes).
            pub fn new(text: &str) -> Result<Self, Coded<NotificationCode>> {
                if word_ok(text) {
                    Ok(Self(text.to_owned()))
                } else {
                    Err(invalid($what, text))
                }
            }

            /// The name as text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

name!(
    /// The id of an open notification. Only this crate issues them ([`crate::Notifications`]): an id that comes from outside
    /// and was not issued here is never trusted (ADR 0106 point 8).
    NoteId,
    "id"
);
name!(
    /// The id of an action (a button) of a notification; unique within it.
    ActionId,
    "action"
);
name!(
    /// The category of a notification (freedesktop `category`, Android channel, WHATWG `tag` kind): `update`, `error` ...
    Category,
    "category"
);
name!(
    /// The occasion a notification belongs to (a view, a task): it closes when the person leaves that scope (ADR 0065 point 4).
    Scope,
    "scope"
);

impl NoteId {
    /// The id the facade issues as number `n`.
    pub(crate) fn issued(n: u64) -> Self {
        Self(format!("n{n}"))
    }
}

/// A text of a notification: the key of its MF2 text and typed parameters, never a sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    key: String,
    params: Vec<(String, Param)>,
}

impl Message {
    /// The text with this key and no parameters.
    ///
    /// # Errors
    /// `name-invalid` (`what` = `message`) when the key is not a word of `a-z`, `0-9`, `-`.
    pub fn new(key: &str) -> Result<Self, Coded<NotificationCode>> {
        if word_ok(key) {
            Ok(Self {
                key: key.to_owned(),
                params: Vec::new(),
            })
        } else {
            Err(invalid("message", key))
        }
    }

    /// A text parameter.
    ///
    /// # Errors
    /// `name-invalid` (`what` = `parameter`) when the parameter key is not a word.
    pub fn with_text(
        self,
        name: &str,
        value: impl Into<String>,
    ) -> Result<Self, Coded<NotificationCode>> {
        self.with_param(name, Param::Text(value.into()))
    }

    /// A number parameter.
    ///
    /// # Errors
    /// `name-invalid` (`what` = `parameter`) when the parameter key is not a word.
    pub fn with_number(self, name: &str, value: u64) -> Result<Self, Coded<NotificationCode>> {
        self.with_param(name, Param::Number(value))
    }

    fn with_param(mut self, name: &str, value: Param) -> Result<Self, Coded<NotificationCode>> {
        if !word_ok(name) {
            return Err(invalid("parameter", name));
        }
        self.params.push((name.to_owned(), value));
        Ok(self)
    }

    /// The message of a code of any crate (its message key and parameters, unchanged).
    ///
    /// # Errors
    /// `name-invalid` when the key of the code is not a word (a code of another crate that breaks the form).
    pub fn from_code(code: &Code) -> Result<Self, Coded<NotificationCode>> {
        let mut m = Self::new(&code.message_key())?;
        for (k, v) in &code.params {
            m = m.with_param(k, v.clone())?;
        }
        Ok(m)
    }

    /// The key of the MF2 text.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The parameters, in order.
    pub fn params(&self) -> impl Iterator<Item = (&str, &Param)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// The same text with every VALUE masked ([`MASK`]) and every name kept: what may be shown where the screen is not the person's
    /// alone (the lock screen, the system form; ADR 0106 point 8, ADR 0069 E2). The names stay so the MF2 text still finds every
    /// placeholder and says a sentence; with no parameter at all it would not resolve and only its key would show.
    #[must_use]
    pub fn redacted(&self) -> Self {
        Self {
            key: self.key.clone(),
            params: self
                .params
                .iter()
                .map(|(name, _)| (name.clone(), Param::Text(MASK.to_owned())))
                .collect(),
        }
    }
}

/// What stands in the place of a value that must not be shown.
pub const MASK: &str = "…";

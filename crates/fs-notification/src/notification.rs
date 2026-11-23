//! The notification itself: the common core of freedesktop Notifications 1.3, the XDG portal, Windows App Notifications, Android
//! and WHATWG Notifications (ADR 0106 point 1): replaces, urgency, title and body as codes, actions, expiry, category.
//!
//! Pattern: Builder (`Notification::tell(..).urgency(..).expire(..)`) + Value object. Checked when it is shown, so a bad one is a
//! code ([`NotificationCode`]), never a panic.

use fs_types::coded::Coded;

use crate::codes::{NotificationCode, keys};
use crate::message::{ActionId, Category, Message, NoteId, Scope};

/// How urgent (freedesktop urgency 0 to 2; the portal's `urgent` is `Critical`).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Urgency {
    /// Can wait; never takes the screen.
    Low,
    /// The usual.
    #[default]
    Normal,
    /// Stays until it is closed (no expiry unless one is set); asks for the person.
    Critical,
}

impl Urgency {
    /// The name in a command (`low`, `normal`, `critical`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::Critical => "critical",
        }
    }

    /// From the name in a command, or the portal's `urgent`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "critical" | "urgent" => Some(Self::Critical),
            _ => None,
        }
    }
}

/// How long it stays.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Expire {
    /// By urgency: `Low` and `Normal` for the time of the policy, `Critical` until it is closed (ADR 0065: success goes by
    /// itself, a failure stays). A question always stays until it is answered.
    #[default]
    Default,
    /// Until it is closed.
    Never,
    /// This many milliseconds.
    After(u64),
}

/// Why a notification is gone (freedesktop 9.2.1).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CloseReason {
    /// The time ran out.
    Expired,
    /// The person closed it (or answered it).
    Dismissed,
    /// The program, or the end of its scope, closed it.
    Closed,
    /// Unknown (an outlet that cannot tell).
    Undefined,
}

impl CloseReason {
    /// The name (`expired`, `dismissed`, `closed`, `undefined`); also the AS2 extension field `fs:closeReason` later.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Expired => "expired",
            Self::Dismissed => "dismissed",
            Self::Closed => "closed",
            Self::Undefined => "undefined",
        }
    }
}

/// One button of a notification: an id and its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    /// The id, unique within the notification.
    pub id: ActionId,
    /// The text of the button.
    pub label: Message,
}

impl Action {
    /// An action.
    #[must_use]
    pub fn new(id: ActionId, label: Message) -> Self {
        Self { id, label }
    }
}

/// A message (`tell`) or a question (`ask`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    /// The headline.
    pub title: Message,
    /// The text below it.
    pub body: Option<Message>,
    /// How urgent.
    pub urgency: Urgency,
    /// What kind of notification.
    pub category: Option<Category>,
    /// How long it stays.
    pub expire: Expire,
    /// The buttons. A question needs at least one.
    pub actions: Vec<Action>,
    /// An open notification of this crate to replace (same id, new content). An id that is not open is ignored.
    pub replaces: Option<NoteId>,
    /// The occasion it belongs to.
    pub scope: Option<Scope>,
    question: bool,
}

impl Notification {
    fn base(title: Message, question: bool) -> Self {
        Self {
            title,
            body: None,
            urgency: Urgency::Normal,
            category: None,
            expire: Expire::Default,
            actions: Vec::new(),
            replaces: None,
            scope: None,
            question,
        }
    }

    /// A message.
    #[must_use]
    pub fn tell(title: Message) -> Self {
        Self::base(title, false)
    }

    /// A question: the person must answer with one of the `actions`.
    ///
    /// # Errors
    /// `actions-invalid` when there is no action or an id stands twice.
    pub fn ask(title: Message, actions: Vec<Action>) -> Result<Self, Coded<NotificationCode>> {
        let mut n = Self::base(title, true);
        n.actions = actions;
        n.check_actions()?;
        Ok(n)
    }

    /// With a body.
    #[must_use]
    pub fn body(mut self, body: Message) -> Self {
        self.body = Some(body);
        self
    }

    /// With an urgency.
    #[must_use]
    pub fn urgency(mut self, urgency: Urgency) -> Self {
        self.urgency = urgency;
        self
    }

    /// With a category.
    #[must_use]
    pub fn category(mut self, category: Category) -> Self {
        self.category = Some(category);
        self
    }

    /// With an expiry.
    #[must_use]
    pub fn expire(mut self, expire: Expire) -> Self {
        self.expire = expire;
        self
    }

    /// Replacing an open notification.
    #[must_use]
    pub fn replaces(mut self, id: NoteId) -> Self {
        self.replaces = Some(id);
        self
    }

    /// Belonging to an occasion.
    #[must_use]
    pub fn scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// With a button (a message may have some too).
    ///
    /// # Errors
    /// `actions-invalid` when its id is already there.
    pub fn action(mut self, action: Action) -> Result<Self, Coded<NotificationCode>> {
        self.actions.push(action);
        self.check_actions()?;
        Ok(self)
    }

    /// Is it a question?
    #[must_use]
    pub fn is_question(&self) -> bool {
        self.question
    }

    /// The same without the parameters of its texts: for a place the person does not own alone ([`crate::Form::System`]).
    #[must_use]
    pub fn redacted(&self) -> Self {
        let mut n = self.clone();
        n.title = n.title.redacted();
        n.body = n.body.as_ref().map(Message::redacted);
        for a in &mut n.actions {
            a.label = a.label.redacted();
        }
        n
    }

    fn check_actions(&self) -> Result<(), Coded<NotificationCode>> {
        let bad = |cause: &str| {
            Coded::new(NotificationCode::ActionsInvalid).with_text(keys::CAUSE, cause)
        };
        if self.question && self.actions.is_empty() {
            return Err(bad("no-action"));
        }
        for (i, a) in self.actions.iter().enumerate() {
            if self.actions[..i].iter().any(|b| b.id == a.id) {
                return Err(bad("duplicate-action"));
            }
        }
        Ok(())
    }
}

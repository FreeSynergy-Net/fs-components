//! `fs-notification`: messages, failures and questions as an interface of their own (ADR 0106). Every program says what happened to
//! the person through ONE module; where and how it appears (a toast, a dialog, the notification of the operating system) is decided
//! by a rule that is a setting, and shown by an outlet that a frontend or an adapter brings. This crate has neither a window nor an
//! operating system.
//!
//! The core is the common ground of freedesktop Notifications 1.3, the XDG portal, Windows App Notifications, Android and WHATWG
//! Notifications: id and `replaces`, urgency low/normal/critical, title and body as CODES with parameters (ADR 0057: the program
//! words them in MF2), actions, expiry, category, close reason expired/dismissed/closed/undefined.
//!
//! Rules of the crate:
//! - One source of truth: a program that has something to say calls [`Notifications`] and nothing else (`fs-starter::Tell` becomes
//!   an adapter on it, A4 of the cut). The status line shows state only.
//! - No sentence in a call: a [`Message`] is the key of an MF2 text and typed parameters.
//! - The form is chosen by [`Policy`] (settings `fs.notification.*`), never by the program.
//! - What comes from outside is not trusted: an answer counts only for an id this crate issued and an action the notification has
//!   (the test of ADR 0106 point 8; the operating system's side is the adapter's job).
//! - The system form never gets the values of the texts: text values are masked, the names stay ([`MASK`]); a number parameter
//!   stays visible (see [`Message::with_number`]: a secret goes as text).
//! - The history (AS2 `Create`/`Note`, `Question`; ADR 0106 point 7) is NOT built here: it waits for `fs-activities` to be connected.
//!   No list and no history exist yet, and nothing here pretends otherwise.
//!
//! Pattern: Port and Adapter ([`Outlet`]) + Facade ([`Notifications`]) + Observer with a lifetime ([`Subscription`]) + Strategy
//! ([`Policy`], a pure function) + Command ([`Notifications::run`]).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

mod codes;
mod command;
mod delivery;
mod events;
mod inbound;
mod message;
mod notification;
mod notifications;
mod outlet;
mod policy;
mod remember;
mod remembered;
mod texts;

pub use codes::{NotificationCode, keys};
pub use command::{ASK, CLOSE, TELL};
pub use events::{Event, Observer, Open, Outcome};
pub use message::{ActionId, Category, MASK, Message, NoteId, Scope};
pub use notification::{Action, CloseReason, Expire, Notification, Urgency};
pub use notifications::{ENDED_KEPT, Notifications, REJECTED_KEPT, Subscription};
pub use outlet::{Form, Outlet, Situation, Window};
pub use policy::{
    ERROR_CATEGORY, EXPIRE_MS, MODAL, ModalWhen, NotificationSettings, Policy, SYSTEM, SystemWhen,
};
pub use remember::{ASK_WORD, NoRemember, Offer, Remember, Remembers, SettingsRemember};
pub use remembered::SETTINGS_ACTION;
pub use texts::{TEXT_DE, TEXT_EN};

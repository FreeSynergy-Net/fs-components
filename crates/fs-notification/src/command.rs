//! The three commands `notification.tell`, `notification.ask`, `notification.close` as calls with text arguments: the way a menu,
//! the palette, MCP, the command line and a block of a workflow reach this module (ADR 0106 point 3). Their DESCRIPTION (ids,
//! parameters, result) is `fs-components-commands` `standard::notification_commands`; the names are taken from there.
//!
//! Argument form (as in the models: freedesktop Notifications `actions` is a list of (action id, label) pairs, Web Notifications
//! `actions` a list of {action, title}): `actions` = pairs `action-id=text-key` (the label is the key of an MF2 text, never a
//! sentence); `params` = pairs `name=value` for the placeholders of the texts, typed by the name: `name=text` is a text,
//! `name:number=3` a whole number (a wrong number is `bad-command`, not a text). Reason: the command line, a menu and a workflow
//! block hand over text only, so the type has to be in the pair, or a count could not be told from a word.
//!
//! Pattern: Command (the string form of the facade's three calls). Arguments are `(name, value)` pairs; a name may repeat
//! (`actions`, `params`).

use fs_types::coded::Coded;

use crate::codes::{NotificationCode, keys};
use crate::message::{ActionId, Category, Message, NoteId, Scope};
use crate::notification::{Action, Expire, Notification, Urgency};
use crate::notifications::Notifications;

/// `notification.tell`, `notification.ask` and `notification.close`: the names are those of the description in `fs-components-commands`
/// (one source, no copy).
pub use fs_components_commands::standard::{
    NOTIFICATION_ASK as ASK, NOTIFICATION_CLOSE as CLOSE, NOTIFICATION_TELL as TELL,
};

fn bad(command: &str, argument: &str) -> Coded<NotificationCode> {
    Coded::new(NotificationCode::BadCommand)
        .with_text(keys::COMMAND, command)
        .with_text(keys::ARGUMENT, argument)
}

fn first<'a>(args: &[(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    args.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
}

fn all<'a>(args: &[(&'a str, &'a str)], name: &str) -> Vec<&'a str> {
    args.iter()
        .filter(|(k, _)| *k == name)
        .map(|(_, v)| *v)
        .collect()
}

fn message(command: &str, key: &str, params: &[&str]) -> Result<Message, Coded<NotificationCode>> {
    let mut m = Message::new(key)?;
    for p in params {
        let (name, value) = p.split_once('=').ok_or_else(|| bad(command, "params"))?;
        // typed: `name=text`, or `name:number=3` for a number (a placeholder with a plural or a count)
        m = match name.strip_suffix(":number") {
            Some(name) => m.with_number(
                name,
                value.parse::<u64>().map_err(|_| bad(command, "params"))?,
            )?,
            None => m.with_text(name, value)?,
        };
    }
    Ok(m)
}

fn build(command: &str, args: &[(&str, &str)]) -> Result<Notification, Coded<NotificationCode>> {
    let params = all(args, "params");
    let title = message(
        command,
        first(args, "title").ok_or_else(|| bad(command, "title"))?,
        &params,
    )?;
    let mut actions = Vec::new();
    for entry in all(args, "actions") {
        let (id, label) = entry
            .split_once('=')
            .ok_or_else(|| bad(command, "actions"))?;
        actions.push(Action::new(
            ActionId::new(id)?,
            message(command, label, &params)?,
        ));
    }
    let mut n = if command == ASK {
        Notification::ask(title, actions)?
    } else {
        let mut n = Notification::tell(title);
        for a in actions {
            n = n.action(a)?;
        }
        n
    };
    if let Some(body) = first(args, "body") {
        n = n.body(message(command, body, &params)?);
    }
    if let Some(u) = first(args, "urgency") {
        n = n.urgency(Urgency::parse(u).ok_or_else(|| bad(command, "urgency"))?);
    }
    if let Some(c) = first(args, "category") {
        n = n.category(Category::new(c)?);
    }
    if let Some(s) = first(args, "scope") {
        n = n.scope(Scope::new(s)?);
    }
    if let Some(r) = first(args, "replaces") {
        n = n.replaces(NoteId::new(r)?);
    }
    if let Some(ms) = first(args, "expire-ms") {
        let ms = ms.parse::<u64>().map_err(|_| bad(command, "expire-ms"))?;
        n = n.expire(if ms == 0 {
            Expire::Never
        } else {
            Expire::After(ms)
        });
    }
    Ok(n)
}

impl Notifications {
    /// Runs one of the three commands. `tell` and `ask` return the id of the notification, `close` returns nothing.
    ///
    /// # Errors
    /// `bad-command` for an unknown command or a missing or wrong argument; else the code of the call.
    pub fn run(
        &self,
        command: &str,
        args: &[(&str, &str)],
        now_ms: u64,
    ) -> Result<Option<NoteId>, Coded<NotificationCode>> {
        match command {
            TELL => self.tell(build(command, args)?, now_ms).map(Some),
            ASK => self.ask(build(command, args)?, now_ms).map(Some),
            CLOSE => {
                let id = first(args, "id").ok_or_else(|| bad(command, "id"))?;
                self.close(&NoteId::new(id)?).map(|()| None)
            }
            other => Err(bad(other, "command")),
        }
    }
}

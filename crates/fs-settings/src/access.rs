//! Who is asking: the view of the settings for a person and for a bot ([`Actor`], [`Surface`]).
//!
//! The actor comes from the CONNECTION only (peer credentials of the socket, or the bot key), never from a field the client
//! sends (rule AKTEUR-AUS-VERBINDUNG): the service builds the [`Actor`], this module only acts on it. A key declared
//! [`crate::Declaration::human_only`] does not exist for a bot, in every way the module offers: list, read, write, search,
//! export, history, problems. A refused read or write answers like a key nobody declared (`not-declared`), a list shows
//! nothing of it, and a change of it is not told: the bot cannot learn that the key exists (ADR 0054, supplement 2: "all
//! `NotFound` or refused"). The check is HERE, in the module that keeps the settings, not in a transport in front of it.
//!
//! Limit (ADR 0054, open point 1): a bot actor means a connection WITH a bot key (the entry of the AI always has one,
//! ADR 0091). A program that runs as the same person on this machine and connects as that person is a person for this
//! module; that is not prevented here and not claimed.
//!
//! Duty of the service in front (fs-settings-rpc, K6-b): it MUST lead every bot way through [`Surface`]. [`Settings`] and
//! [`Admin`] are never handed to a bot directly: they give a human-only key unfiltered.
//!
//! Pattern: Proxy (`Surface` has the methods of [`Settings`] and [`Admin::log`] and filters for the actor).

use fs_types::coded::{Coded, Param};

use crate::admin::{Admin, InstanceLogEntry};
use crate::codes::{SettingCode, keys};
use crate::declare::Declaration;
use crate::layer::Target;
use crate::names::SettingKey;
use crate::resolve::Effective;
use crate::settings::{Change, Settings};
use crate::value::Value;
use fs_types::ObjectId;

#[cfg(test)]
mod tests;

/// Who acts, as the connection says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Actor {
    /// A person (their own connection).
    Person,
    /// A bot: a connection with a bot key, such as the entry of an AI.
    Bot,
    /// Test only: an actor that a later version could add; it must be restricted like a bot.
    #[cfg(test)]
    Other,
}

/// The settings of one program as one actor sees them.
pub struct Surface<'a> {
    settings: &'a Settings,
    actor: Actor,
}

impl<'a> Surface<'a> {
    /// The view of `settings` for `actor`.
    #[must_use]
    pub fn new(settings: &'a Settings, actor: Actor) -> Self {
        Self { settings, actor }
    }

    /// Whether `key` exists for this actor: declared, and not human-only for a bot.
    #[must_use]
    pub fn sees(&self, key: &SettingKey) -> bool {
        self.settings
            .catalog()
            .get(key)
            .is_some_and(|d| self.shows(d))
    }

    /// The ONE rule (fail closed): everything that is not a person is restricted, also an actor added later.
    fn restricted(&self) -> bool {
        self.actor != Actor::Person
    }

    fn shows(&self, d: &Declaration) -> bool {
        !self.restricted() || !d.human_only
    }

    fn hidden_key(&self, key: &str) -> bool {
        self.restricted()
            && self
                .settings
                .catalog()
                .iter()
                .any(|d| d.human_only && d.key.as_str() == key)
    }

    fn not_found(key: &SettingKey) -> Coded<SettingCode> {
        Coded::new(SettingCode::NotDeclared).with_text(keys::KEY, key.as_str())
    }

    /// The declarations the actor can see, ordered by key.
    #[must_use]
    pub fn list(&self) -> Vec<&'a Declaration> {
        self.settings
            .catalog()
            .iter()
            .filter(|d| self.shows(d))
            .collect()
    }

    /// The declarations whose key contains `needle` (ignoring case), among those the actor can see.
    #[must_use]
    pub fn search(&self, needle: &str) -> Vec<&'a Declaration> {
        let needle = needle.to_lowercase();
        let mut found = self.list();
        found.retain(|d| d.key.as_str().to_lowercase().contains(&needle));
        found
    }

    /// The effective value of `key`.
    ///
    /// # Errors
    /// `not-declared` when no block declared it, or it is human-only and the actor is a bot.
    pub fn get(&self, key: &SettingKey) -> Result<Effective, Coded<SettingCode>> {
        if !self.sees(key) {
            return Err(Self::not_found(key));
        }
        self.settings.get(key).ok_or_else(|| Self::not_found(key))
    }

    /// Every effective value the actor can see, ordered by key (the export of the module).
    #[must_use]
    pub fn export(&self) -> Vec<(SettingKey, Effective)> {
        self.list()
            .into_iter()
            .filter_map(|d| Some((d.key.clone(), self.settings.get(&d.key)?)))
            .collect()
    }

    /// [`Settings::save`] for the actor.
    ///
    /// # Errors
    /// As [`Settings::save`]; `not-declared` for a human-only key and a bot: nothing is written then. The changes the
    /// answer lists are those of keys the actor can see.
    pub fn save(
        &self,
        key: &SettingKey,
        value: Option<&Value>,
        target: Target,
    ) -> Result<Vec<Change>, Coded<SettingCode>> {
        if !self.sees(key) {
            return Err(Self::not_found(key));
        }
        let mut changes = self.settings.save(key, value, target)?;
        changes.retain(|c| self.sees(&c.key));
        Ok(changes)
    }

    /// What went wrong at the last read, without the problems that name a key the actor cannot see.
    #[must_use]
    pub fn problems(&self) -> Vec<Coded<SettingCode>> {
        let mut all = self.settings.problems();
        all.retain(|p| {
            !p.params()
                .any(|(k, v)| k == keys::KEY && matches!(v, Param::Text(t) if self.hidden_key(t)))
        });
        all
    }

    /// [`Admin::log`] for the actor: the lines of keys the actor cannot see are left out, and the page is filled up with
    /// the next visible lines, so it has `size` lines unless the log really ends (side channel: a short page, or an empty
    /// page in the middle, would tell a bot that and how often a hidden key changed). The caller pages on with the id of
    /// the last line it got. The ids of the lines carry no counter either (see [`InstanceLogEntry::id`]): the distance of
    /// two ids does not tell how many hidden lines lie between them (only a rough, statistical trace of a random step is
    /// left; the exact count is not in the ids).
    #[must_use]
    pub fn history(
        &self,
        admin: &Admin,
        after: Option<ObjectId>,
        size: u32,
    ) -> Vec<InstanceLogEntry> {
        let want = size as usize;
        let mut page = Vec::new();
        let mut from = after;
        while page.len() < want {
            let chunk = admin.log(from, size);
            let Some(last) = chunk.last().map(|l| l.id) else {
                break;
            };
            for line in chunk {
                if !self.hidden_key(line.key.as_str()) && page.len() < want {
                    page.push(line);
                }
            }
            from = Some(last);
        }
        page
    }
}

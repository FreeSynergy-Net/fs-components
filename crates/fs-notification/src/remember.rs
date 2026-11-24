//! Remembering the answer of a question as a setting: what may be remembered ([`Remember`]), whether the box is offered
//! ([`Offer`]) and where the answer goes ([`Remembers`]).
//!
//! Pattern: Strategy + Command (the setting chooses the behaviour out of a closed list whose word `ask` defers the choice to the
//! person; "remember" is a command, `value.set` over `fs-settings`, that writes the answer as the new value of the same setting).
//! The name of the pattern elsewhere is "Don't ask again" (KDE `dontAskAgainName`, Win32 `TDF_VERIFICATION_FLAG_CHECKED`, W3C
//! Permissions `prompt`); it is not in the table of `coding.md` section 2, so Strategy + Command stands here (reported to the strategy).
//! The module writes, never the caller (macOS `suppressionButton` is the counter-example: every program writes itself).
//! This crate draws nothing: it hands out data (the key, the words, where it can be changed), a frontend draws the box.

use std::sync::Arc;

use fs_components_words::{Args, TextId, Words};
use fs_settings::{Actor, Coded, Kind, SettingCode, SettingKey, Settings, Surface, Target, Value};

/// The word of the choice that means "ask": it is never offered as an action and never remembered.
pub const ASK_WORD: &str = "ask";

/// What a question may remember: the declared setting and which of the two save buttons is used.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remember {
    /// The declared setting the answer may become. The ids of the actions are the words of its list.
    pub key: SettingKey,
    /// Where it is saved (the default of the draft: all programs).
    pub target: Target,
}

impl Remember {
    /// Remember as `key`, saved where `target` says (the draft's default is [`Target::AllPrograms`]).
    #[must_use]
    pub fn new(key: SettingKey, target: Target) -> Self {
        Self { key, target }
    }
}

/// Whether the box "remember" may be offered for a key.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Offer {
    /// Yes: the setting can be written by this actor.
    Yes {
        /// The MF2 key of the name of the setting (`setting-label-...`): where it can be changed later.
        where_key: String,
        /// The words of its list, without `ask`: the ids of the actions that may be remembered.
        words: Vec<String>,
    },
    /// A lock holds the key; nobody changes it (the reason is the admin's).
    Locked {
        /// The reason as the admin wrote it.
        reason: String,
    },
    /// Only a person may change it, and this answer does not come from one.
    HumanOnly,
    /// Nothing can be written here: no settings are connected (a state of the host, told once by a yellow line).
    Unavailable,
    /// The key is not a declared choice (not declared, or another kind): a mistake of the program that asks, not a state.
    Invalid,
}

/// Where a remembered answer goes.
///
/// Pattern: Strategy – the host hands one over ([`SettingsRemember`]), else [`NoRemember`] holds.
pub trait Remembers: Send + Sync {
    /// Whether the box is offered for `key`.
    fn offers(&self, key: &SettingKey) -> Offer;

    /// The words a person reads in the line after a remembered answer: the name of the setting and the name of the choice, in the
    /// language of the texts the adapter holds. `None` when they cannot be said (no texts handed over): the line then names neither.
    fn describe(&self, _key: &SettingKey, _word: &str) -> Option<(String, String)> {
        None
    }

    /// Writes `value` as the new value of `key`.
    ///
    /// # Errors
    /// The codes of `fs-settings`, unchanged: `not-declared`, `locked`, the kind's own code, `write-failed`.
    fn remember(
        &self,
        key: &SettingKey,
        value: &str,
        target: Target,
    ) -> Result<(), Coded<SettingCode>>;
}

/// Null object: nothing is remembered, no box is offered (the state until the host reads settings).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoRemember;

impl Remembers for NoRemember {
    fn offers(&self, _key: &SettingKey) -> Offer {
        Offer::Unavailable
    }

    fn remember(
        &self,
        key: &SettingKey,
        _value: &str,
        _target: Target,
    ) -> Result<(), Coded<SettingCode>> {
        Err(Coded::new(SettingCode::NotDeclared).with_text(fs_settings::keys::KEY, key.as_str()))
    }
}

/// Adapter on [`Settings`]: the answer becomes the value of the setting, through the same door a bot would use ([`Surface`]), so a
/// human-only key is closed to a bot and a lock holds.
pub struct SettingsRemember {
    settings: Arc<Settings>,
    actor: Actor,
    words: Option<Arc<dyn Words>>,
}

impl SettingsRemember {
    /// For the person at the keyboard.
    #[must_use]
    pub fn new(settings: Arc<Settings>) -> Self {
        Self::acting_as(settings, Actor::Person)
    }

    /// For answers that come from this actor (a bot over MCP is never a person).
    #[must_use]
    pub fn acting_as(settings: Arc<Settings>, actor: Actor) -> Self {
        Self {
            settings,
            actor,
            words: None,
        }
    }

    /// The texts that say the name of a setting and of a choice (`setting-label-...`, `setting-choice-...`) for the line after a
    /// remembered answer; the language is theirs.
    #[must_use]
    pub fn with_words(mut self, words: Arc<dyn Words>) -> Self {
        self.words = Some(words);
        self
    }
}

impl Remembers for SettingsRemember {
    fn offers(&self, key: &SettingKey) -> Offer {
        let Some(d) = self.settings.catalog().get(key) else {
            return Offer::Invalid;
        };
        if !Surface::new(&self.settings, self.actor).sees(key) {
            return Offer::HumanOnly;
        }
        if let Some(lock) = self.settings.get(key).and_then(|e| e.lock) {
            return Offer::Locked {
                reason: lock.reason,
            };
        }
        match &d.kind {
            Kind::Choice { keys } => Offer::Yes {
                where_key: d.label_key(),
                words: keys.iter().filter(|w| *w != ASK_WORD).cloned().collect(),
            },
            _ => Offer::Invalid,
        }
    }

    fn describe(&self, key: &SettingKey, word: &str) -> Option<(String, String)> {
        let words = self.words.as_ref()?;
        let d = self.settings.catalog().get(key)?;
        let say = |id: String| {
            let said = words.say(&TextId::new(id), &Args::new());
            said.is_found().then(|| said.text().to_owned())
        };
        Some((say(d.label_key())?, say(d.choice_key(word))?))
    }

    fn remember(
        &self,
        key: &SettingKey,
        value: &str,
        target: Target,
    ) -> Result<(), Coded<SettingCode>> {
        let v = Value::Choice(value.to_owned());
        Surface::new(&self.settings, self.actor)
            .save(key, Some(&v), target)
            .map(|_| ())
    }
}

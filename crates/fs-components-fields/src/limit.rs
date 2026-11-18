//! How many field values a slot of a block takes (the facts of a card, the elements of a detail frame): a SETTING of the
//! instance, never a constant of the code (rule 57: a number a person or an admin could want to change is a default). The
//! block declares the setting ([`declaration`]) with the default it brings; the host reads the effective value ([`Limit::read`])
//! and hands it to the card or the frame, which check it in their constructor and in their `fact`/`element`.
//!
//! There is no ceiling of ours: the kind of a whole number needs a largest value, and it is `i32::MAX`, the largest number
//! every platform's `usize` holds; a layout or memory limit would have to be measured and named here, none is.
//!
//! Pattern: Value object (`Limit`, never zero) + Factory (`declaration`, the one way a block declares its limit).

use fs_settings::{Declaration, GroupId, Kind, SettingKey, Settings, Value};

/// How many values a slot takes: at least one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(usize);

impl Limit {
    /// A limit of `max`; `None` for 0 (a slot that takes nothing is no slot).
    pub fn new(max: usize) -> Option<Self> {
        (max > 0).then_some(Self(max))
    }

    /// A limit known at compile time (a default); 0 does not compile where it is used in a `const`.
    ///
    /// # Panics
    /// For 0.
    pub const fn of(max: usize) -> Self {
        assert!(max > 0, "a limit is at least one");
        Self(max)
    }

    /// The number.
    pub fn get(self) -> usize {
        self.0
    }

    /// The effective value of `key`, or `default` when no block declared the key or the value is no whole number of at least
    /// one (the problem is then in `Settings::problems`, where the settings window shows it).
    pub fn read(settings: &Settings, key: &SettingKey, default: Limit) -> Limit {
        match settings.get(key).map(|e| e.value) {
            Some(Value::Whole(n)) => usize::try_from(n)
                .ok()
                .and_then(Limit::new)
                .unwrap_or(default),
            _ => default,
        }
    }
}

/// The declaration of a limit: a whole number from 1 to `i32::MAX`, `default` as the block's default (the lowest layer).
pub fn declaration(key: SettingKey, default: Limit, group: GroupId) -> Declaration {
    Declaration::new(
        key,
        Kind::Whole {
            min: 1,
            max: i64::from(i32::MAX),
        },
        Value::Whole(i64::try_from(default.get()).unwrap_or(i64::from(i32::MAX))),
        group,
    )
}

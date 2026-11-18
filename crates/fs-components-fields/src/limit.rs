//! How many field values a slot of a block takes (the facts of a card, the elements of a detail frame). The block does NOT read
//! settings and does not depend on `fs-settings`: the limit comes in as a PARAMETER (a checked [`Limit`] handed to the
//! constructor), and the block brings a default as a CONSTANT. What the block also brings, as plain data, is the DECLARATION of
//! the setting ([`LimitSetting`]: key, default, group, the keys of its words); the host (fs-forms-bind over the settings store)
//! registers it and hands the effective value in (rule 57: a number a person or an admin could want to change is a default).
//!
//! The one ceiling is [`Limit::MAX`], `i32::MAX`: the largest whole number every platform's `usize` and the settings kind of a
//! whole number hold. It is no layout or memory limit of ours; none is measured, none is claimed.
//!
//! Pattern: Value object (`Limit`, 1 ..= MAX) + plain data (`LimitSetting`).

/// How many values a slot takes: from one up to [`Limit::MAX`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit(usize);

impl Limit {
    /// The smallest limit: a slot that takes nothing is no slot.
    pub const MIN: usize = 1;
    /// The largest limit (`i32::MAX`): no ceiling of ours, the largest number the settings kind of a whole number holds.
    pub const MAX: usize = 2_147_483_647;

    /// A limit of `max`; `None` for 0 and for more than [`Limit::MAX`].
    pub fn new(max: usize) -> Option<Self> {
        (Self::MIN..=Self::MAX).contains(&max).then_some(Self(max))
    }

    /// A limit known at compile time (a default).
    ///
    /// # Panics
    /// Outside `MIN..=MAX` (at compile time where it is used in a `const`).
    pub const fn of(max: usize) -> Self {
        assert!(
            max >= Self::MIN && max <= Self::MAX,
            "a limit is at least one and at most MAX"
        );
        Self(max)
    }

    /// The number.
    pub fn get(self) -> usize {
        self.0
    }
}

/// The declaration of a limit as a setting, as DATA: the host registers it with the settings store (kind: whole number
/// from [`Limit::MIN`] to [`Limit::MAX`], default [`LimitSetting::default`]) and hands the effective value to the block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitSetting {
    /// The key, `fs.<block>.<name>`.
    pub key: &'static str,
    /// The block's default (the lowest layer).
    pub default: Limit,
    /// The group the settings window shows it in.
    pub group: &'static str,
    /// The MF2 id of its name (`setting-label-` and the key with `.` as `-`).
    pub label_key: &'static str,
    /// The MF2 id of its description.
    pub about_key: &'static str,
}

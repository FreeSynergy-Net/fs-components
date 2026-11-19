//! The one rule for changing the profile index, used by every store (memory, service, database): [`index_step_ok`].
//!
//! The base value of a [`ProfileSet<u64>`] is the highest id ever issued. An id once issued is never issued again, so a
//! deleted profile's values in other programs' parts can never come back under another name.
//!
//! Pattern: pure function (Specification).

use std::collections::BTreeSet;

use fs_types::coded::Coded;

use crate::codes::{SettingCode, keys};
use crate::profile::ProfileSet;
use crate::profile_name::{ProfileName, ProfileRef};

fn invalid(name: &str) -> Coded<SettingCode> {
    Coded::new(SettingCode::Profile)
        .with_text(keys::CAUSE, "id-invalid")
        .with_text(keys::NAME, name)
}

fn ids(set: &ProfileSet<u64>) -> Vec<(&ProfileName, u64)> {
    set.names()
        .filter_map(|n| set.get(&ProfileRef::Named(n.clone())).map(|id| (n, *id)))
        .collect()
}

/// Whether `new` is a legal successor of `old`: the base (highest id ever) never falls; every id in `new` is either one
/// `old` has (a rename keeps it) or a new one above `old`'s base; no id is `0` or above `new`'s base; no id twice. An id
/// of `old` that is gone in `new` (deleted) never comes back.
///
/// # Errors
/// `profile{cause=id-invalid, name}` (the name of the profile that breaks the rule; empty for the base).
pub fn index_step_ok(
    old: &ProfileSet<u64>,
    new: &ProfileSet<u64>,
) -> Result<(), Coded<SettingCode>> {
    let (old_top, new_top) = (
        old.get(&ProfileRef::Base).copied().unwrap_or(0),
        new.get(&ProfileRef::Base).copied().unwrap_or(0),
    );
    if new_top < old_top {
        return Err(invalid(""));
    }
    let had: BTreeSet<u64> = ids(old).into_iter().map(|(_, id)| id).collect();
    let mut seen = BTreeSet::new();
    for (name, id) in ids(new) {
        let known = had.contains(&id);
        let fresh = id > old_top && id <= new_top;
        if id == 0 || !(known || fresh) || !seen.insert(id) {
            return Err(invalid(name.as_str()));
        }
    }
    Ok(())
}

//! `Why` from a command `State`: the one bridge between the command model and the disabled decorator.
//!
//! Pattern: Adapter (a `State` of `fs-components-commands` to a [`Why`]). `Why` lives in `fs-components-overlays`, which does not
//! know the commands, so this is a free function and not `Why::from_state` (an inherent impl cannot be added from another crate).

use fs_components_commands::{State, TextKey};

use crate::disabled::Why;

/// `Unsupported` becomes [`Why::Unsupported`] (the words are the decorator's, the reason key is not shown); `Disabled` becomes
/// [`Why::Now`] with `say(reason)`; an enabled command has no reason, so `None`.
pub fn why_from_state(state: &State, say: &dyn Fn(&TextKey) -> String) -> Option<Why> {
    match state {
        State::Disabled { reason } => Some(Why::Now(say(reason))),
        State::Unsupported { .. } => Some(Why::Unsupported),
        _ => None, // Enabled and any later state: nothing to say
    }
}

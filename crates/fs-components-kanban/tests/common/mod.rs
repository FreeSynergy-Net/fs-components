//! The host side of a gesture in the tests: the host finds the `[[action]]` of the gesture and maps it to the channel of the form
//! (`drop` -> `move`, `quick-add` -> `create`); the doubles then answer as they did for the channel.
//!
//! Pattern: Adapter (the test double of a host behind the port `FormWrite`).

use fs_components_core::form_write::Gesture;
use fs_components_core::form_write::trigger::DROP;

/// The channel of the form a gesture of the board goes to.
pub fn channel_of(gesture: Gesture<'_>) -> &'static str {
    if gesture.trigger == DROP {
        "move"
    } else {
        "create"
    }
}

//! The host side of a gesture in the tests, ONCE: the host finds the `[[action]]` of the gesture and maps it to the channel of the form
//! (`drop` -> `move`, `quick-add` -> `create`); the doubles then answer as they did for the channel. A gesture the table does not
//! know is a failure of the test (no catch-all channel): the board fired a gesture nobody bound.
//!
//! A double writes only `can` and `send` and adds `gestures_by_channel!();` inside its `impl FormWrite`.
//!
//! Pattern: Adapter (the test double of a host behind the port `FormWrite`), as a macro so that no double repeats it.

use fs_components_core::form_write::Gesture;
use fs_components_core::form_write::trigger::{DROP, QUICK_ADD};

/// The channel of the form a gesture of the board goes to.
///
/// # Panics
/// For every trigger but `drop` and `quick-add`: the board must not fire any other gesture here.
pub fn channel_of(gesture: Gesture<'_>) -> &'static str {
    match gesture.trigger {
        DROP => "move",
        QUICK_ADD => "create",
        other => panic!("the board fired a gesture no test host binds: {other}"),
    }
}

/// `may_fire` and `fire` of a double over its own `can` and `send` (the channel comes from [`channel_of`]).
macro_rules! gestures_by_channel {
    () => {
        fn may_fire(
            &self,
            gesture: ::fs_components_core::form_write::Gesture<'_>,
            record: Option<&str>,
        ) -> Result<(), ::fs_components_core::form_write::WriteRefused> {
            if self.can(common::channel_of(gesture), record) {
                Ok(())
            } else {
                Err(::fs_components_core::form_write::WriteRefused::new(
                    "locked",
                ))
            }
        }

        fn fire(
            &self,
            gesture: ::fs_components_core::form_write::Gesture<'_>,
            record: Option<&str>,
            args: &[::fs_components_core::form_write::WriteArg],
        ) -> Result<
            ::fs_components_core::form_write::WriteDone,
            ::fs_components_core::form_write::WriteRefused,
        > {
            self.send(common::channel_of(gesture), record, args)
        }
    };
}

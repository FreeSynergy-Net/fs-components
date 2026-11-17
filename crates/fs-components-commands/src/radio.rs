//! Radio groups and check marks: choosing one line switches the others of its group off.
//!
//! Pattern: State (the toggle of a command changes through these operations only), so "exactly one on" cannot be broken by a caller.

use crate::command::{CommandId, Toggle};
use crate::commands::Commands;

/// Why a choice did not change anything.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RadioError {
    /// No command with this id.
    Unknown(CommandId),
    /// The command is no member of a radio group (or, for a check mark, has no check).
    NotToggle(CommandId),
    /// The command is locked; a locked line cannot be chosen.
    Locked(CommandId),
}

impl Commands {
    /// Choose a radio line: it goes on, every other line of its group goes off.
    ///
    /// # Errors
    /// [`RadioError`] if the id is unknown, no radio line, or locked; then nothing changes.
    pub fn choose_radio(&mut self, id: &CommandId) -> Result<(), RadioError> {
        let group = match self.get(id) {
            None => return Err(RadioError::Unknown(id.clone())),
            Some(c) if c.state.is_locked() => return Err(RadioError::Locked(id.clone())),
            Some(c) => match &c.toggle {
                Toggle::Radio { group, .. } => group.clone(),
                _ => return Err(RadioError::NotToggle(id.clone())),
            },
        };
        for command in &mut self.list {
            if let Toggle::Radio { group: g, on } = &mut command.toggle
                && *g == group
            {
                *on = &command.id == id;
            }
        }
        Ok(())
    }

    /// Set a check line on or off.
    ///
    /// # Errors
    /// [`RadioError`] if the id is unknown, no check line, or locked; then nothing changes.
    pub fn set_check(&mut self, id: &CommandId, value: bool) -> Result<(), RadioError> {
        match self.list.iter_mut().find(|c| &c.id == id) {
            None => Err(RadioError::Unknown(id.clone())),
            Some(c) if c.state.is_locked() => Err(RadioError::Locked(id.clone())),
            Some(c) => match &mut c.toggle {
                Toggle::Check { on } => {
                    *on = value;
                    Ok(())
                }
                _ => Err(RadioError::NotToggle(id.clone())),
            },
        }
    }

    /// The lines of this radio group that are on, in order.
    #[must_use]
    pub fn radio_on(&self, group: &str) -> Vec<&CommandId> {
        self.list
            .iter()
            .filter(|c| matches!(&c.toggle, Toggle::Radio { group: g, on: true } if g == group))
            .map(|c| &c.id)
            .collect()
    }
}

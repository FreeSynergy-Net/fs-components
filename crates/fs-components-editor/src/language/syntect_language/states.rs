//! The states of one language: syntect's parser state cannot be written out, so a [`LineState`] is a number into this table.
//!
//! Pattern: Flyweight (interning) – equal states are stored once and get the same number, so two `LineState`s are equal
//! exactly when the parser states are; the table grows with the number of different states, not with the number of lines.

use std::sync::Mutex;

use syntect::parsing::{ParseState, ScopeStack};

use crate::language::LineState;

/// The table of the states a language has handed out.
#[derive(Default)]
pub struct States {
    table: Mutex<Vec<(ParseState, ScopeStack)>>,
}

impl States {
    /// The number of the state (a new one if it is new).
    pub fn put(&self, parse: ParseState, stack: ScopeStack) -> LineState {
        let mut table = self
            .table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = table
            .iter()
            .position(|(p, s)| *p == parse && *s == stack)
            .unwrap_or_else(|| {
                table.push((parse, stack));
                table.len() - 1
            });
        LineState((index as u64).to_le_bytes().to_vec())
    }

    /// The state with that number; `None` for bytes that this table did not hand out (the text then starts afresh).
    pub fn get(&self, state: &LineState) -> Option<(ParseState, ScopeStack)> {
        let bytes: [u8; 8] = state.0.as_slice().try_into().ok()?;
        let index = usize::try_from(u64::from_le_bytes(bytes)).ok()?;
        let table = self
            .table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        table.get(index).cloned()
    }
}

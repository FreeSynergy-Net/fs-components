//! The keys of a grid whose cells hold elements (variant `cells-hold`, WAI-ARIA APG "interactive cells"): who has the key.
//!
//! Pattern: State (a pure table: where the keys are x the key = what the grid does) – the one place where Enter and Escape are told
//! apart, so no key means two things.
//!
//! | The keys are | Key | The grid |
//! |---|---|---|
//! | on the cursor, a body cell | Enter | goes into the cell (`Entered`); Space still activates |
//! | on the cursor, a spanning cell (a head) | Enter | activates, as in a grid without holding |
//! | on the cursor | arrows, Home, End, Page keys, Space | as in every grid |
//! | on the cursor | Escape | nothing (the window's) |
//! | inside a cell | Escape | back to the cursor (`Left`) |
//! | inside a cell | every other key | nothing: the content has them (Enter, arrows between its elements, Space) |

use crate::keys::Key;

/// What the table says to do with one key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// The grid's normal rule (the move of the cursor, Space, the parent rows).
    Normal,
    /// Go into the cell under the cursor.
    Enter,
    /// Come back out of the cell.
    Leave,
    /// The key belongs to the content of the cell: the grid does nothing with it.
    Content,
}

/// What `key` does when the grid `holds` cells, the keys are `inside` a cell and the cursor stands on a `spanning` cell or not.
pub fn hold(holds: bool, inside: bool, spanning: bool, key: Key) -> Hold {
    match (holds, inside, key) {
        (true, true, Key::Escape) => Hold::Leave,
        (true, true, _) => Hold::Content,
        (true, false, Key::Enter) if !spanning => Hold::Enter,
        _ => Hold::Normal,
    }
}

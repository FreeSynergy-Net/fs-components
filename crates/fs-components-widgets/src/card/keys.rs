//! The keys of a card: who has the key, and what it means. One table, so no key means two things.
//!
//! Pattern: State (a pure table: the mode of the card x the key = what the card does). The words of the answer are those of
//! `sortable_list::Command`: the keyboard way to move a card is the sorter's (lift, walk, put down, give up), not a second one. The
//! container walks (it knows the cells); the card says only what the key meant.
//!
//! | Mode | Key | The card says |
//! |---|---|---|
//! | rest | Enter | `Activate` |
//! | rest | Space | `Sort(Toggle)` = lift |
//! | rest | Shift+F10, Menu key | `Menu` |
//! | rest | Escape, arrows, others | `Pass` (the container's, the window's) |
//! | held | arrows, Home, End, Page keys | `Sort(Up ...)` = the container chooses the next place |
//! | held | Enter, Space | `Sort(Toggle)` = put down |
//! | held | Escape | `Sort(Cancel)` = give up |
//! | held | Shift+F10, Menu key | `Pass` (a held card is not opened) |
//! | carried (by the pointer) | Escape | `Sort(Cancel)` |
//! | carried (by the pointer) | every other key | `Pass` |
//! | locked | Enter | `Activate` (a locked card still opens) |
//! | locked | Space | `Refuse` (the reason is said, nothing is lifted) |
//! | locked | Shift+F10, Menu key | `Menu`; every other key `Pass` |

use crate::keys::{Key, KeyEvent};
use crate::sortable_list::Command;

/// DOM code of the `F10` key.
const F10: u32 = 121;
/// DOM code of the context menu key.
const MENU_KEY: u32 = 93;

/// How the card stands, as far as the keys are concerned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Not lifted.
    #[default]
    Rest,
    /// Lifted by the keyboard: the container chooses the place, a key puts it down.
    Held,
    /// Carried by the pointer.
    Carried,
    /// May not be lifted.
    Locked,
}

/// What a key means to the card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Not the card's: the container or the window has it.
    Pass,
    /// The trigger `activate`.
    Activate,
    /// The trigger `menu`.
    Menu,
    /// The sorter's word: lift or put down (`Toggle`), give up (`Cancel`), walk to the next place.
    Sort(Command),
    /// A locked card was asked to be lifted: say why.
    Refuse,
}

fn is_menu(e: &KeyEvent) -> bool {
    e.key == Key::Other(MENU_KEY) || (e.shift && e.key == Key::Other(F10))
}

/// What `e` means to a card in `mode`.
pub fn step(mode: Mode, e: &KeyEvent) -> Step {
    if e.ctrl || e.alt {
        return Step::Pass;
    }
    match mode {
        Mode::Rest => match e.key {
            Key::Enter => Step::Activate,
            Key::Space => Step::Sort(Command::Toggle),
            _ if is_menu(e) => Step::Menu,
            _ => Step::Pass,
        },
        Mode::Held => match e.key {
            Key::Up => Step::Sort(Command::Up),
            Key::Down => Step::Sort(Command::Down),
            Key::Left => Step::Sort(Command::Left),
            Key::Right => Step::Sort(Command::Right),
            Key::Home => Step::Sort(Command::First),
            Key::End => Step::Sort(Command::Last),
            Key::PageUp => Step::Sort(Command::PageUp),
            Key::PageDown => Step::Sort(Command::PageDown),
            Key::Enter | Key::Space => Step::Sort(Command::Toggle),
            Key::Escape => Step::Sort(Command::Cancel),
            _ => Step::Pass,
        },
        Mode::Carried => match e.key {
            Key::Escape => Step::Sort(Command::Cancel),
            _ => Step::Pass,
        },
        Mode::Locked => match e.key {
            Key::Enter => Step::Activate,
            Key::Space => Step::Refuse,
            _ if is_menu(e) => Step::Menu,
            _ => Step::Pass,
        },
    }
}

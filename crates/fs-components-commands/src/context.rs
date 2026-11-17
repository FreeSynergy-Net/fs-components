//! The context menu as a pure model (plan `PLAN-components-menue`, M3): where it opens, what it acts on, its two stages, and the
//! state of one open menu.
//!
//! Pattern: State (`ContextState`) + Strategy (the rules for the place, the target and the stage are pure functions).
//! Nothing here knows Blinc: the frontend turns a right click, the menu key or Shift+F10 into a [`Trigger`], asks [`ContextState`],
//! and draws what it says. Rules:
//! - **Target = selection**: a press on a row that is part of the selection acts on the whole selection; a press on a row outside
//!   of it first makes that row the selection ([`target_for`]); a press on empty space acts on nothing (the program offers its
//!   `context:empty` arrangement then).
//! - **Two stages**: the first level is short and ends with the submenu *More options* (the key [`MORE_KEY`]). Shift+right click
//!   opens the second stage at once: the same lines with the content of *More options* in place of the submenu line ([`stages`]).
//! - **Keyboard**: the menu key and Shift+F10 open the menu at the focused element ([`Trigger::point`]), never at the pointer.
//! - **Once**: a chosen line closes the menu and runs once; a second press of a double click finds no menu and runs nothing.
//!   A locked line does not close the menu and does not run; it reports its reason. A press outside closes.

use crate::resolve::Resolved;

/// The key of the text of the submenu that holds the second stage ("More options").
pub const MORE_KEY: &str = "shell-context-more";

/// A point in the window, in pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// From the left.
    pub x: f32,
    /// From the top.
    pub y: f32,
}

/// The rectangle of an element in the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Area {
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// How the person asked for the menu.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trigger {
    /// A right click at a point; `shift` held opens the second stage at once.
    Pointer {
        /// Where the pointer was.
        at: Point,
        /// Was Shift held?
        shift: bool,
    },
    /// The menu key at the focused element.
    MenuKey {
        /// The element that has the keyboard.
        element: Area,
    },
    /// Shift+F10 at the focused element.
    ShiftF10 {
        /// The element that has the keyboard.
        element: Area,
    },
}

/// The first stage or the second (the whole menu).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The short menu with the submenu *More options*.
    First,
    /// All lines at once.
    Full,
}

/// The most a keyboard-opened menu hangs below the top of the element (a focused list must not push it out of the window).
const KEY_DROP: f32 = 32.0;
/// How far from the left edge of the element the keyboard-opened menu starts.
const KEY_INSET: f32 = 8.0;

impl Trigger {
    /// Where the menu starts: at the pointer; for a key a little inside the left edge of the focused element, at most
    /// [`KEY_DROP`] below its top.
    #[must_use]
    pub fn point(&self) -> Point {
        match *self {
            Self::Pointer { at, .. } => at,
            Self::MenuKey { element } | Self::ShiftF10 { element } => Point {
                x: element.x + KEY_INSET.min(element.width),
                y: element.y + KEY_DROP.min(element.height),
            },
        }
    }

    /// Which stage it opens: the second for Shift+right click, else the first.
    #[must_use]
    pub fn stage(&self) -> Stage {
        match self {
            Self::Pointer { shift: true, .. } => Stage::Full,
            _ => Stage::First,
        }
    }
}

/// What the menu acts on: a kind of object and the ids of the selected ones (as the program names them). Empty for empty space.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Target {
    /// The kind of object (`file`, `group`, ...); the arrangement is `context:<kind>`.
    pub kind: String,
    /// The selected objects, in the order of the selection.
    pub ids: Vec<String>,
}

/// The target of a press on `hit` (the row under the pointer or the focused row) with `selection` as it is. A row in the
/// selection acts on the whole selection; a row outside makes that row the selection; no row is empty space.
#[must_use]
pub fn target_for(kind: &str, selection: &[String], hit: Option<&str>) -> Target {
    let ids = match hit {
        None => Vec::new(),
        Some(row) if selection.iter().any(|s| s == row) => selection.to_vec(),
        Some(row) => vec![row.to_owned()],
    };
    Target {
        kind: kind.to_owned(),
        ids,
    }
}

/// The two stages of a context arrangement: `first` as given, `full` with the submenu *More options* dissolved into its lines
/// (a separator between them and the lines before it, if none stands there). Without such a submenu both are equal.
#[derive(Clone, Debug, PartialEq)]
pub struct Stages {
    /// The short menu.
    pub first: Vec<Resolved>,
    /// The whole menu.
    pub full: Vec<Resolved>,
}

impl Stages {
    /// Is there a second stage at all?
    #[must_use]
    pub fn is_two_stage(&self) -> bool {
        self.first != self.full
    }
}

/// Split resolved items into the two stages (see [`Stages`]).
#[must_use]
pub fn stages(items: &[Resolved]) -> Stages {
    let mut full: Vec<Resolved> = Vec::new();
    for item in items {
        match item {
            Resolved::Submenu {
                label, children, ..
            } if label.as_str() == MORE_KEY => {
                if !matches!(full.last(), None | Some(Resolved::Separator)) {
                    full.push(Resolved::Separator);
                }
                full.extend(children.iter().cloned());
            }
            other => full.push(other.clone()),
        }
    }
    Stages {
        first: items.to_vec(),
        full,
    }
}

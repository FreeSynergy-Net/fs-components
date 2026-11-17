//! The state of one open context menu (M3): closed, or open with a stage, a target and maybe one submenu.
//!
//! Pattern: State (a small state machine: `open`, `toggle_sub`, `choose`, `press_outside`, `escape`). The rules of "once", "locked
//! stays open" and "Escape closes the submenu first" are here, so the frontend only draws what this says (see `context.rs`).

use crate::command::{CommandId, TextKey};
use crate::context::{Point, Stage, Target, Trigger, stages};
use crate::resolve::Resolved;

/// What a press on a line led to.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Run this command on this target; the menu is closed.
    Run {
        /// The command.
        id: CommandId,
        /// What it acts on.
        target: Target,
    },
    /// The line is locked: nothing runs, the menu stays, the reason is shown.
    Locked {
        /// The command.
        id: CommandId,
        /// Why (key of the text).
        reason: TextKey,
    },
    /// No menu was open, or no such line: nothing happens (the second press of a double click ends here).
    Nothing,
}

struct Open {
    at: Point,
    stage: Stage,
    target: Target,
    items: Vec<Resolved>,
    sub: Option<usize>,
}

/// The state of the context menu of a window: closed, or open with a stage, a target and maybe one submenu.
#[derive(Default)]
pub struct ContextState {
    open: Option<Open>,
}

impl ContextState {
    /// Closed.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Open the menu for `trigger` on `target` with the items of the arrangement (`resolve`); another open menu is replaced.
    pub fn open(&mut self, trigger: &Trigger, target: Target, items: &[Resolved]) {
        let stage = trigger.stage();
        let split = stages(items);
        self.open = Some(Open {
            at: trigger.point(),
            stage,
            target,
            items: match stage {
                Stage::First => split.first,
                Stage::Full => split.full,
            },
            sub: None,
        });
    }

    /// Is a menu open?
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Where it starts (the point of the trigger), if open.
    #[must_use]
    pub fn at(&self) -> Option<Point> {
        self.open.as_ref().map(|o| o.at)
    }

    /// The stage it was opened in, if open.
    #[must_use]
    pub fn stage(&self) -> Option<Stage> {
        self.open.as_ref().map(|o| o.stage)
    }

    /// What it acts on, if open.
    #[must_use]
    pub fn target(&self) -> Option<&Target> {
        self.open.as_ref().map(|o| &o.target)
    }

    /// The lines of the first level, if open.
    #[must_use]
    pub fn items(&self) -> &[Resolved] {
        self.open.as_ref().map_or(&[], |o| &o.items)
    }

    /// The position of the line whose submenu is open.
    #[must_use]
    pub fn open_sub(&self) -> Option<usize> {
        self.open.as_ref().and_then(|o| o.sub)
    }

    /// Open the submenu of the line at `position`, or close it if it is the open one. `false` if no menu is open or the line
    /// has no submenu.
    pub fn toggle_sub(&mut self, position: usize) -> bool {
        let Some(open) = &mut self.open else {
            return false;
        };
        if !matches!(open.items.get(position), Some(Resolved::Submenu { .. })) {
            return false;
        }
        open.sub = if open.sub == Some(position) {
            None
        } else {
            Some(position)
        };
        true
    }

    /// Choose the line of this command (in the first level or in the open submenu).
    pub fn choose(&mut self, id: &CommandId) -> Choice {
        let Some(open) = &self.open else {
            return Choice::Nothing;
        };
        let Some(command) = find(&open.items, id) else {
            return Choice::Nothing;
        };
        if let Some(reason) = command.state.reason() {
            return Choice::Locked {
                id: id.clone(),
                reason: reason.clone(),
            };
        }
        let target = open.target.clone();
        self.open = None;
        Choice::Run {
            id: id.clone(),
            target,
        }
    }

    /// A press outside of the menu and its submenu: close. `true` if a menu was open.
    pub fn press_outside(&mut self) -> bool {
        self.open.take().is_some()
    }

    /// Escape: the submenu closes first, then the menu. `true` if something closed.
    pub fn escape(&mut self) -> bool {
        let Some(open) = &mut self.open else {
            return false;
        };
        if open.sub.take().is_some() {
            return true;
        }
        self.open = None;
        true
    }
}

fn find<'a>(items: &'a [Resolved], id: &CommandId) -> Option<&'a crate::Command> {
    items.iter().find_map(|item| match item {
        Resolved::Command(c) if &c.id == id => Some(c),
        Resolved::IconRow(row) => row.iter().find(|c| &c.id == id),
        Resolved::Submenu { children, .. } => find(children, id),
        _ => None,
    })
}

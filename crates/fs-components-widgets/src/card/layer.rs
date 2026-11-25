//! The ghost and the insertion line of a container: what the carried card looks like at the pointer and where it would land.
//!
//! Pattern: Template method (pure parts: the container calls them with its own origin, so a card looks the same in every container).
//! The container draws them (it knows the cell); the card only reports its state and the pointer. Both are laid out in the container's
//! coordinates: the caller subtracts the origin of its layer from the window coordinates the card gives.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::hit::Line;
use crate::answer::Reason;

/// The top left corner of a ghost of `size` centred under `pointer`.
pub fn ghost_origin(pointer: (f32, f32), size: (f32, f32)) -> (f32, f32) {
    (pointer.0 - size.0 / 2.0, pointer.1 - size.1 / 2.0)
}

/// `line` (window coordinates) in the coordinates of a layer whose top left corner lies at `origin` in the window.
pub fn within(line: Line, origin: (f32, f32)) -> Line {
    Line {
        x: line.x - origin.0,
        y: line.y - origin.1,
        w: line.w,
    }
}

/// The line where the card would land; red when the place does not take it. Does not take the pointer.
pub fn drop_line(id: &str, line: Line, blocked: bool) -> Div {
    let mut d = div()
        .id(id)
        .class("fs-card-drop-line")
        .absolute()
        .left(line.x)
        .top(line.y)
        .w(line.w)
        .pointer_events_none();
    if blocked {
        d = d.class("fs-card-drop-blocked");
    }
    d
}

/// The picture of the carried card under the pointer (`pointer` in the layer's coordinates, `size` = the card's size), with the reason
/// in the card's border and text when the place does not take it. Does not take the pointer.
pub fn ghost(
    id: &str,
    parts: Vec<Div>,
    pointer: (f32, f32),
    size: (f32, f32),
    blocked: Option<&Reason>,
) -> Div {
    let (x, y) = ghost_origin(pointer, size);
    let mut d = div()
        .id(id)
        .class("fs-card-ghost")
        .absolute()
        .left(x)
        .top(y)
        .w(size.0)
        .pointer_events_none();
    for part in parts {
        d = d.child(part);
    }
    if let Some(reason) = blocked {
        d = d.class("fs-card-ghost-blocked").child(
            text(reason.said().to_owned())
                .class("fs-card-ghost-reason")
                .id(format!("{id}-reason")),
        );
    }
    d
}

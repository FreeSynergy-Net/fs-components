//! Where the panel of a dropdown stands: the one rule for every menu that hangs on an anchor.
//!
//! Pattern: Strategy (a pure function over rectangles, no Blinc), so the rule is testable without a window.
//! Under the anchor if the panel fits there; else above it (flip); else on the side with more room, shortened to that room (the
//! panel then scrolls); never outside the window.

/// The margin kept between a panel and the edge of the window.
pub const MARGIN: f32 = 8.0;

/// The space between the anchor and the panel.
pub const GAP: f32 = 4.0;

/// A rectangle: left, top, width, height.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// Which edge of the anchor the panel lines up with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    /// Left edges together.
    #[default]
    Start,
    /// Right edges together.
    End,
}

/// The panel as placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// The panel's rectangle in the window. Its height is at most the wanted one; less means the panel scrolls.
    pub rect: Rect,
    /// The panel stands above the anchor.
    pub flipped: bool,
}

/// Place a panel of `want` (width, height) at `anchor` in a window of `window`. A window of zero size means "not known yet":
/// the panel then stands under the anchor as asked.
#[must_use]
pub fn place(anchor: Rect, want: (f32, f32), window: (f32, f32), align: Align) -> Placed {
    let (vw, vh) = window;
    let (mut width, mut height) = want;
    let left = match align {
        Align::Start => anchor.x,
        Align::End => anchor.x + anchor.width - width,
    };
    let below = anchor.y + anchor.height + GAP;
    if vw <= 0.0 || vh <= 0.0 {
        let rect = Rect {
            x: left,
            y: below,
            width,
            height,
        };
        return Placed {
            rect,
            flipped: false,
        };
    }
    width = width.min((vw - 2.0 * MARGIN).max(1.0));
    let x = left.min(vw - width - MARGIN).max(MARGIN);
    let room_below = vh - below - MARGIN;
    let room_above = anchor.y - GAP - MARGIN;
    let (y, flipped) = if height <= room_below {
        (below, false)
    } else if height <= room_above {
        (anchor.y - GAP - height, true)
    } else if room_below >= room_above {
        height = room_below.max(1.0);
        (below, false)
    } else {
        height = room_above.max(1.0);
        (anchor.y - GAP - height, true)
    };
    // An anchor that fills the window leaves no room on either side: hold the panel inside the window then.
    height = height.min((vh - 2.0 * MARGIN).max(1.0));
    let y = y.min(vh - height - MARGIN).max(MARGIN);
    Placed {
        rect: Rect {
            x,
            y,
            width,
            height,
        },
        flipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: (f32, f32) = (800.0, 500.0);

    fn at(x: f32, y: f32) -> Rect {
        Rect {
            x,
            y,
            width: 60.0,
            height: 28.0,
        }
    }

    #[test]
    fn under_the_anchor_when_there_is_room() {
        let p = place(at(20.0, 40.0), (220.0, 200.0), WINDOW, Align::Start);
        assert_eq!((p.rect.x, p.rect.y, p.flipped), (20.0, 72.0, false));
    }

    #[test]
    fn above_the_anchor_when_there_is_no_room_below() {
        let p = place(at(20.0, 450.0), (220.0, 200.0), WINDOW, Align::Start);
        assert_eq!((p.rect.y, p.rect.height, p.flipped), (246.0, 200.0, true));
    }

    #[test]
    fn held_in_the_window_and_shortened_when_neither_side_fits() {
        let p = place(at(20.0, 200.0), (220.0, 400.0), WINDOW, Align::Start);
        assert!(p.rect.height < 400.0);
        assert!(p.rect.y >= MARGIN && p.rect.y + p.rect.height <= 500.0 - MARGIN);
    }

    #[test]
    fn kept_inside_at_the_sides() {
        let right = place(at(780.0, 40.0), (220.0, 100.0), WINDOW, Align::Start);
        assert_eq!(right.rect.x + right.rect.width, 800.0 - MARGIN);
        let left = place(at(0.0, 40.0), (220.0, 100.0), WINDOW, Align::End);
        assert_eq!(left.rect.x, MARGIN);
    }

    #[test]
    fn an_unknown_window_leaves_the_place_as_asked() {
        let p = place(at(20.0, 450.0), (220.0, 200.0), (0.0, 0.0), Align::Start);
        assert_eq!((p.rect.y, p.flipped), (482.0, false));
    }
}

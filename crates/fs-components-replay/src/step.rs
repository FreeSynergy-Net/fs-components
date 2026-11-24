//! Our own step types: what a plan says, free of any `WebDriver` type.
//!
//! Pattern: Command (a step is data; `perform` executes it). Deviation: no undo.

/// A mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// Button 0.
    Left,
    /// Button 1.
    Middle,
    /// Button 2.
    Right,
}

/// A special key of the W3C table (normalised key values U+E000 and up) that a driver may know.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Named {
    /// U+E006, U+E007.
    Enter,
    /// U+E004.
    Tab,
    /// U+E00C.
    Escape,
    /// U+E003.
    Backspace,
    /// U+E00D.
    Space,
    /// U+E00E.
    PageUp,
    /// U+E00F.
    PageDown,
    /// U+E010.
    End,
    /// U+E011.
    Home,
    /// U+E012.
    Left,
    /// U+E013.
    Up,
    /// U+E014.
    Right,
    /// U+E015.
    Down,
    /// U+E017.
    Delete,
    /// U+E008, a modifier.
    Shift,
    /// U+E009, a modifier.
    Control,
    /// U+E00A, a modifier.
    Alt,
    /// U+E03D, a modifier.
    Meta,
}

/// A key: a printable character (typed as text) or a special key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    /// One printable character.
    Char(char),
    /// A special key.
    Named(Named),
}

/// Where a coordinate pair counts from.
#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
    /// The top left of the surface.
    Viewport,
    /// The pointer where it is now.
    Pointer,
    /// The centre of the element with this id (the `WebDriver` element reference holds the id of our UI).
    Element(String),
}

/// One action of one input source in one tick.
#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    /// Nothing; the time is the tick's.
    Pause,
    /// Move the pointer; with a duration it moves in frames on the way.
    Move {
        /// What the offset counts from.
        origin: Origin,
        /// Offset right.
        x: f32,
        /// Offset down.
        y: f32,
        /// Milliseconds the move takes.
        duration_ms: u32,
    },
    /// Press or release a button at the pointer.
    Button {
        /// Which button.
        button: Button,
        /// Pressed (true) or released.
        down: bool,
    },
    /// Press or release a key.
    Key {
        /// Which key.
        key: Key,
        /// Pressed (true) or released.
        down: bool,
    },
    /// Turn the wheel at a point.
    Wheel {
        /// What the point counts from.
        origin: Origin,
        /// Offset right.
        x: f32,
        /// Offset down.
        y: f32,
        /// Scroll right.
        dx: f32,
        /// Scroll down.
        dy: f32,
        /// Milliseconds the scroll takes.
        duration_ms: u32,
    },
}

/// What happens at the same time: the n-th action of every source. It lasts as long as its longest step.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    /// The steps, in the order of the sources.
    pub steps: Vec<Step>,
    /// Milliseconds the tick lasts (the largest duration of its steps).
    pub duration_ms: u32,
}

/// A whole input: the ticks in order. An empty plan is valid (it does nothing).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    /// The ticks in order.
    pub ticks: Vec<Tick>,
}

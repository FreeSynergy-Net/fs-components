//! The pointer gesture of the board: press, a few pixels, carry, release.
//!
//! Pattern: State machine, pure. The machine itself lives in `fs-components-widgets` (`card::gesture`, the one gesture of every card);
//! the board uses it with its own types: the card and the column are texts, a place is a [`Spot`]. Until the board is built on `grid`
//! and `card` (P5) both ways exist; then this file goes.

use crate::hit::Spot;
pub use fs_components_widgets::card::gesture::THRESHOLD;
use fs_components_widgets::card::gesture::{Gesture, Moved as Step, Release as Done};

/// Where the gesture is: the card and its column are texts, the place a [`Spot`].
pub type Drag = Gesture<String, String, Spot>;
/// What a pointer movement did.
pub type Moved = Step;
/// What a release came to.
pub type Release = Done<String, String, Spot>;

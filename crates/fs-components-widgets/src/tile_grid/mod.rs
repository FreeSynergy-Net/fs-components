//! `tile-grid`: tiles in rows that share their lines, numbered, each with a head (number, title), a body (the face or picture of the
//! record and its facts) and a foot (its state); as many tiles in a row as fit, the rest wraps like text.
//!
//! Pattern: Composite (grid of tiles of cards) + Adapter ([`form_view`]: the form `tiles` in, tiles out) + Abstract Factory (`View`).
//! Nothing new is drawn here: the rows are the `grid` of the lists (variant `headless`), every tile is the general `card` (variant
//! `shared-lines`), the face is `person-face`. The element knows no module; a person, a program or a file is only what the tile says.

pub mod form_view;
mod model;
mod tile;
mod view;

pub use model::Tile;
pub use view::{MIN_WIDTH, STYLES, TileGrid};

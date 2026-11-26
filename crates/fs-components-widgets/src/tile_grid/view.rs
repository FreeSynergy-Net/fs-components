//! The element: the tiles as the cells of a headless grid.
//!
//! Pattern: Composite + Strategy — the rows are the `grid` (one row per `n` tiles, `n` from the room), the strategy [`TileCells`] answers
//! "the card of this tile" for a cell; the last row may have fewer tiles, its empty cells stay empty.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_lists::grid::{CellAt, CellView, Column, Grid, Row};

use super::{Tile, tile};
use crate::craft::tile_columns;

/// The least width of a tile, in pixels: a row takes as many tiles as fit, the rest wraps.
pub const MIN_WIDTH: f32 = 240.0;

/// The style sheet of the tiles (the cards bring their own).
pub const STYLES: &str = r"
.fs-tile-grid { border-top-width: 1px; border-top-color: var(--border); border-left-width: 1px; border-left-color: var(--border); }
.fs-tile-grid .fs-grid { border-width: 0px; border-radius: 0px; background: var(--background); }
.fs-tile-grid .fs-grid-cell { display: flex; flex-direction: column; padding: 0px; border-width: 0px; }
.fs-tile-grid .fs-grid-cell-current { outline-width: 2px; outline-color: var(--border-focus); outline-offset: -2px; }
.fs-tile-grid .fs-card-shared { flex-grow: 1; }
.fs-tile-number { color: var(--text-secondary); font-size: var(--text-xs); font-weight: 700; }
.fs-tile-face { flex-shrink: 0; }
.fs-tile-facts { min-width: 0px; overflow: hidden; }
.fs-tile-fact-label { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-tile-fact { color: var(--text-primary); font-size: var(--text-sm); }
.fs-tile-state { color: var(--text-secondary); font-size: var(--text-xs); }
";

/// Tiles in rows that share their lines.
#[derive(Clone, Debug)]
pub struct TileGrid {
    tiles: Arc<Vec<Tile>>,
    id: String,
    least: f32,
}

impl TileGrid {
    /// A grid of these tiles; ids: `<id>` (the grid), `<id>-tile-<key>` (a card).
    pub fn new(tiles: Vec<Tile>) -> Self {
        Self {
            tiles: Arc::new(tiles),
            id: "tiles".to_owned(),
            least: MIN_WIDTH,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The least width of a tile in pixels (below it the row takes fewer tiles).
    #[must_use]
    pub fn least(mut self, px: f32) -> Self {
        self.least = px.max(1.0);
        self
    }

    /// How many tiles a row takes in `room` pixels, and how wide each one is.
    pub fn columns(&self, room: f32) -> (usize, f32) {
        tile_columns(room, self.least)
    }

    /// The grid of the tiles for `room` pixels (a handle: the keys go to [`Grid::press`]).
    pub fn grid(&self, room: f32) -> Grid {
        let (cols, width) = self.columns(room);
        let columns: Vec<Column> = (0..cols).map(|c| Column::new(col_key(c), "")).collect();
        let rows: Vec<Row> = self
            .tiles
            .chunks(cols)
            .enumerate()
            .map(|(r, _)| Row::new(row_key(r), vec![String::new(); cols]))
            .collect();
        Grid::new(columns, &rows)
            .id(&self.id)
            .headless()
            .column_width(width)
            .cells(Arc::new(TileCells {
                tiles: Arc::clone(&self.tiles),
                cols,
                id: self.id.clone(),
            }))
    }

    /// The element for `room` pixels: the frame that closes the top and the left line, the grid in it.
    pub fn build(&self, room: f32) -> Div {
        self.frame(room, &self.grid(room))
    }

    /// The frame around a grid made by [`TileGrid::grid`].
    pub fn frame(&self, room: f32, grid: &Grid) -> Div {
        div()
            .class("fs-tile-grid")
            .w(room)
            .child(grid.clone().build())
    }
}

fn row_key(r: usize) -> String {
    format!("r{r}")
}

fn col_key(c: usize) -> String {
    format!("c{c}")
}

/// The strategy: the card of the tile at a cell.
struct TileCells {
    tiles: Arc<Vec<Tile>>,
    cols: usize,
    id: String,
}

impl CellView for TileCells {
    fn cell(&self, at: &CellAt<'_>) -> Option<Div> {
        let r: usize = at.row.strip_prefix('r')?.parse().ok()?;
        let c: usize = at.col.strip_prefix('c')?.parse().ok()?;
        let tile = self.tiles.get(r * self.cols + c)?;
        Some(tile::element(
            tile,
            &format!("{}-tile-{}", self.id, tile.key),
        ))
    }
}

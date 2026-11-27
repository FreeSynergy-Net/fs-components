//! The board as a grid: the columns are the columns of the grid, a plain board is ONE row of cells, a board in lanes is a treegrid
//! with a spanning row per lane and under it the row of cells of that lane. Folding a lane is the grid's (an arrow key, a click on
//! the head of the lane); the cell holds the cards of that column (and lane) and the foot.
//!
//! Pattern: Strategy (`BoardCells` is the `CellView` the grid asks "what is in this cell?") + Facade (one call, `board_grid`, makes the
//! grid for the shape of the data and keeps it over redraws: the cursor and the lanes the reader closed stay).
//! Every lane the host named is a row, empty or not; the heads of the columns are the head row of the grid.

use std::collections::HashSet;
use std::sync::{Arc, PoisonError};

use blinc_layout::div::Div;
use fs_components_lists::grid::{CellAt, CellView, Column, Grid, Row};

use crate::BoardState;
use crate::bench::BoardBench;
use crate::sizes::{COLUMN_W, PAD};
use crate::swimlanes::Lane;

/// The key of the one row of a plain board.
const PLAIN_ROW: &str = "board";
/// The prefix of the key of the row of cells under a lane.
const CELLS: &str = "cells~";

/// What the cells of the board hold, asked by the grid when it draws.
struct BoardCells {
    bench: BoardBench,
    state: Arc<BoardState>,
    lanes: Option<Arc<Vec<Lane>>>,
}

impl CellView for BoardCells {
    fn head(&self, col: &str) -> Option<Div> {
        let n = self.state.heads().iter().position(|h| h.key == col)?;
        Some(self.bench.head_row(&self.state, n))
    }

    fn cell(&self, at: &CellAt<'_>) -> Option<Div> {
        let n = self.state.heads().iter().position(|h| h.key == at.col)?;
        let lane = match &self.lanes {
            None => None,
            Some(lanes) => {
                let key = at.row.strip_prefix(CELLS)?;
                Some(lanes.iter().find(|l| l.key == key)?)
            }
        };
        Some(self.bench.cell(&self.state, lane, n))
    }
}

impl BoardBench {
    /// The grid for this state: made again only when the columns or the lanes are other than at the last draw (then the lanes the
    /// reader closed stay closed); otherwise the same one, with the view over the state of now.
    pub(crate) fn board_grid(
        &self,
        state: &Arc<BoardState>,
        lanes: Option<Arc<Vec<Lane>>>,
    ) -> Grid {
        let columns: Vec<Column> = state
            .heads()
            .iter()
            .map(|h| Column::new(&h.key, &h.title))
            .collect();
        let n = columns.len();
        let shape = format!(
            "{:?}|{:?}",
            columns.iter().map(|c| &c.key).collect::<Vec<_>>(),
            lanes
                .iter()
                .flat_map(|l| l.iter().map(|l| (&l.key, &l.label, l.cards.len())))
                .collect::<Vec<_>>()
        );
        let mut cache = self.grid.lock().unwrap_or_else(PoisonError::into_inner);
        let grid = match cache.as_ref() {
            Some((had, grid)) if *had == shape => grid.clone(),
            old => {
                let closed: HashSet<String> = lanes
                    .iter()
                    .flat_map(|l| l.iter())
                    .filter(|l| old.is_some_and(|(_, g)| !g.is_open(&l.key)))
                    .map(|l| l.key.clone())
                    .collect();
                let rows = match &lanes {
                    None => vec![Row::new(PLAIN_ROW, vec![String::new(); n])],
                    Some(lanes) => lanes
                        .iter()
                        .map(|lane| lane_row(lane, n, !closed.contains(&lane.key)))
                        .collect(),
                };
                let grid = if lanes.is_some() {
                    Grid::treegrid(columns, &rows)
                } else {
                    Grid::new(columns, &rows)
                };
                let grid = grid
                    .id(&format!("{}-grid", self.id))
                    .name(self.title.as_deref().unwrap_or("Board"))
                    .column_width(COLUMN_W + 2.0 * PAD);
                *cache = Some((shape, grid.clone()));
                grid
            }
        };
        drop(cache);
        let (this, lane_keys): (Self, HashSet<String>) = (
            self.clone(),
            lanes
                .iter()
                .flat_map(|l| l.iter().map(|l| l.key.clone()))
                .collect(),
        );
        let (keys, enter, lanes_in) = (self.clone(), self.clone(), lanes.clone());
        grid.cells(Arc::new(BoardCells {
            bench: self.clone(),
            state: state.clone(),
            lanes,
        }))
        .hold_cells()
        // Outside a cell the arrows are the grid's (from cell to cell); Enter goes in; inside, the keys are the board's (the cards,
        // the held card, Shift+arrow), and Escape that the board has no use for comes back out (the grid's own rule).
        .first_key(move |event| {
            keys.cancel_carry(event)
                || (keys.cached_grid().is_some_and(|g| g.inside()) && keys.press_key(event))
        })
        .on_enter(move |row, col| enter.enter_cell(row, col, lanes_in.as_deref()))
        .on_activate(move |row, _| {
            // Enter or a click on the head of a lane folds it (the grid itself has only the arrow keys for that).
            if lane_keys.contains(row) {
                let grid = this
                    .grid
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .as_ref()
                    .map(|(_, g)| g.clone());
                if let Some(grid) = grid {
                    grid.set_open(row, !grid.is_open(row));
                }
            }
        })
    }
}

impl BoardBench {
    /// The grid as it was made last.
    pub(crate) fn cached_grid(&self) -> Option<Grid> {
        let cache = self.grid.lock().unwrap_or_else(PoisonError::into_inner);
        cache.as_ref().map(|(_, g)| g.clone())
    }

    /// The keys went into the cell `row` x `col`: the cursor of the cards stands on its first card (the handover grid -> card).
    fn enter_cell(&self, row: &str, col: &str, lanes: Option<&Vec<Lane>>) {
        {
            let mut state = self.lock();
            let lane = lanes.and_then(|l| {
                let key = row.strip_prefix(CELLS)?;
                l.iter().find(|l| l.key == key)
            });
            let first = state
                .lanes()
                .iter()
                .find(|l| l.key == col)
                .and_then(|l| {
                    l.items
                        .iter()
                        .find(|i| lane.is_none_or(|lane| lane.cards.contains(&i.key)))
                })
                .map(|i| i.key.clone());
            if let Some(first) = first {
                state.sorter.set_cursor(&first);
            }
        }
        self.refresh();
    }
}

/// The spanning row of a lane ("Ada (2)") with the row of its cells under it.
fn lane_row(lane: &Lane, columns: usize, open: bool) -> Row {
    let head = Row::spanning(&lane.key, format!("{} ({})", lane.label, lane.cards.len()))
        .with_children(vec![Row::new(
            format!("{CELLS}{}", lane.key),
            vec![String::new(); columns],
        )]);
    if open { head.opened() } else { head }
}

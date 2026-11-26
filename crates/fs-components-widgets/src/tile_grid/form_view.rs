//! The tiles as a [`FormView`] of the form `tiles@^1.0.0`: every record is a tile. `label` names it, `order` numbers it (else its
//! place), `preview` is its picture, `state` is its foot, the facts are `facts` (1.1.0: each under the word of its field, found by PATH,
//! so a fact without a value shifts nothing and a fact with two values shows both under one label) and, as in 1.0.0, `kind` and `time`
//! as plain lines. An older Minor is read the same way (tolerant reader).
//!
//! The picture is drawn as a face only when the block of the type of `preview` says so (`person-face`, chosen by the catalog from the
//! contract) or says nothing; a preview of another block is not drawn here.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (`FormInput` in, [`Tile`]s out); the heads and the section are those of the detail page.

use std::sync::{Mutex, PoisonError};

use blinc_layout::div::Div;
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::keys::{Key, KeyEvent};
use fs_components_lists::grid::Grid;

use super::{Tile, TileGrid};
use crate::detail_page::Field;
use crate::detail_page::form_view::{heads, section};

/// The widget this view is (`widget` of the block `tile-grid`).
pub const WIDGET: &str = "TileGrid";

/// The channel of `tiles@1.1.0` that carries the facts.
pub const FACTS: &str = "facts";

/// The channel of the picture and the block that draws it as a face.
const PREVIEW: &str = "preview";
const FACE_BLOCK: &str = "person-face";

/// The tiles of `input`, one per record, in order.
pub fn tiles(input: &FormInput, say: &dyn Fn(&str) -> String) -> Vec<Tile> {
    let block = input
        .channel(PREVIEW)
        .and_then(|c| c.fields.first())
        .map_or("", |f| f.block.as_str());
    let face = block.is_empty() || block == FACE_BLOCK;
    let heads = heads(input, FACTS, say);
    input
        .rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut facts = section(row, FACTS, &heads).fields;
            // tiles@1.0.0: a kind and a time stand as plain lines
            facts.extend(
                ["kind", "time"]
                    .iter()
                    .filter_map(|c| row.get(c))
                    .map(|v| Field::new("", v)),
            );
            let number = row
                .get("order")
                .map_or_else(|| format!("{:02}", i + 1), str::to_owned);
            let mut tile = Tile::new(&row.key, row.get("label").unwrap_or(&row.key))
                .numbered(number)
                .with_facts(facts)
                .with_state(row.get("state").unwrap_or_default());
            if face {
                if let Some(image) = row.get(PREVIEW) {
                    tile = tile.with_image(image);
                }
            } else {
                tile = tile.without_face();
            }
            tile
        })
        .collect()
}

/// The view of the block `tile-grid`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        let grid = TileGrid::new(tiles(&input, &*ctx.say));
        Box::new(MountedTiles(Mutex::new(Held {
            grid,
            ctx,
            drawn: None,
        })))
    }
}

/// The tiles, the context and the grid last drawn, so the cursor stays where it is: with the room it was drawn for (`None` = the
/// tiles changed since, draw again).
struct Held {
    grid: TileGrid,
    ctx: DrawCtx,
    drawn: Option<(Option<f32>, Grid)>,
}

struct MountedTiles(Mutex<Held>);

impl Mounted for MountedTiles {
    fn build(&self, width: f32, _height: f32) -> Div {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let fresh = match &held.drawn {
            Some((Some(room), grid)) if (room - width).abs() < f32::EPSILON => grid.clone(),
            old => {
                let fresh = held.grid.grid(width);
                if let Some((row, col)) = old.as_ref().and_then(|(_, g)| g.cursor()) {
                    fresh.go_to(&row, &col);
                }
                fresh
            }
        };
        held.drawn = Some((Some(width), fresh.clone()));
        held.grid.frame(width, &fresh)
    }

    fn refresh(&self, input: FormInput) {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        held.grid = TileGrid::new(tiles(&input, &*held.ctx.say));
        held.drawn = held.drawn.take().map(|(_, grid)| (None, grid));
    }

    fn key(&self, key: Key) -> bool {
        let held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        held.drawn
            .as_ref()
            .is_some_and(|(_, grid)| grid.press(KeyEvent::new(key)))
    }
}

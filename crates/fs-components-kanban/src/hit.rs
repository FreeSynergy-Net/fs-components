//! Where a pointer is on the board: which column, and between which cards.
//!
//! Pattern: none - pure geometry. Blinc sends a drag only to the element that was pressed and has no drop-target event, so the
//! board knows its own rectangles. They are exact because a column and a card have fixed sizes here (a card is `CARD_H` high
//! whatever its text; a long title is cut), so nothing needs measuring. All values are in the coordinates of the columns row
//! (its top left corner is 0, 0), the pointer is taken there by the caller.

/// Width of a column in pixels.
pub const COLUMN_W: f32 = 288.0;
/// Space between two columns.
pub const COLUMN_GAP: f32 = 16.0;
/// Space around the columns row.
pub const PAD: f32 = 16.0;
/// Height of a column head (name, count, menu).
pub const HEAD_H: f32 = 48.0;
/// Height of a card.
pub const CARD_H: f32 = 84.0;
/// Space between two cards.
pub const CARD_GAP: f32 = 10.0;
/// Height of the foot of a column (the quick entry).
pub const FOOT_H: f32 = 48.0;

/// A place between cards: the column (by number) and the index among the shown cards without the one carried.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spot {
    /// Which column.
    pub column: usize,
    /// Which index.
    pub index: usize,
}

/// The columns with their shown cards, as rectangles.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Geometry {
    columns: Vec<Vec<String>>,
}

impl Geometry {
    /// The rectangles for columns with these shown cards (keys, in order).
    pub fn new(columns: Vec<Vec<String>>) -> Self {
        Self { columns }
    }

    /// The number of columns.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// No columns.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Left edge of column `n`.
    #[allow(clippy::cast_precision_loss)] // a few dozen columns
    pub fn column_x(&self, n: usize) -> f32 {
        PAD + n as f32 * (COLUMN_W + COLUMN_GAP)
    }

    /// Top edge of the card at `index` of the full list of a column (also while one is carried).
    #[allow(clippy::cast_precision_loss)] // a few thousand cards at most
    pub fn card_y(index: usize) -> f32 {
        PAD + HEAD_H + index as f32 * (CARD_H + CARD_GAP)
    }

    /// The rectangle (x, y, width, height) of a card, if it is shown.
    pub fn card_rect(&self, key: &str) -> Option<(f32, f32, f32, f32)> {
        self.columns.iter().enumerate().find_map(|(n, cards)| {
            cards
                .iter()
                .position(|c| c == key)
                .map(|i| (self.column_x(n), Self::card_y(i), COLUMN_W, CARD_H))
        })
    }

    /// The column under `x` (both edges belong to the column; the gap between two columns belongs to none).
    pub fn column_at(&self, x: f32) -> Option<usize> {
        (0..self.columns.len()).find(|&n| {
            let left = self.column_x(n);
            x >= left && x <= left + COLUMN_W
        })
    }

    /// The place under a pointer at `x`, `y`, when the card `skip` is carried: the column under `x` (none in a gap or outside),
    /// and the index among its shown cards without `skip`: before the first card whose middle is below `y`. A card exactly at its
    /// middle counts as above (the pointer must be past the middle to go after it).
    #[allow(clippy::cast_precision_loss)] // a few thousand cards at most
    pub fn spot(&self, x: f32, y: f32, skip: Option<&str>) -> Option<Spot> {
        let column = self.column_at(x)?;
        let shown = self.columns[column]
            .iter()
            .filter(|c| Some(c.as_str()) != skip);
        let index = shown
            .enumerate()
            .take_while(|(i, _)| Self::card_y(*i) + CARD_H / 2.0 < y)
            .count();
        Some(Spot { column, index })
    }

    /// The key of the shown card before `spot`, without `skip`: what the program is told as `after`.
    pub fn after(&self, spot: Spot, skip: Option<&str>) -> Option<String> {
        let cards = self.columns.get(spot.column)?;
        cards
            .iter()
            .filter(|c| Some(c.as_str()) != skip)
            .nth(spot.index.checked_sub(1)?)
            .cloned()
    }

    /// The y of the line that shows where a card would land at `spot` (between two cards, or under the last).
    #[allow(clippy::cast_precision_loss)] // a few thousand cards at most
    pub fn line_y(spot: Spot) -> f32 {
        PAD + HEAD_H + spot.index as f32 * (CARD_H + CARD_GAP) - CARD_GAP / 2.0
    }

    /// The size of the whole row of columns, with the tallest column and the room for the foot.
    #[allow(clippy::cast_precision_loss)] // a few thousand cards at most
    pub fn size(&self) -> (f32, f32) {
        let n = self.columns.len().max(1) as f32;
        let tallest = self.columns.iter().map(Vec::len).max().unwrap_or(0);
        (
            PAD * 2.0 + n * COLUMN_W + (n - 1.0) * COLUMN_GAP,
            Self::card_y(tallest) + FOOT_H + PAD,
        )
    }
}

impl fs_components_widgets::card::gesture::Over<String, Spot> for Geometry {
    fn place(&self, x: f32, y: f32, skip: Option<&String>) -> Option<Spot> {
        self.spot(x, y, skip.map(String::as_str))
    }
}

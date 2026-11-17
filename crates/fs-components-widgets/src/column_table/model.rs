//! The table of columns as data: rows, the edits, the check and what it says about saving.
//!
//! Pattern: State, pure - no Blinc. The table knows no program: a row's `status` and `stage` are opaque texts the program
//! understands (a status key, a stage name); the program's [`ColumnCheck`] says what is wrong, and saving is **blocked with the
//! reason** while anything is (never a dead button without a word). A row with cards cannot be removed without saying where its
//! cards go.

/// One column as a row of the table.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnRow {
    /// Identity (stays when the name changes).
    pub key: String,
    /// The name.
    pub name: String,
    /// The program's status this column stands for (opaque; "" = none).
    pub status: String,
    /// The stage within the status (opaque; "" = the basic column).
    pub stage: String,
    /// The limit; `None` = no limit.
    pub wip: Option<u32>,
    /// How many cards it holds now.
    pub cards: usize,
}

impl ColumnRow {
    /// A row with a key and a name.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            status: String::new(),
            stage: String::new(),
            wip: None,
            cards: 0,
        }
    }

    /// With the status.
    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    /// With the stage.
    #[must_use]
    pub fn stage(mut self, stage: impl Into<String>) -> Self {
        self.stage = stage.into();
        self
    }

    /// With a limit.
    #[must_use]
    pub fn wip(mut self, wip: u32) -> Self {
        self.wip = Some(wip);
        self
    }

    /// With a count of cards.
    #[must_use]
    pub fn cards(mut self, cards: usize) -> Self {
        self.cards = cards;
        self
    }
}

/// Something wrong with the table, said by the program (a status without a column) or by the table (a name that is empty).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// The row it is about; `None` = the table as a whole.
    pub row: Option<usize>,
    /// The sentence the reader sees.
    pub text: String,
}

/// The program's check of the whole table.
pub trait ColumnCheck: Send + Sync {
    /// What is wrong with these rows; empty = fine.
    fn check(&self, rows: &[ColumnRow]) -> Vec<Problem>;
}

/// Nothing is wrong, whatever the rows say.
#[derive(Clone, Copy, Debug, Default)]
pub struct AllowAll;

impl ColumnCheck for AllowAll {
    fn check(&self, _rows: &[ColumnRow]) -> Vec<Problem> {
        Vec::new()
    }
}

/// Why a row cannot be removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveError {
    /// No such row.
    NoSuchRow,
    /// It holds cards: say where they go (`Some(target row)`).
    NeedsTarget(usize),
    /// The target is the row itself, or does not exist.
    BadTarget,
    /// It is the last one.
    LastRow,
}

/// May the table be saved now?
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Save {
    /// Yes.
    pub allowed: bool,
    /// If not: why (the first problem, in the program's words).
    pub reason: Option<String>,
}

/// The text of the limit cell: empty = no limit, digits = the limit; anything else is not a limit.
///
/// # Errors
/// `()` for text that is neither empty nor a whole number.
#[allow(clippy::result_unit_err)] // the one reason is "not a number"
pub fn parse_wip(text: &str) -> Result<Option<u32>, ()> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(None);
    }
    t.parse::<u32>().map(Some).map_err(|_| ())
}

/// The table of columns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rows {
    rows: Vec<ColumnRow>,
    added: usize,
}

impl Rows {
    /// A table of these rows.
    pub fn new(rows: Vec<ColumnRow>) -> Self {
        Self { rows, added: 0 }
    }

    /// The rows in order.
    pub fn rows(&self) -> &[ColumnRow] {
        &self.rows
    }

    /// Move row `from` to place `to` (the drag of the handle, or the row menu's up and down). False if either does not exist or they are the same.
    pub fn move_row(&mut self, from: usize, to: usize) -> bool {
        if from == to || from >= self.rows.len() || to >= self.rows.len() {
            return false;
        }
        let row = self.rows.remove(from);
        self.rows.insert(to, row);
        true
    }

    /// A new empty row at the end with a fresh key (`new-1`, `new-2`, ...); returns its number.
    pub fn add(&mut self, name: &str) -> usize {
        self.added += 1;
        self.rows
            .push(ColumnRow::new(format!("new-{}", self.added), name));
        self.rows.len() - 1
    }

    /// Remove row `n`; a row with cards needs `into`, the row that takes the cards (they are added to its count).
    ///
    /// # Errors
    /// [`RemoveError`].
    pub fn remove(&mut self, n: usize, into: Option<usize>) -> Result<(), RemoveError> {
        let Some(row) = self.rows.get(n) else {
            return Err(RemoveError::NoSuchRow);
        };
        if self.rows.len() == 1 {
            return Err(RemoveError::LastRow);
        }
        let cards = row.cards;
        if cards > 0 {
            let Some(target) = into else {
                return Err(RemoveError::NeedsTarget(cards));
            };
            if target == n || target >= self.rows.len() {
                return Err(RemoveError::BadTarget);
            }
            self.rows[target].cards += cards;
        }
        self.rows.remove(n);
        Ok(())
    }

    /// Rename row `n`.
    pub fn set_name(&mut self, n: usize, name: &str) {
        if let Some(r) = self.rows.get_mut(n) {
            name.trim().clone_into(&mut r.name);
        }
    }

    /// Set the status of row `n`.
    pub fn set_status(&mut self, n: usize, status: &str) {
        if let Some(r) = self.rows.get_mut(n) {
            status.clone_into(&mut r.status);
        }
    }

    /// Set the stage of row `n`.
    pub fn set_stage(&mut self, n: usize, stage: &str) {
        if let Some(r) = self.rows.get_mut(n) {
            stage.trim().clone_into(&mut r.stage);
        }
    }

    /// Set the limit of row `n` from the text of the cell; false (and nothing changed) if the text is not a number.
    pub fn set_wip(&mut self, n: usize, text: &str) -> bool {
        match (parse_wip(text), self.rows.get_mut(n)) {
            (Ok(wip), Some(r)) => {
                r.wip = wip;
                true
            }
            _ => false,
        }
    }

    /// Everything wrong: the table's own (an empty name) and the program's.
    pub fn problems(&self, check: &dyn ColumnCheck) -> Vec<Problem> {
        let mut out: Vec<Problem> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.name.trim().is_empty())
            .map(|(n, _)| Problem {
                row: Some(n),
                text: "A column needs a name.".to_owned(),
            })
            .collect();
        out.extend(check.check(&self.rows));
        out
    }

    /// May it be saved? Not while anything is wrong; the reason is the first problem.
    pub fn save(&self, check: &dyn ColumnCheck) -> Save {
        match self.problems(check).into_iter().next() {
            Some(p) => Save {
                allowed: false,
                reason: Some(p.text),
            },
            None => Save {
                allowed: true,
                reason: None,
            },
        }
    }
}

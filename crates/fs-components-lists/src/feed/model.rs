//! The pure state of a feed.
//!
//! Pattern: State — how many articles, where the cursor is, whether more are coming; no element, no callback.

use crate::keys::Key;

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the feed.
    Ignored,
    /// The cursor moved to this article.
    Moved(usize),
}

/// The count, the cursor and the busy mark.
#[derive(Clone, Debug, Default)]
pub struct State {
    count: usize,
    total: Option<usize>,
    cursor: Option<usize>,
    busy: bool,
}

impl State {
    /// A feed of `count` articles; the cursor starts on the first.
    pub fn new(count: usize) -> Self {
        Self {
            count,
            total: None,
            cursor: (count > 0).then_some(0),
            busy: false,
        }
    }

    /// The articles that are there.
    pub fn count(&self) -> usize {
        self.count
    }

    /// More articles arrived at the end.
    pub fn grow(&mut self, by: usize) {
        self.count += by;
        if self.cursor.is_none() && self.count > 0 {
            self.cursor = Some(0);
        }
    }

    /// The size of the whole feed when the caller knows it (`aria-setsize`); `None`: not known (`-1`).
    pub fn set_total(&mut self, total: Option<usize>) {
        self.total = total;
    }

    /// The `aria-setsize` to tell: the total, or `-1` while it is not known.
    pub fn set_size(&self) -> i64 {
        self.total
            .map_or(-1, |t| i64::try_from(t.max(self.count)).unwrap_or(i64::MAX))
    }

    /// Whether more articles are being loaded (`aria-busy`).
    pub fn busy(&self) -> bool {
        self.busy
    }

    /// Say that more articles are being loaded.
    pub fn set_busy(&mut self, busy: bool) {
        self.busy = busy;
    }

    /// The article the cursor is on.
    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    /// Put the cursor on an article (the pointer's way).
    pub fn go(&mut self, article: usize) -> bool {
        if article < self.count {
            self.cursor = Some(article);
            return true;
        }
        false
    }

    /// What a key does.
    pub fn press(&mut self, key: Key) -> Outcome {
        let Some(at) = self.cursor else {
            return Outcome::Ignored;
        };
        let to = match key {
            Key::PageDown if at + 1 < self.count => at + 1,
            Key::PageUp if at > 0 => at - 1,
            _ => return Outcome::Ignored,
        };
        self.cursor = Some(to);
        Outcome::Moved(to)
    }
}

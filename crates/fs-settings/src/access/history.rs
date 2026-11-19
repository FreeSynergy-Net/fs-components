//! The page of the instance log a restricted actor gets, and the limit of one call ([`HistoryPage`], [`MAX_LINES_READ`]).
//!
//! A call of [`super::Surface::history`] skips the lines of hidden keys until the page is full. With a log of very many
//! hidden lines that would read the whole store for one call, so a call reads at most [`MAX_LINES_READ`] lines of the
//! store. When it stops there with a page that is not full, the page says so ([`HistoryPage::cut`]): a short page then
//! does not look like the end of the log.
//!
//! The pointer to go on is the id of the last line of the page, as ever: it is the id of a VISIBLE line, never of a
//! line that was only read and skipped (that id would tell a bot when a hidden key changed). The honest limit: a run of
//! at least `MAX_LINES_READ` hidden lines right after the last visible one is not passed by this pointer (the next call
//! reads the same lines again and cuts again, with an empty page). Only a person can make such a run (a bot cannot write
//! the key), and the bot learns no more than that a long run of lines it cannot see lies ahead: it is told "cut", never
//! how many lines or when.
//!
//! Pattern: Proxy (the filtered, bounded read of [`Admin::log`], in blocks).

use std::ops::Deref;

use fs_types::ObjectId;

use crate::admin::{Admin, InstanceLogEntry};

/// The most lines of the store one call reads (also the hidden ones): it bounds the work of a call to the store.
pub const MAX_LINES_READ: usize = 4096;
/// The lines the store is asked for at a time.
const BLOCK: u32 = 64;

/// The lines of one page of the history, and whether the call stopped at the read limit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryPage {
    /// The lines of the page, oldest first (all of them lines the actor can see).
    pub lines: Vec<InstanceLogEntry>,
    /// The call read [`MAX_LINES_READ`] lines and the page is not full: the log MAY go on behind the last line. Ask again
    /// with the id of the last line. `false` on a short page is the real end.
    pub cut: bool,
}

impl Deref for HistoryPage {
    type Target = [InstanceLogEntry];

    fn deref(&self) -> &[InstanceLogEntry] {
        &self.lines
    }
}

/// The page after `after` with up to `size` lines that `skip` does not hide.
pub(super) fn read(
    admin: &Admin,
    after: Option<ObjectId>,
    size: u32,
    skip: impl Fn(&str) -> bool,
) -> HistoryPage {
    let want = size as usize;
    let mut page = HistoryPage::default();
    let (mut from, mut read) = (after, 0usize);
    while page.lines.len() < want {
        if read >= MAX_LINES_READ {
            page.cut = true;
            break;
        }
        let chunk = admin.log(from, BLOCK);
        let Some(last) = chunk.last().map(|l| l.id) else {
            break;
        };
        read += chunk.len();
        for line in chunk {
            if !skip(line.key.as_str()) && page.lines.len() < want {
                page.lines.push(line);
            }
        }
        from = Some(last);
    }
    page
}

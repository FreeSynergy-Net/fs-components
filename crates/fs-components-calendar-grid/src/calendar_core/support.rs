//! What every view of `bars-in-time` shares besides the reading: the notice for rows that could not be read, and the day it opens on.
//!
//! Pattern: Adapter (the context of the host in, the notice and the day out) – one place for `calendar-grid` and
//! `timeline`, so none keeps a copy.

use fs_components_core::form_view::DrawCtx;
use fs_components_core::states::{Labels, PartialNotice};

use fs_components_words::Localized;

use super::day::Day;

/// The notice "N of M could not be shown" with the keys of the rows that could not be read as names.
#[must_use]
pub fn partial_notice(ctx: &DrawCtx, invalid: &[String], total: usize) -> PartialNotice {
    PartialNotice::new(&Labels::for_locale(&ctx.locale), invalid, total)
}

/// The day on the clock (UTC): where a view opens, because the context knows no clock and no zone yet (`DrawCtx`, C2-V1).
#[must_use]
pub fn today_utc() -> Day {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    Day::from_number(i64::try_from(secs / 86_400).unwrap_or(0))
}

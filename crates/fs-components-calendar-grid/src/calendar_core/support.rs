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

/// The day of today on the clock of the context (UTC; the context knows no zone yet): where a view opens and what it marks as today.
#[must_use]
pub fn today(ctx: &DrawCtx) -> Day {
    Day::from_number(ctx.today_days())
}

//! `feed`: articles one after the other that go on and on (a timeline, a list of notices); Page Down and Page Up walk the articles.
//!
//! Pattern: State (`model.rs`, pure: the count, the cursor, the busy mark) + Observer (`view.rs`: `Live` draws the articles again, the
//! feed's own element stays so the keyboard stays) + Adapter. WAI-ARIA APG, feed: role `feed` (named by `aria-label`, `aria-busy`
//! while more are loading), every article is `article` named by its title (`aria-labelledby`) with `aria-posinset` and `aria-setsize`
//! (`-1` while the total is not known). Page Down goes to the next article, Page Up to the one before (no wrap). Limits: the feed is
//! ONE tab stop and the cursor is the `fs-article-current` mark (like `segmented`); the APG's Ctrl+Home and Ctrl+End (leave the feed)
//! are the host's; loading more is the caller's (`Feed::busy`, `Feed::push`); no virtual scrolling (`message_list` is the chat one).

mod model;
mod styles;
mod view;

pub use model::{Outcome, State};
pub use styles::STYLES;
pub use view::{Article, Feed};

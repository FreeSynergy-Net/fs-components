//! `disclosure` and `accordion`: a heading that shows or hides what belongs to it, one alone or a stack of them.
//!
//! Pattern: State (a mark per section) + Observer (`Live` draws the whole thing again, as `popover` does, because `aria-expanded`
//! changes) + Template method (`parts.rs`: one header and one panel shared by both). WAI-ARIA APG, disclosure: a button with
//! `aria-expanded` and `aria-controls` naming the content; Enter or Space toggles. APG, accordion: every header is a button inside
//! a heading (`aria-level`, a setting, default 3), the panel is a `region` named by its header; in the single way (`Accordion::single`)
//! the open header is `aria-disabled` (it cannot be closed by itself, as in the APG). Limits: the optional arrow keys between
//! headers are not done (every header is a tab stop, Tab walks them; the arrows would need the host to move the keyboard).

mod accordion;
mod parts;
mod styles;
mod single;

pub use accordion::{Accordion, Section};
pub use single::Disclosure;
pub use styles::STYLES;

//! `disabled`: the one place that draws a control that cannot be used, and says why.
//!
//! Pattern: Decorator - wraps any control in the disabled look (same size, class `fs-disabled`, `aria-disabled`, tab stop kept) and
//! hands its reason to pointer, keyboard, touch and screen reader through the [`Tooltip`](crate::tooltip::Tooltip) of this crate.
//! Standards: WAI-ARIA 1.2 `aria-disabled` (the element stays reachable, so the keyboard can reach the reason), APG tooltip
//! (`aria-describedby`, `role=tooltip`), WCAG 2.2 SC 1.4.13 (dismissable, hoverable, persistent: the tooltip). Deviation for touch,
//! which has no hover: a press on the control opens the tooltip and does nothing else.
//! No extra line, no star, no change of size: the reason is only in the tooltip (ADR 0096 H-K5, contract state `unsupported`).

mod labels;
mod observer;
mod styles;
mod wrap;

pub use labels::{Labels, TEXTS};
pub use observer::on_first_unsupported;
pub use styles::STYLES;
pub use wrap::{Disabled, Why};

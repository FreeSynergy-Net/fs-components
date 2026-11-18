//! `tooltip`: a small text that names or explains a control, shown while the pointer rests on it or the keyboard is on it.
//!
//! Pattern: State with an injected clock (`model.rs`) + Decorator (`view.rs`: [`Tooltip::attach`] decorates the trigger).
//! WAI-ARIA APG, tooltip pattern (still work in progress at W3C): the bubble has role `tooltip`, the trigger names it with
//! `aria-describedby`, the keyboard stays on the trigger, Escape closes, a focus tooltip closes when the focus leaves, a hover
//! tooltip appears after a delay (a setting, default [`DEFAULT_DELAY_MS`]) and stays while the pointer is over the trigger or
//! the bubble. Limits: the bubble sits under the trigger in the flow of its host (no overlay, no flip at the window edge).

mod model;
mod styles;
mod view;

pub use model::DEFAULT_DELAY_MS;
pub use styles::STYLES;
pub use view::Tooltip;

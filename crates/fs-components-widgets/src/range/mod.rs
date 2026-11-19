//! `range`: two values on one scale (a low and a high), each with its own thumb; the multi-thumb slider.
//!
//! Pattern: State (the pure rule of `Which::clamp`: a thumb never passes the other) + Observer (`view.rs`: `Live` draws the thumbs
//! again, the track stays so a drag is not cut off) + Adapter. WAI-ARIA APG, slider (multi-thumb): every thumb is a `slider` with its
//! own name, `aria-valuenow`, and limits that follow the other thumb (the low thumb's `aria-valuemax` is the high value, the high
//! thumb's `aria-valuemin` is the low value). EACH thumb is a tab stop (the APG's way); the keys are those of `slider`. The pointer
//! grabs the thumb that is nearer to where it presses and moves that one. Limits: horizontal only; the two thumbs may meet.

mod layers;
mod view;

pub use view::{Range, Which};

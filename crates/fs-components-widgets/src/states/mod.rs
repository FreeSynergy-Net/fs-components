//! The surfaces of the four states of a fetched thing and the blocks they are made of: `empty-state`, `alert` (the failed
//! surface, with "Try again"), `partial-notice`, `skeleton` (loading), and the [`Surface`] that picks one for a [`Load`].
//!
//! Pattern: State (`load.rs`) + Builder/Adapter (one small file per block). A block owns no data and no wording of its own
//! beyond the standard words of [`Labels`] (ADR 0057, German and English); what is missing and what to do is the caller's.
//!
//! The calendar grid maps its `Fetch` (ready with damaged ranges, loading, failed) to [`Load`]; the software list maps its
//! sources the same way. Limit: nothing here animates (the skeleton does not pulse).

mod alert;
mod empty;
mod labels;
mod load;
mod partial;
mod severity;
mod skeleton;
mod styles;

/// The least line of the glyph of a message in device pixels: a thin line of a small symbol never reaches 3:1 (WCAG 1.4.11).
pub const STROKE: f32 = 1.8;

pub use alert::{Act, Alert};
pub use empty::EmptyState;
pub use labels::{Labels, TEXTS};
pub use load::{Load, Surface};
pub use partial::PartialNotice;
pub use severity::Severity;
pub use skeleton::Skeleton;
pub use styles::STYLES;

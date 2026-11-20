//! `meter`: a value inside a known range, with zones that say whether it is good (a disk that is 70 % full, a battery, a score).
//!
//! Pattern: Strategy (`zone.rs`: the pure rule that names the zone of a value, after HTML's `meter`: `low`, `high`, `optimum`) +
//! Adapter. WAI-ARIA 1.2 role `meter` (no APG pattern): `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-valuetext` (the
//! value and the zone in words, MF2 de/en), named by `aria-label`. The zone is told by a word next to the bar and by the class
//! (`fs-meter-good`, `-fair`, `-poor`), not by colour alone. A meter is read, not set: no keys, no tab stop (a value the person sets
//! is a `slider`). Limits: horizontal only.

mod labels;
mod styles;
mod view;
mod zone;

pub use labels::{Labels, TEXTS};
pub use styles::STYLES;
pub use view::Meter;
pub use zone::{Zone, zone};

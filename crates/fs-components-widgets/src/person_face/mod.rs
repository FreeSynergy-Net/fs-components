//! `person-face`: the round picture of a person, or a circle with the initials when there is no picture.
//!
//! Pattern: Adapter over Blinc's image and text — the circle is ours (any size, not the few steps of `blinc_cn`'s avatar), the
//! initials come from [`initials`](crate::avatar_group::initials) (safe for any name) and the rule is the one of the avatar group:
//! a picture that cannot be loaded is never handed over (Blinc would draw a grey disc), the initials stand instead. A face is a
//! picture to the eye, so the name is put on it for a screen reader (`aria-label`).

mod face;

pub use face::{PersonFace, STYLES};

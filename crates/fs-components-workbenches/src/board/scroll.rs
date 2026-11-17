//! Auto-scroll while a card is carried: near the edge of a scroll area the area moves.
//!
//! Pattern: none - pure function. One step per pointer movement (Blinc sends a drag event only when the pointer moves).

/// How near to the edge the pointer starts the scroll, in pixels.
pub const EDGE: f32 = 32.0;
/// The most one step moves, in pixels.
pub const MAX_STEP: f32 = 24.0;

/// The step (negative = towards the start) for a pointer at `pos` in an area from `start` to `end` along one axis:
/// 0 in the middle, growing the nearer to the edge, `MAX_STEP` at the edge or beyond.
pub fn step(pos: f32, start: f32, end: f32) -> f32 {
    let (near_start, near_end) = (pos - start, end - pos);
    if near_start < EDGE {
        -MAX_STEP * ((EDGE - near_start) / EDGE).min(1.0)
    } else if near_end < EDGE {
        MAX_STEP * ((EDGE - near_end) / EDGE).min(1.0)
    } else {
        0.0
    }
}

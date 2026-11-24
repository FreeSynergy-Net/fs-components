//! Playing a plan: tick by tick against a driver.
//!
//! Pattern: Command (executes the step objects of a plan; no undo).

use crate::driver::Driver;
use crate::error::{Code, ReplayError};
use crate::step::{Button, Key, Named, Origin, Plan, Step};

/// The frame of the clock a moving pointer is cut into (milliseconds).
pub const FRAME_MS: u32 = 16;

/// Play every tick of the plan. The plan is checked against `Driver::caps` first: a step the driver cannot do stops the replay
/// before anything happened.
///
/// # Errors
/// The first `ReplayError` of the check or of a step (a missing element, a refused key ...).
pub fn perform(plan: &Plan, driver: &mut dyn Driver) -> Result<(), ReplayError> {
    check(plan, driver)?;
    for tick in &plan.ticks {
        let mut spent = 0;
        for step in &tick.steps {
            spent = spent.max(run(step, driver)?);
        }
        if tick.duration_ms > spent {
            driver.advance(tick.duration_ms - spent);
        }
    }
    Ok(())
}

fn check(plan: &Plan, driver: &dyn Driver) -> Result<(), ReplayError> {
    let caps = driver.caps();
    for step in plan.ticks.iter().flat_map(|t| &t.steps) {
        match step {
            Step::Key { key, down } => {
                let shift = matches!(key, Key::Named(Named::Shift));
                let modifier = matches!(key, Key::Named(Named::Control | Named::Alt | Named::Meta));
                if !down && !shift && !caps.key_up {
                    return Err(ReplayError::new(
                        Code::KeyUp,
                        format!("keyUp of {key:?}: the driver cannot release a key"),
                    ));
                }
                if modifier && !caps.modifiers {
                    return Err(ReplayError::new(
                        Code::Modifier,
                        format!("{key:?}: the driver holds only Shift"),
                    ));
                }
            }
            Step::Button { button, .. } if *button != Button::Left && !caps.extra_buttons => {
                return Err(ReplayError::new(
                    Code::Unsupported,
                    format!("{button:?} button: the driver has the left button only"),
                ));
            }
            Step::Wheel { .. } if !caps.wheel => {
                return Err(ReplayError::new(
                    Code::Unsupported,
                    "wheel: the driver has none",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Where an origin and an offset point to.
fn target(driver: &dyn Driver, origin: &Origin, x: f32, y: f32) -> Result<(f32, f32), ReplayError> {
    let (bx, by) = match origin {
        Origin::Viewport => (0.0, 0.0),
        Origin::Pointer => driver.pointer(),
        Origin::Element(id) => driver.centre(id)?,
    };
    Ok((bx + x, by + y))
}

/// One step; returns the milliseconds of the tick it used up itself.
fn run(step: &Step, driver: &mut dyn Driver) -> Result<u32, ReplayError> {
    match step {
        Step::Pause => Ok(0),
        Step::Move {
            origin,
            x,
            y,
            duration_ms,
        } => {
            let to = target(driver, origin, *x, *y)?;
            glide(driver, to, *duration_ms)
        }
        Step::Button { button, down } => driver.button(*button, *down).map(|()| 0),
        Step::Key { key, down } => if *down {
            driver.key_down(key)
        } else {
            driver.key_up(key)
        }
        .map(|()| 0),
        Step::Wheel {
            origin,
            x,
            y,
            dx,
            dy,
            duration_ms,
        } => {
            let (px, py) = target(driver, origin, *x, *y)?;
            driver.move_to(px, py)?;
            driver.wheel(*dx, *dy)?;
            driver.advance(*duration_ms);
            Ok(*duration_ms)
        }
    }
}

/// Move to a point: at once without a duration, else in frames of [`FRAME_MS`] along the straight line.
#[allow(clippy::cast_precision_loss)] // frame counts are tiny
fn glide(driver: &mut dyn Driver, to: (f32, f32), duration_ms: u32) -> Result<u32, ReplayError> {
    if duration_ms == 0 {
        driver.move_to(to.0, to.1)?;
        return Ok(0);
    }
    let from = driver.pointer();
    let frames = duration_ms.div_ceil(FRAME_MS);
    let mut spent = 0;
    for i in 1..=frames {
        let t = i as f32 / frames as f32;
        driver.move_to(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)?;
        let dt = FRAME_MS.min(duration_ms - spent);
        driver.advance(dt);
        spent += dt;
    }
    Ok(spent)
}

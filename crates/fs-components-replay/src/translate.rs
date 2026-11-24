//! The ONE place that knows the `webdriver` crate: its action types become our [`Plan`].
//!
//! Pattern: Adapter (`webdriver::actions` -> our step types). Swapping the crate means changing this file only.
//! Processing follows W3C `WebDriver` section 17.5: tick n takes the n-th action of every source, a tick lasts as long
//! as its longest action.

use webdriver::actions::{
    ActionSequence, ActionsType, GeneralAction, KeyAction, KeyActionItem, NullActionItem,
    PointerAction, PointerActionItem, PointerOrigin, PointerType, WheelAction, WheelActionItem,
};
use webdriver::command::ActionsParameters;

use crate::error::{Code, ReplayError};
use crate::step::{Button, Key, Named, Origin, Plan, Step, Tick};

/// The longest single duration a step may ask for (one minute); more is an input error, not a long wait.
pub const MAX_DURATION_MS: u32 = 60_000;

/// Read the body of Perform Actions (`{"actions":[...]}`) into a plan.
///
/// # Errors
/// `Code::Invalid` when the text is not a valid body; `Code::Unsupported`, `Code::Key` for what this module does not play.
pub fn parse(body: &str) -> Result<Plan, ReplayError> {
    let parameters: ActionsParameters = serde_json::from_str(body)
        .map_err(|e| ReplayError::new(Code::Invalid, format!("not a Perform Actions body: {e}")))?;
    translate(&parameters)
}

/// Turn parsed `WebDriver` parameters into a plan.
///
/// # Errors
/// As [`parse`].
pub(crate) fn translate(parameters: &ActionsParameters) -> Result<Plan, ReplayError> {
    let mut sources: Vec<Vec<(Step, u32)>> = Vec::new();
    for sequence in &parameters.actions {
        sources.push(source(sequence)?);
    }
    let length = sources.iter().map(Vec::len).max().unwrap_or(0);
    let ticks = (0..length)
        .map(|n| {
            let items: Vec<&(Step, u32)> = sources.iter().filter_map(|s| s.get(n)).collect();
            Tick {
                duration_ms: items.iter().map(|(_, d)| *d).max().unwrap_or(0),
                steps: items.into_iter().map(|(s, _)| s.clone()).collect(),
            }
        })
        .collect();
    Ok(Plan { ticks })
}

fn source(sequence: &ActionSequence) -> Result<Vec<(Step, u32)>, ReplayError> {
    let id = &sequence.id;
    match &sequence.actions {
        ActionsType::Null { actions } => actions
            .iter()
            .map(|NullActionItem::General(g)| pause(g))
            .collect(),
        ActionsType::Key { actions } => actions.iter().map(key_item).collect(),
        ActionsType::Pointer {
            parameters,
            actions,
        } => {
            if parameters.pointer_type != PointerType::Mouse {
                return Err(unsupported(format!(
                    "pointerType {:?} of source {id}: only mouse",
                    parameters.pointer_type
                )));
            }
            actions.iter().map(pointer_item).collect()
        }
        ActionsType::Wheel { actions } => actions.iter().map(wheel_item).collect(),
    }
}

fn pause(GeneralAction::Pause(p): &GeneralAction) -> Result<(Step, u32), ReplayError> {
    Ok((Step::Pause, duration(p.duration, "pause")?))
}

/// A duration in milliseconds; too long is clamped, and `check_duration` has refused the absurd before.
fn clamp(ms: u64) -> u32 {
    u32::try_from(ms.min(u64::from(MAX_DURATION_MS))).unwrap_or(MAX_DURATION_MS)
}

fn duration(d: Option<u64>, what: &str) -> Result<u32, ReplayError> {
    let ms = d.unwrap_or(0);
    if ms > u64::from(MAX_DURATION_MS) {
        return Err(ReplayError::new(
            Code::Invalid,
            format!("{what}: duration {ms} ms is over the limit of {MAX_DURATION_MS} ms"),
        ));
    }
    Ok(clamp(ms))
}

fn unsupported(detail: String) -> ReplayError {
    ReplayError::new(Code::Unsupported, detail)
}

fn pointer_item(item: &PointerActionItem) -> Result<(Step, u32), ReplayError> {
    match item {
        PointerActionItem::General(g) => pause(g),
        PointerActionItem::Pointer(PointerAction::Cancel) => {
            Err(unsupported("pointerCancel".into()))
        }
        PointerActionItem::Pointer(PointerAction::Down(a)) => Ok((
            Step::Button {
                button: button(a.button)?,
                down: true,
            },
            0,
        )),
        PointerActionItem::Pointer(PointerAction::Up(a)) => Ok((
            Step::Button {
                button: button(a.button)?,
                down: false,
            },
            0,
        )),
        PointerActionItem::Pointer(PointerAction::Move(a)) => {
            let ms = duration(a.duration, "pointerMove")?;
            Ok((
                Step::Move {
                    origin: origin(&a.origin),
                    x: coord(a.x),
                    y: coord(a.y),
                    duration_ms: ms,
                },
                ms,
            ))
        }
    }
}

fn wheel_item(item: &WheelActionItem) -> Result<(Step, u32), ReplayError> {
    match item {
        WheelActionItem::General(g) => pause(g),
        WheelActionItem::Wheel(WheelAction::Scroll(a)) => {
            if a.origin == PointerOrigin::Pointer {
                return Err(unsupported(
                    "scroll with origin pointer (not allowed for a wheel)".into(),
                ));
            }
            let (Some(x), Some(y), Some(dx), Some(dy)) = (a.x, a.y, a.deltaX, a.deltaY) else {
                return Err(ReplayError::new(
                    Code::Invalid,
                    "scroll needs x, y, deltaX and deltaY",
                ));
            };
            let ms = duration(a.duration, "scroll")?;
            Ok((
                Step::Wheel {
                    origin: origin(&a.origin),
                    x: coord_i(x),
                    y: coord_i(y),
                    dx: coord_i(dx),
                    dy: coord_i(dy),
                    duration_ms: ms,
                },
                ms,
            ))
        }
    }
}

fn key_item(item: &KeyActionItem) -> Result<(Step, u32), ReplayError> {
    match item {
        KeyActionItem::General(g) => pause(g),
        KeyActionItem::Key(KeyAction::Down(a)) => Ok((
            Step::Key {
                key: key(&a.value)?,
                down: true,
            },
            0,
        )),
        KeyActionItem::Key(KeyAction::Up(a)) => Ok((
            Step::Key {
                key: key(&a.value)?,
                down: false,
            },
            0,
        )),
    }
}

fn button(number: u64) -> Result<Button, ReplayError> {
    match number {
        0 => Ok(Button::Left),
        1 => Ok(Button::Middle),
        2 => Ok(Button::Right),
        n => Err(unsupported(format!("button {n}"))),
    }
}

fn origin(origin: &PointerOrigin) -> Origin {
    match origin {
        PointerOrigin::Viewport => Origin::Viewport,
        PointerOrigin::Pointer => Origin::Pointer,
        PointerOrigin::Element(e) => Origin::Element(e.0.clone()),
    }
}

#[allow(clippy::cast_possible_truncation)] // pixels of a surface, far below the f32 range that matters
fn coord(v: f64) -> f32 {
    v as f32
}

#[allow(clippy::cast_precision_loss)] // pixels of a surface
fn coord_i(v: i64) -> f32 {
    v as f32
}

/// A key value: one printable character, or one of the normalised special values of section 17.4.2.
fn key(value: &str) -> Result<Key, ReplayError> {
    let mut chars = value.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return Err(ReplayError::new(
            Code::Key,
            format!("{value:?}: one character expected"),
        ));
    };
    let named = match c {
        '\u{E003}' => Named::Backspace,
        '\u{E004}' => Named::Tab,
        '\u{E006}' | '\u{E007}' => Named::Enter,
        '\u{E008}' => Named::Shift,
        '\u{E009}' => Named::Control,
        '\u{E00A}' => Named::Alt,
        '\u{E00C}' => Named::Escape,
        '\u{E00D}' => Named::Space,
        '\u{E00E}' => Named::PageUp,
        '\u{E00F}' => Named::PageDown,
        '\u{E010}' => Named::End,
        '\u{E011}' => Named::Home,
        '\u{E012}' => Named::Left,
        '\u{E013}' => Named::Up,
        '\u{E014}' => Named::Right,
        '\u{E015}' => Named::Down,
        '\u{E017}' => Named::Delete,
        '\u{E03D}' => Named::Meta,
        '\u{E000}'..='\u{F8FF}' => {
            return Err(ReplayError::new(
                Code::Key,
                format!(
                    "U+{:04X}: a special key outside the supported table",
                    u32::from(c)
                ),
            ));
        }
        c => return Ok(Key::Char(c)),
    };
    Ok(Key::Named(named))
}

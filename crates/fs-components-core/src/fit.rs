//! `fit`: the sizes of a panel as data (draft "Fluss-Panels" part B, the small part of A9, without `measure`).
//!
//! A length as written in a contract, a layout or by the person (`Len`), the statements of one layer (`SizeSpec`), the
//! resolved sizes in lpx for ONE generation (`Fit`) and `merge`, which applies the order contract, theme, layout, person.
//! Each layer may only narrow the one before it; every clamp of a layout or person statement comes back as a `FitNote` (visible, never silent).
//! A contract wish below the theme lower bound is raised to it without a note (both belong to the system).
//!
//! Pattern: Value Object – plain data plus one pure function (`merge`); no state, no Blinc type.

/// One length as written (EZ M2): `em` for everything that depends on text, `lpx` otherwise.
///
/// Pattern: Value Object – a length before it is resolved against the theme's `em`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    /// Logical pixels.
    Lpx(f32),
    /// Multiples of the body text size of the theme.
    Em(f32),
    /// "As the content needs": says nothing about the size.
    Fit,
    /// "The rest of the frame": for a height the panel fills; for a width it says nothing.
    Fill,
}

impl Len {
    /// The length in lpx, `None` for `Fit`, `Fill` and non-finite or negative numbers.
    pub fn resolve(self, em: f32) -> Option<f32> {
        let v = match self {
            Len::Lpx(x) => x,
            Len::Em(x) => x * em,
            Len::Fit | Len::Fill => return None,
        };
        (v.is_finite() && v >= 0.0).then_some(v)
    }
}

/// The statements of one layer (contract, layout or person); `None` says nothing.
///
/// Pattern: Value Object – what one layer wants, before the layers are merged.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SizeSpec {
    /// `min-width`.
    pub min: Option<Len>,
    /// `width`: the wish.
    pub width: Option<Len>,
    /// `max-width`.
    pub max: Option<Len>,
    /// `grow` (share of the free space after the wish).
    pub grow: Option<f32>,
    /// `min-height`.
    pub min_h: Option<Len>,
    /// `height` (`Len::Fill` = fills the rest of the frame).
    pub height: Option<Len>,
}

/// The sizes of a panel in lpx, valid for ONE generation (theme, font or scale change = next generation).
///
/// Pattern: Value Object – resolved sizes, `min <= width <= max` always holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    /// Smallest width the panel can be drawn in.
    pub min: f32,
    /// Wished width.
    pub width: f32,
    /// Largest width (`f32::INFINITY` if nobody limits it).
    pub max: f32,
    /// Share of the free space after the wish.
    pub grow: f32,
    /// Smallest height.
    pub min_h: f32,
    /// Height (default `min_h`).
    pub height: f32,
    /// The panel takes the rest of the frame's height.
    pub fill: bool,
    /// The panel scrolls in two directions inside itself (board, table, map, calendar grid); set by the caller from the contract.
    pub reflow_2d: bool,
}

/// What `merge` had to change; shown to the author, never silent.
///
/// Pattern: Value Object – one finding of the merge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FitNote {
    /// The layout went beyond what the contract allows for this key; clamped to the contract.
    LayoutWiderThanContract {
        /// `min-width`, `width` or `max-width`.
        key: &'static str,
    },
    /// The lower bound is above the upper bound; the lower bound wins (readable before tight).
    MinAboveMax,
    /// The person's width was moved into `[min, max]`.
    PersonClamped,
}

fn r(len: Option<Len>, em: f32) -> Option<f32> {
    len.and_then(|l| l.resolve(em))
}

/// Merge the layers in the order contract, theme, layout, person (draft B.1 to B.4).
///
/// The theme gives only a lower bound (`theme_min`); the person sets only the wish `width`. Layout statements beyond the
/// contract are clamped to it (`LayoutWiderThanContract`); a lower bound above the upper bound wins (`MinAboveMax`).
/// `reflow_2d` is not a size statement and stays `false`; the caller sets it from the contract.
pub fn merge(
    contract: &SizeSpec,
    layout: &SizeSpec,
    person: &SizeSpec,
    theme_min: f32,
    em: f32,
) -> (Fit, Vec<FitNote>) {
    let mut notes = Vec::new();
    let theme_min = if theme_min.is_finite() {
        theme_min.max(0.0)
    } else {
        0.0
    };
    let c_min = r(contract.min, em).unwrap_or(0.0).max(theme_min);
    let c_max = r(contract.max, em).unwrap_or(f32::INFINITY);
    let c_wish = r(contract.width, em).unwrap_or(c_min);

    // Layout may raise `min`, lower `max`, set `width` inside; anything wider than the contract is clamped.
    let mut min = c_min;
    if let Some(v) = r(layout.min, em) {
        if v < c_min {
            notes.push(FitNote::LayoutWiderThanContract { key: "min-width" });
        } else {
            min = v;
        }
    }
    let mut max = c_max;
    if let Some(v) = r(layout.max, em) {
        if v > c_max {
            notes.push(FitNote::LayoutWiderThanContract { key: "max-width" });
        } else {
            max = v;
        }
    }
    if min > max {
        notes.push(FitNote::MinAboveMax);
        max = min;
    }
    let mut width = c_wish;
    if let Some(v) = r(layout.width, em) {
        // Out of the contract, or out of the already narrowed [min, max]: a note (the clamp below moves it).
        if v < min || v > max {
            notes.push(FitNote::LayoutWiderThanContract { key: "width" });
        }
        if v >= c_min && v <= c_max {
            width = v;
        }
    }
    if let Some(v) = r(person.width, em) {
        if v < min || v > max {
            notes.push(FitNote::PersonClamped);
        }
        width = v;
    }
    width = width.clamp(min, max);

    let grow = [layout.grow, contract.grow]
        .into_iter()
        .flatten()
        .find(|g| g.is_finite() && *g >= 0.0)
        .unwrap_or(1.0);
    let min_h = r(layout.min_h, em)
        .unwrap_or(0.0)
        .max(r(contract.min_h, em).unwrap_or(0.0));
    let h_len = layout.height.or(contract.height);
    let fill = h_len == Some(Len::Fill);
    let height = r(h_len, em).unwrap_or(min_h).max(min_h);
    let fit = Fit {
        min,
        width,
        max,
        grow,
        min_h,
        height,
        fill,
        reflow_2d: false,
    };
    (fit, notes)
}

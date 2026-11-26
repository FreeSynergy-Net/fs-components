//! The channel `color` of `bars-in-time@1.1.0` (JSCalendar `color`, RFC 8984 section 4.2.11: a CSS colour value) mapped to one of the six
//! colour slots (`color-0` to `color-5`) of the calendar components. Nothing is derived from `lane`: no `color`, no slot.
//!
//! Pattern: Strategy (one pure function from a colour value to a slot) – the same colour always gives the same slot, in every view.
//! The colour is read by the `csscolorparser` crate (names, `#rgb`, `rgb()`, `hsl()` ...), so `red`, `#f00` and `rgb(255 0 0)` are one colour
//! and one slot; the slot is a fixed hash (FNV-1a) of its four bytes, so it does not change between runs or versions of the host.
//! A fully transparent colour is no colour to show and counts like an unreadable one.

/// How many colour slots the components have.
pub const SLOTS: u8 = 6;

/// FNV-1a, 32 bit: the offset basis and the prime (written in decimal: they are numbers, not colours).
const FNV_OFFSET: u32 = 2_166_136_261;
const FNV_PRIME: u32 = 16_777_619;

/// The slot of a CSS colour value; `None` for a value that is not a colour (the caller counts it and shows the row without colour).
#[must_use]
pub fn color_slot(value: &str) -> Option<u8> {
    let value = value.trim();
    // The crate also takes hex digits without the `#`; CSS (and so RFC 8984) does not, and no colour name is made of hex digits only.
    if value.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let rgba = csscolorparser::parse(value).ok()?.to_rgba8();
    if rgba[3] == 0 {
        return None;
    }
    let hash = rgba.iter().fold(FNV_OFFSET, |h, b| {
        (h ^ u32::from(*b)).wrapping_mul(FNV_PRIME)
    });
    u8::try_from(hash % u32::from(SLOTS)).ok()
}

//! A value that is a link: underlined text in the link colour, a tab stop, opened by Enter or a click.
//!
//! Pattern: Adapter over Blinc's `open_url` (through `fs_components_widgets::text_view::open_external`) and the one
//! `activatable` of the controls. The ink is `link_ink`: the accent as text, moved toward the text colour until it reads (Apple's blue does not at 3.5:1). A link is told by the underline AND the colour, never by colour alone. WAI-ARIA `link`:
//! Enter follows it. Limit: the window turns Enter AND Space into a click on the focused element, which arrives as `Key::Activate`, so Space follows too (APG: Enter only); it cannot be told apart at this layer.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_styles::{Palette, Rgba};
use fs_components_widgets::keys::Key;
use fs_components_widgets::text_view::open_external;

use super::Look;
use crate::controls::activatable;

/// What follows a link: called with the address. `None` opens it in the system's program.
pub type Follow = Option<Arc<dyn Fn(&str) + Send + Sync>>;

fn follow(handler: &Follow, href: &str) {
    match handler {
        Some(f) => f(href),
        None => open_external(href),
    }
}

/// The WCAG ratio a link's ink needs against the ground and the panel it stands on.
pub const LINK_CONTRAST: f32 = 4.5;

fn toward(from: Rgba, to: Rgba, step: u16) -> Rgba {
    let mix = |a: u8, b: u8| {
        let (a, b) = (i32::from(a), i32::from(b));
        u8::try_from((a * (100 - i32::from(step)) + b * i32::from(step)) / 100).unwrap_or(u8::MAX)
    };
    Rgba {
        r: mix(from.r, to.r),
        g: mix(from.g, to.g),
        b: mix(from.b, to.b),
        a: from.a,
    }
}

/// The ink of a link in a palette: the accent as text (`accent_text`), moved toward the text colour in steps of 5 per cent until
/// it reads at [`LINK_CONTRAST`] on the ground and the panel. A style whose accent already reads (Admin, Windows) keeps it
/// unchanged; Apple's own blue (3.5:1) is darkened or lightened just enough.
pub fn link_ink(pal: &Palette) -> Rgba {
    let reads =
        |c: Rgba| c.contrast(pal.ground) >= LINK_CONTRAST && c.contrast(pal.panel) >= LINK_CONTRAST;
    let mut ink = pal.accent_text;
    for step in (5..=100).step_by(5) {
        if reads(ink) {
            break;
        }
        ink = toward(pal.accent_text, pal.text, step);
    }
    ink
}

/// `content` as a link to `href`, `size` pixels, in the link colour of the look.
pub(super) fn linked(
    look: &Look,
    id: &str,
    content: &str,
    size: f32,
    href: &str,
    to: &Follow,
) -> Div {
    let pal = look.palette();
    let (keys_href, press_href) = (href.to_owned(), href.to_owned());
    let (keys_to, press_to) = (to.clone(), to.clone());
    let root = div()
        .id(id)
        .class("fs-field-link")
        .attr("role", "link")
        .attr("href", href)
        .cursor_pointer()
        .child(look.text(content, size, link_ink(pal)).underline());
    activatable(
        root,
        Arc::new(move |event| {
            if matches!(event.key, Key::Activate | Key::Enter) {
                follow(&keys_to, &keys_href);
                true
            } else {
                false
            }
        }),
        Arc::new(move || follow(&press_to, &press_href)),
    )
}

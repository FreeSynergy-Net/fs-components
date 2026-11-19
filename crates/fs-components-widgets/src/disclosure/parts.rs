//! The header and the panel both widgets draw.
//!
//! Pattern: Template method — the same header (a button with the mark, the title, the ARIA state and the keys) and the same panel
//! for one disclosure and for every section of an accordion; the caller says what a press does.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::event_router::keyboard_focused_node;
use blinc_layout::prelude::text;

use crate::keys::{Key, bind_keys};

/// What a press on a header does.
pub type Press = Arc<dyn Fn() + Send + Sync>;

/// The mark before the title: a triangle that points right when closed and down when open.
pub fn mark(open: bool) -> &'static str {
    if open { "\u{25BE}" } else { "\u{25B8}" }
}

/// A header: a button named by `title`, with `aria-expanded` and `aria-controls` naming the panel. `inert` (the open header of a
/// single accordion) is `aria-disabled` and does nothing. Enter or Space (as `Key::Activate` or raw) and a click press it once.
pub fn header(id: &str, panel: &str, title: &str, open: bool, inert: bool, press: &Press) -> Div {
    let mut head = div()
        .id(id)
        .class("fs-disc-head")
        .attr("role", "button")
        .attr("aria-expanded", if open { "true" } else { "false" })
        .attr("aria-controls", panel)
        .child(div().class("fs-disc-mark").attr("aria-hidden", "true").child(text(mark(open))))
        .child(text(title).class("fs-disc-title"));
    if inert {
        return head.class("fs-disc-head-off").attr("aria-disabled", "true").attr("tabindex", "0");
    }
    if open {
        head = head.class("fs-disc-head-open");
    }
    let (by_key, by_pointer) = (Arc::clone(press), Arc::clone(press));
    bind_keys(
        head,
        Arc::new(move |event| match event.key {
            Key::Activate | Key::Enter | Key::Space => {
                by_key();
                true
            }
            _ => false,
        }),
    )
    .on_click(move |ctx| {
        // The keyboard's Enter and Space come as a click too: `bind_keys` already took them.
        if keyboard_focused_node() != Some(ctx.node_id) {
            by_pointer();
        }
    })
}

/// The panel: `role="region"` named by its header when `region` (an accordion), else a plain container (a disclosure).
pub fn panel(id: &str, header_id: &str, region: bool, content: Div) -> Div {
    let panel = div().id(id).class("fs-disc-panel");
    let panel = if region {
        panel.attr("role", "region").attr("aria-labelledby", header_id)
    } else {
        panel
    };
    panel.child(content)
}

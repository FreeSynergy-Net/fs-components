//! Wiring an element for the context menu (M3): right click, menu key, Shift+F10.
//!
//! Pattern: Adapter (Blinc pointer and key events to the neutral [`Trigger`] of `fs-components-commands`).

use std::sync::Arc;

use blinc_layout::div::Div;
use blinc_layout::event_handler::EventContext;
use blinc_layout::event_router::keyboard_focused_node;
use fs_components_commands::{Area, Point, Trigger};

use crate::key_events::context_key;

/// Make `element` open a context menu: a right click on it (Shift held asks for the second stage), and the menu key or Shift+F10
/// while it has the keyboard (at the element, not at the pointer). `ask` gets the [`Trigger`]; it finds the target
/// (`fs_components_commands::target_for`) and calls `AppWidget::open_context_menu`. The element becomes a tab stop.
pub fn context_trigger(element: Div, ask: impl Fn(Trigger) + Send + Sync + 'static) -> Div {
    let ask = Arc::new(ask);
    let by_mouse = Arc::clone(&ask);
    element
        .attr("tabindex", "0")
        .on_right_click(move |ctx| {
            by_mouse(Trigger::Pointer {
                at: Point {
                    x: ctx.mouse_x,
                    y: ctx.mouse_y,
                },
                shift: ctx.shift,
            });
        })
        .on_key_down(move |ctx| {
            // A key press is broadcast to every handler: only the element that has the keyboard answers.
            if keyboard_focused_node() == Some(ctx.node_id)
                && let Some(trigger) = context_key(ctx.key_code, ctx.shift, area(ctx))
            {
                ask(trigger);
            }
        })
}

fn area(ctx: &EventContext) -> Area {
    Area {
        x: ctx.bounds_x,
        y: ctx.bounds_y,
        width: ctx.bounds_width,
        height: ctx.bounds_height,
    }
}

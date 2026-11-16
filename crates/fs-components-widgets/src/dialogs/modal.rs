//! A dialog over a page: the page cannot take the keyboard while the dialog is open.
//!
//! Pattern: Decorator — the page is wrapped and marked out of the tab order (`data-focus-active=false`, Blinc's own switch),
//! the dialog sits in a scrim over it. Tab then walks only the stops of the dialog and starts again at its first (the trap).
//! A frontend that opens the dialog in Blinc's overlay gets the same from the overlay's focus scope and does not need this.

use blinc_layout::div::{Div, div};
use blinc_layout::stack::stack;

use crate::Element;

/// `page` with `dialog` centred over it in a scrim. Part names of the contract: `dialog.backdrop` is the scrim. The host is a
/// Blinc `Stack`: each child is a layer, and Blinc draws the surfaces AND the text of a layer together, so the text of the page
/// lies under the scrim (a plain `div` with an absolute scrim draws the page text after the scrim: measured, the text stayed
/// fully bright and showed through the dialog).
pub fn modal(page: impl Element + 'static, dialog: impl Element + 'static) -> Div {
    let layers = stack()
        .w_full()
        .h_full()
        .child(
            div()
                .class("fs-dlg-page")
                .w_full()
                .h_full()
                .attr("data-focus-active", "false")
                .child(page),
        )
        .child(
            div()
                .class("fs-dlg-backdrop")
                .w_full()
                .h_full()
                .child(div().class("fs-dlg-slot").child(dialog)),
        );
    div().class("fs-dlg-host").w_full().h_full().child(layers)
}

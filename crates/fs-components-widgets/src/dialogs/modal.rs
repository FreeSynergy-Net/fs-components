//! A dialog over a page: the page cannot take the keyboard while the dialog is open.
//!
//! Pattern: Decorator — the page is wrapped and marked out of the tab order (`data-focus-active=false`, Blinc's own switch),
//! the dialog sits in a scrim over it. Tab then walks only the stops of the dialog and starts again at its first (the trap).
//! A frontend that opens the dialog in Blinc's overlay gets the same from the overlay's focus scope and does not need this.

use blinc_layout::div::{Div, div};

use crate::Element;

/// `page` with `dialog` centred over it in a scrim. Part names of the contract: `dialog.backdrop` is the scrim.
pub fn modal(page: impl Element + 'static, dialog: impl Element + 'static) -> Div {
    div()
        .class("fs-dlg-host")
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
                .absolute()
                .inset(0.0)
                .child(div().class("fs-dlg-slot").child(dialog)),
        )
}

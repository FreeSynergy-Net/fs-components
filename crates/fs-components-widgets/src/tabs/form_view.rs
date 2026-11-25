//! The tabs as a [`FormView`] of the form `rows`: one tab per row, the channel `label` is the tab, the row whose channel `state` is
//! `current` is the chosen one, a choice goes out as the command `open` for the key of the row.
//!
//! The tabs here are the Frontends of a program or any other list the module calls a row; the panel is not drawn here (the
//! frontend shows it, see the block `tablist`). A tab is locked when the host does not let the person `open` that row.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, [`Tabs`] out, the choice out as a write through the port).

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::FormWrite;
use fs_components_core::keys::Key;

use super::{Tab, Tabs};

/// The widget this view is (`widget` of the block `tablist`).
pub const WIDGET: &str = "Tabs";

/// The command of the form `rows` that a choice sends.
pub const OPEN: &str = "open";

/// The value of the channel `state` that marks the chosen row.
pub const CURRENT: &str = "current";

/// The view of the block `tablist`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        let tabs = make(&input, &ctx);
        Box::new(MountedTabs(Mutex::new((ctx, tabs))))
    }
}

/// The tabs of `input`: a row the host does not let be opened is a locked tab; the chosen one is the row marked `current`.
pub(crate) fn make(input: &FormInput, ctx: &DrawCtx) -> Tabs {
    let tabs: Vec<Tab> = input
        .rows
        .iter()
        .map(|r| {
            let tab = Tab::new(r.key.as_str(), r.get("label").unwrap_or(&r.key));
            if ctx.write.can(OPEN, Some(&r.key)) {
                tab
            } else {
                tab.disabled()
            }
        })
        .collect();
    let write: Arc<dyn FormWrite> = ctx.write.clone();
    let row = Tabs::new(tabs).id("tablist").on_select(move |key| {
        // a refusal changes nothing here: the module's next answer says which tab is current
        let _ = write.send(OPEN, Some(key), &[]);
    });
    if let Some(current) = input.rows.iter().find(|r| r.get("state") == Some(CURRENT)) {
        row.choose(&current.key);
    }
    row
}

/// The context and the tabs.
struct MountedTabs(Mutex<(DrawCtx, Tabs)>);

impl MountedTabs {
    fn tabs(&self) -> Tabs {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .1
            .clone()
    }
}

impl Mounted for MountedTabs {
    fn build(&self, width: f32, _height: f32) -> Div {
        div().w(width).child(self.tabs().build())
    }

    fn refresh(&self, input: FormInput) {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        held.1 = make(&input, &held.0);
    }

    fn key(&self, key: Key) -> bool {
        self.tabs().press(key)
    }
}

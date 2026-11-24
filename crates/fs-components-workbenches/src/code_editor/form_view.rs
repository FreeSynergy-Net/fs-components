//! The entry: [`View`] says which widget it is and makes the mounted editor over a [`FormInput`].
//!
//! Pattern: Abstract Factory (`View`) + Adapter (the rows of `single-record@1.1.0` in, a code editor out, one `edit` back).

use std::sync::Arc;

use blinc_layout::div::Div;
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};

use super::draw;
use super::wiring::Wiring;

/// The widget this view is (`widget` of the block `code-editor`).
pub const WIDGET: &str = "CodeEditor";

/// The view of the block `code-editor`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        Box::new(MountedEditor(Arc::new(Wiring::new(input, &ctx))))
    }
}

/// The editor with its draft; kept by the host over a rebuild, so what was typed stays.
pub(super) struct MountedEditor(pub(super) Arc<Wiring>);

impl Mounted for MountedEditor {
    fn build(&self, width: f32, height: f32) -> Div {
        draw::build(&self.0, width, height)
    }

    fn refresh(&self, input: FormInput) {
        self.0.load(input);
    }
}

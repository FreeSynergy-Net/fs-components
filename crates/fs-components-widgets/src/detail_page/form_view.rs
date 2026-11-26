//! The page as a [`FormView`] of the form `single-record@^1.2.0`: the first record is the page; `label` names it, `preview` is its
//! picture, `summary` and `state` stand under the name, `fields` are the cards. The values of a field are found by their PATH, never
//! by position, so a field with two values (two e-mails) shows two cards under one label and a field without a value shifts nothing.
//! An older Minor (1.0.0, 1.1.0, whose `fields` do not repeat) is read the same way (tolerant reader, `forms.txt` rule "repeated value").
//!
//! The same adapter ([`heads`], [`section`]) reads the channel `columns` of the form `rows` for the people workbench: one rule for
//! every reader of a record.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (`FormInput` in, [`Sheet`] out).

use std::sync::{Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};

use super::{Field, FieldKind, Section, Sheet, page};

/// The widget this view is (`widget` of the block `detail-page`).
pub const WIDGET: &str = "DetailPage";

/// The channel of `single-record` that carries the fields.
pub const FIELDS: &str = "fields";

/// What names one field of a channel: its path, the words shown for it and what it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Head {
    /// The path of the field (what its values carry).
    pub path: String,
    /// The words shown as its label.
    pub shown: String,
    /// What it holds, from the block of its type.
    pub kind: FieldKind,
}

/// The heads of the fields behind `channel`, in the order of the module's map (none when the module does not fill it).
pub fn heads(input: &FormInput, channel: &str, say: &dyn Fn(&str) -> String) -> Vec<Head> {
    input.channel(channel).map_or_else(Vec::new, |c| {
        c.fields
            .iter()
            .map(|f| Head {
                path: f.name.clone(),
                shown: f.shown(say),
                kind: FieldKind::of_block(&f.block),
            })
            .collect()
    })
}

/// The one section (no title) of a record: every head with its values from `channel`, in the order of the module; a head without a
/// value gives no field. Until the type catalog gives groups there is exactly one section.
pub fn section(row: &FormRow, channel: &str, heads: &[Head]) -> Section {
    let fields = heads
        .iter()
        .filter_map(|h| {
            let values: Vec<String> = row
                .field(channel, &h.path)
                .into_iter()
                .filter(|v| !v.trim().is_empty())
                .map(str::to_owned)
                .collect();
            (!values.is_empty()).then(|| Field::many(&h.shown, values).kind(h.kind))
        })
        .collect();
    Section::new("", fields)
}

/// The sheet of one record of a `single-record` form.
pub fn sheet(input: &FormInput, row: &FormRow, say: &dyn Fn(&str) -> String) -> Sheet {
    let text = |channel: &str| row.get(channel).unwrap_or_default().to_owned();
    let mut sheet = Sheet::new(
        row.get("label").unwrap_or(&row.key),
        vec![section(row, FIELDS, &heads(input, FIELDS, say))],
    )
    .with_summary(text("summary"))
    .with_state(text("state"));
    if let Some(image) = row.get("preview") {
        sheet = sheet.with_image(image);
    }
    sheet
}

/// The view of the block `detail-page`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        Box::new(MountedPage {
            held: Mutex::new(input),
            ctx,
        })
    }
}

struct MountedPage {
    held: Mutex<FormInput>,
    ctx: DrawCtx,
}

impl Mounted for MountedPage {
    fn build(&self, width: f32, _height: f32) -> Div {
        let input = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let out = div().class("fs-detail-frame").w(width).flex_col();
        // the first record is the page; a form of several records shows the first
        match input.rows.first() {
            Some(row) => out.child(page(
                &sheet(&input, row, &*self.ctx.say),
                "detail-page",
                "",
                &|_, _| Vec::new(),
            )),
            None => out,
        }
    }

    fn refresh(&self, input: FormInput) {
        *self.held.lock().unwrap_or_else(PoisonError::into_inner) = input;
    }
}

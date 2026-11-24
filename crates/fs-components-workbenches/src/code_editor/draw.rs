//! The look of the code editor: the bar (title, language, Discard, Save), the text with its overview map, and for Markdown the
//! preview beside it behind a divider that can be moved.
//!
//! Pattern: Adapter (the state of the [`Wiring`] to Blinc elements) — nothing here changes the draft except through the wiring.

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_cn::{resizable_group, resizable_panel};
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Block, TextView};

use super::wiring::Wiring;

/// The least height of the editor, in pixels.
const MIN_H: f32 = 320.0;
/// The height of the bar, in pixels.
const BAR_H: f32 = 44.0;
/// The width of the overview map, in pixels.
const MAP_W: f32 = 90.0;
/// The least width of the preview beside the text, in pixels.
const PREVIEW_MIN: f32 = 160.0;

/// The whole editor for this size.
pub(super) fn build(wiring: &Arc<Wiring>, width: f32, height: f32) -> Div {
    let height = height.max(MIN_H);
    let body_h = height - BAR_H;
    let bar = {
        let w = Arc::clone(wiring);
        wiring.live.region("bar", move || bar(&w))
    };
    let body = if wiring.has_preview() {
        split(wiring, width, body_h)
    } else {
        text_with_map(wiring, width, body_h)
    };
    div().w(width).h(height).flex_col().child(bar).child(body)
}

/// Title, language, the reason when locked or refused, and the two buttons.
fn bar(wiring: &Arc<Wiring>) -> Div {
    let locked = wiring.locked();
    let dirty = wiring.draft().is_dirty();
    let mut bar = div()
        .flex_row()
        .items_center()
        .gap(8.0)
        .h(BAR_H)
        .child(text(wiring.label()))
        .child(div().flex_1())
        .child(text(wiring.language_label()));
    if let Some(reason) = locked.clone().or_else(|| wiring.refusal()) {
        bar = bar.child(text(reason));
    }
    let may = locked.is_none() && dirty;
    let (discard, save) = (Arc::clone(wiring), Arc::clone(wiring));
    bar.child(
        cn::button(wiring.say("code-editor.discard"))
            .variant(ButtonVariant::Secondary)
            .disabled(!may)
            .on_click(move |_| discard.discard()),
    )
    .child(
        cn::button(wiring.say("code-editor.save"))
            .variant(ButtonVariant::Primary)
            .disabled(!may)
            .on_click(move |_| save.save()),
    )
}

/// The text (editable, or locked with its reason in the bar) and the overview map right of it.
fn text_with_map(wiring: &Arc<Wiring>, width: f32, height: f32) -> Div {
    let map = {
        let w = Arc::clone(wiring);
        wiring.live.region("map", move || {
            div().w(MAP_W).child(w.draft().editor().minimap())
        })
    };
    let editor = if wiring.locked().is_some() {
        div()
            .w(width - MAP_W)
            .h(height)
            .child(wiring.draft().editor().read_only())
    } else {
        let typed = Arc::clone(wiring);
        let widget = wiring
            .draft()
            .editor()
            .widget()
            .w(width - MAP_W)
            .h(height)
            .on_change(move |_| typed.typed());
        div().w(width - MAP_W).h(height).child(widget)
    };
    div().w(width).h(height).flex_row().child(editor).child(map)
}

/// The text and the preview of the draft side by side; the divider moves.
fn split(wiring: &Arc<Wiring>, width: f32, height: f32) -> Div {
    let key = wiring.key().unwrap_or_default();
    let preview = {
        let w = Arc::clone(wiring);
        wiring.live.region("preview", move || preview(&w))
    };
    let group = resizable_group()
        .horizontal()
        .key(format!("code-editor-split-{key}"))
        .panel(
            resizable_panel()
                .flex_grow()
                .child(text_with_map(wiring, width / 2.0, height)),
        )
        .panel(
            resizable_panel()
                .default_size(width / 2.0)
                .min_size(PREVIEW_MIN)
                .child(preview),
        );
    div().w(width).h(height).child(group)
}

/// The draft shown under the title "Preview". There is no Markdown reader in the repository yet, so the text is shown as it is
/// (`Block::Plain`), not rendered (see the report of E4).
fn preview(wiring: &Arc<Wiring>) -> Div {
    let draft = wiring.draft().text();
    div()
        .flex_col()
        .child(text(wiring.say("code-editor.preview")))
        .child(TextView::new(vec![Block::Plain(draft)]).build())
}

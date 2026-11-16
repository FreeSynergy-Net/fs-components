//! The top line of the Notion layout: the way to the page in the column of the text, the buttons at the right edge.
//!
//! Pattern: none — one pure function from the reader to elements; what is clicked is reported through `Actions`.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::icon::{Glyph, Tone, icon, icon_stroked};

use super::head::{crumbs, lang_switch};
use super::notion::column;
use crate::page_info::WikiLabels;
use crate::reader::Reader;
use crate::view::Actions;

/// The least line of the symbols in the top line, in device pixels (the style's own is 0.9 px at 18 px and pale in Craft).
const HEAD_LINE: f32 = 1.5;

/// The top line: the way to the page in the column of the text, the buttons at the right edge.
pub(super) fn top_bar(
    reader: &Reader,
    labels: &WikiLabels,
    actions: &Actions,
    (path, title): (&str, &str),
    wide: bool,
) -> Div {
    let info = reader.page_info();
    // The buttons stand at the right edge; the way to the page stands in the column of the text (the same width and the same
    // padding as the page), so the two start at the same x.
    let mut buttons = div()
        .absolute()
        .right(16.0)
        .top(0.0)
        .h(44.0)
        .flex_row()
        .items_center()
        .gap(12.0)
        .child(lang_switch(&info, labels, actions));
    if reader.can_edit() {
        let (act, word, glyph) = if reader.editing().is_some() {
            (
                actions.save.clone(),
                reader.labels().save.clone(),
                Glyph::Check,
            )
        } else {
            (actions.edit.clone(), labels.edit.clone(), Glyph::Edit)
        };
        buttons = buttons.child(
            div()
                .class("fs-wk-btn")
                .flex_row()
                .items_center()
                .on_click(move |_| act())
                .child(icon(glyph, 16.0, Tone::Normal))
                .child(text(word).no_cursor()),
        );
    }
    buttons = buttons
        .child(icon_stroked(
            Glyph::History,
            18.0,
            Tone::Normal,
            Some(HEAD_LINE),
        ))
        .child(icon_stroked(
            Glyph::Star,
            18.0,
            Tone::Normal,
            Some(HEAD_LINE),
        ));
    div()
        .class("fs-wk-b-top")
        .id("fs-wk-top")
        .relative()
        .w_full()
        .child(
            div()
                .w_full()
                .h(44.0)
                .flex_row()
                .items_center()
                .justify_center()
                .child(column(
                    wide,
                    div().class("fs-wk-b-crumbs").child(crumbs(path, title)),
                )),
        )
        .child(buttons)
}

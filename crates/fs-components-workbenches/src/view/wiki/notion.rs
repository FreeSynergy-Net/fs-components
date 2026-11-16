//! The Notion-like layout: a calm sidebar, thin top line, a big title, a reading column of blocks, ticks at the edge.
//!
//! Pattern: Strategy (one of the layouts) — same three functions as the other layout.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use blinc_layout::text;
use blinc_layout::widgets::scroll::{Scroll, SharedScrollPhysics, scroll};
use fs_components_widgets::{Block, LinkHandler, TextView, count_tasks};

use super::blocks::BlockEditor;
use super::confluence::canonical;
use super::head::{notice_band, page_icon, state_chip};
use super::outline::{body, ticks};
use super::rows::{Look, room, row_height, tree_scroll};
use super::top::top_bar;
use super::tree::PageTree;
use super::widths;
use crate::page_info::WikiLabels;
use crate::reader::{Reader, Shown};
use crate::view::{Actions, locked_note, results_list, with_tasks};

/// The widest the column of the text is by default, in pixels.
const COLUMN: f32 = 720.0;

/// The window: sidebar left, the page right.
pub(crate) fn notion(
    search: impl ElementBuilder + 'static,
    side: impl ElementBuilder + 'static,
    main: impl ElementBuilder + 'static,
    width: f32,
    height: f32,
) -> Div {
    let stacked = widths::stacked(width);
    let mut side_column = div()
        .class("fs-wk-b-side")
        .w(widths::notion(width))
        .flex_col()
        .min_h(0.0)
        .child(div().class("fs-wk-b-head").flex_col().child(search))
        .child(side);
    let root = div().class("fs-wk-root").w(width).h(height);
    if stacked {
        // Narrow window: the sidebar stands above the page, so neither is squeezed into a few letters.
        // Its height is its head and whole rows of the tree (`side`), not a share of the window.
        side_column = side_column.class("fs-wk-stacked").flex_shrink_0();
        let page = div().flex_1().min_h(0.0).w_full().flex_col().child(main);
        root.flex_col().child(side_column).child(page)
    } else {
        root.flex_row()
            .child(side_column.flex_shrink_0().h_full())
            .child(main)
    }
}

/// The live part of the sidebar: a section heading and the tree (the search above it is the widget's).
pub(crate) fn side(
    reader: &Reader,
    tree: &PageTree,
    labels: &WikiLabels,
    actions: &Actions,
    (width, height): (f32, f32),
) -> Div {
    let rows = widths::stacked(width)
        .then(|| widths::stacked_rows(height, row_height(Look::Quiet), tree.rows(1500).len()));
    let list = if reader.term().is_empty() && reader.problem().is_none() {
        let selected = reader.current_path().map(|p| canonical(reader, p));
        let wide = widths::notion(width);
        tree_scroll(tree, selected.as_deref(), actions, Look::Quiet, wide, rows)
    } else {
        room(rows, Look::Quiet).child(
            scroll()
                .w_full()
                .h_full()
                .content(results_list(reader, actions)),
        )
    };
    let column = div()
        .flex_col()
        .w_full()
        .child(
            div().class("fs-wk-b-section").child(
                text(&labels.pages)
                    .class("fs-wk-b-section-text")
                    .no_cursor(),
            ),
        )
        .child(list);
    // Stacked, the column is as high as its content; beside the page it fills the height.
    if rows.is_some() {
        column
    } else {
        column.flex_1().min_h(0.0)
    }
}

/// The page: top line with the way and the buttons, the title, the language switch, the blocks.
pub(crate) fn main(
    reader: &Reader,
    labels: &WikiLabels,
    actions: &Actions,
    physics: Option<&SharedScrollPhysics>,
    blocks: Option<&BlockEditor>,
    report: &std::sync::Arc<dyn Fn(&str) + Send + Sync>,
    wide: bool,
) -> Div {
    let info = reader.page_info();
    let (path, document) = match reader.shown() {
        Shown::Document { path, document } => (canonical(reader, path), Some(document)),
        Shown::Failed { path, .. } => (path.clone(), None),
        _ => (String::new(), None),
    };
    let title = document.map_or(String::new(), |d| d.title.clone());
    let top = top_bar(reader, labels, actions, (&path, &title), wide);
    let mut page = div().class("fs-wk-b-page").flex_col().w_full();
    if let Some(document) = document {
        page = page.child(page_icon(info.emoji.as_deref()));
        // While editing, the title is a big field above the blocks (it holds the leading heading); else plain text.
        page = match (reader.editing(), blocks) {
            (Some(_), Some(b)) => page.child(b.title_view(labels, report)),
            _ => page.child(
                // The place of the title starts where the text of a block starts (6 px of block padding in), the text itself is
                // not padded: a padding on a text element is not drawn.
                div().class("fs-wk-b-lead").child(
                    text(&title)
                        .class("fs-wk-b-title")
                        .size(fs_components_primitives::TITLE_PX)
                        .no_cursor(),
                ),
            ),
        };
        let meta = div()
            .class("fs-wk-b-meta")
            .flex_row()
            .items_center()
            .child(state_chip(&info, labels));
        page = page.child(meta);
        if let Some(band) = notice_band(&info) {
            page = page.child(band);
        }
        if let (Some(_), Some(blocks)) = (reader.editing(), blocks) {
            page = page.child(blocks.view(labels, report));
        } else {
            let shown = body(document);
            page = page
                .child(block_list(&shown, reader, actions))
                .child(locked_note(reader, &shown));
        }
    } else if let Shown::Failed { reason, .. } = reader.shown() {
        page = page.child(div().class("fs-rd-failed").child(text(reason.clone())));
    } else {
        page = page.child(text(&reader.labels().nothing_open));
    }
    let scroller = physics.map_or_else(blinc_layout::widgets::scroll::scroll, |p| {
        Scroll::with_physics(p.clone())
    });
    let centred = div()
        .flex_row()
        .justify_center()
        .w_full()
        .child(column(wide, page));
    let edge = document.map_or_else(div, ticks);
    let middle = div()
        .flex_col()
        .flex_1()
        .min_w(0.0)
        .min_h(0.0)
        .child(top)
        .child(
            div()
                .flex_1()
                .min_h(0.0)
                .w_full()
                .child(scroller.w_full().h_full().no_bounce().content(centred)),
        );
    div()
        .flex_row()
        .flex_1()
        .min_w(0.0)
        .h_full()
        .child(middle)
        .child(edge)
}

/// The column of the text: 720 px wide and centred, or the whole width of the window beside the sidebar ("full width", a setting
/// of the page in Notion's page menu).
pub(super) fn column(wide: bool, inner: Div) -> Div {
    let column = div().w_full();
    if wide {
        column.child(inner)
    } else {
        column.max_w(COLUMN).child(inner)
    }
}

/// The blocks of a page, one element each; a click on a block starts editing there, a click on a checkbox ticks it
/// (and starts no edit: the box says so through `ticked`, which the block's own click reads and clears).
fn block_list(blocks: &[Block], reader: &Reader, actions: &Actions) -> Div {
    let mut column = div().flex_col().w_full().gap(1.0);
    let ticked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut first = 0;
    for (i, block) in blocks.iter().enumerate() {
        let edit = actions.block.clone();
        let follow = actions.follow.clone();
        let handler: LinkHandler = std::sync::Arc::new(move |href| follow(href));
        let mut tick = actions.clone();
        let (task, flag) = (actions.task.clone(), std::sync::Arc::clone(&ticked));
        tick.task = std::sync::Arc::new(move |n, done| {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
            task(n, done);
        });
        let flag = std::sync::Arc::clone(&ticked);
        let view = with_tasks(
            TextView::new(vec![block.clone()])
                .reading()
                .on_link(handler),
            reader,
            &tick,
            first,
        );
        first += count_tasks(std::slice::from_ref(block));
        column = column.child(
            div()
                .class("fs-wk-block")
                .w_full()
                .on_click(move |_| {
                    if !flag.swap(false, std::sync::atomic::Ordering::SeqCst) {
                        edit(i);
                    }
                })
                .child(view.build()),
        );
    }
    column
}

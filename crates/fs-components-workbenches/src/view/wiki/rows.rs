//! The rows of the page tree as elements.
//!
//! Pattern: Adapter (arena rows to Blinc elements); a click on a folder toggles it, a click on a page opens it.

use blinc_cn::components::tooltip::TooltipBuilder;

use blinc_layout::InstanceKey;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::icon::{Glyph, Tone, icon, icon_stroked};
use fs_components_widgets::list_view::fit_cell;

use super::tree::PageTree;
use crate::view::Actions;

/// Width of the arrow place (and of the blank that stands in for it), the size of its icon: the title of a row
/// starts at the same x whether the row has an arrow or not.
const ARROW: f32 = 14.0;

/// The least line of the arrow in device pixels: the style's own line (0.7 px at this size) is not seen on the selection.
const ARROW_LINE: f32 = 1.6;

/// What a row spends besides the title and the indent: its two paddings, the arrow place, the icon and the gaps.
const ROW_FIXED: f32 = 16.0 + ARROW + 17.0 + 12.0;
/// One level of indent (`--space-3`).
const INDENT: f32 = 12.0;

/// The title of a row as it fits a tree `width` px wide at `depth`: cut with an ellipsis, the whole title as the tooltip of
/// the cut one. Blinc has no ellipsis of its own (`fit_cell` measures).
fn title(label: &str, key: &str, depth: u16, width: f32, folder: bool) -> Div {
    let room = width - ROW_FIXED - INDENT * f32::from(depth);
    let shown = fit_cell(label, room);
    let cut = shown != label;
    let face = move |shown: &str| {
        let t = text(shown).no_cursor().no_wrap();
        if folder { t.class("fs-wk-folder") } else { t }
    };
    if !cut {
        return div().id(format!("fs-wk-label-{key}")).child(face(&shown));
    }
    let tip = label.to_owned();
    let tkey = InstanceKey::explicit(format!("fs-wk-tip-{key}"));
    let trigger = move || div().child(face(&shown));
    div()
        .id(format!("fs-wk-label-{key}"))
        .child(TooltipBuilder::with_key(trigger, tkey).text(tip))
}

/// Most rows built at once: a guard, the open folders of a real tree stay far below.
const MOST_ROWS: usize = 1500;

/// How the rows look.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Look {
    /// Folder and page icons, a bar at the selected page (Confluence-like).
    Icons,
    /// Text only, soft surface at the selected page (Notion-like).
    Quiet,
}

/// One step of the space scale (`--space-3`) per level, in one box that every row has (also at depth 0, empty), so the
/// row's gap lies once before the title at every depth: a child stands exactly one step right of its parent.
fn indent(depth: u16, key: &str) -> Div {
    let mut steps = div()
        .id(format!("fs-wk-steps-{key}"))
        .flex_row()
        .flex_shrink_0();
    for _ in 0..depth {
        steps = steps.child(div().class("fs-wk-indent"));
    }
    steps
}

/// The height of a row of the tree in `look` (the stylesheet's `.fs-wk-row` and `.fs-wk-row-quiet`).
pub(super) fn row_height(look: Look) -> f32 {
    match look {
        Look::Icons => 30.0,
        Look::Quiet => 28.0,
    }
}

/// The place of the tree: it fills its room, or (`rows`: a stacked column) is exactly that many rows high, so no row is cut.
pub(super) fn room(rows: Option<usize>, look: Look) -> Div {
    match rows {
        // Row counts are tiny; the cast to f32 is exact.
        #[allow(clippy::cast_precision_loss)]
        Some(n) => div()
            .flex_shrink_0()
            .w_full()
            .h(n as f32 * row_height(look)),
        None => div().flex_1().min_h(0.0).w_full(),
    }
}

/// The tree in a scroll area filling its room (`rows`: exactly that many rows high).
pub(super) fn tree_scroll(
    tree: &PageTree,
    selected: Option<&str>,
    actions: &Actions,
    look: Look,
    width: f32,
    rows: Option<usize>,
) -> Div {
    let mut column = div().flex_col().w_full().class("fs-wk-tree");
    for row in tree.rows(MOST_ROWS) {
        let node = row.node;
        let here = node.page && selected == Some(node.key.as_str());
        let key = node.key.clone();
        // A plain folder toggles on any click; a page opens by its title (and a page with sub-pages expands by its arrow).
        let toggle = std::sync::Arc::new({
            let (toggle, key) = (actions.toggle.clone(), key.clone());
            move || toggle(&key)
        });
        let open = std::sync::Arc::new({
            let (open, key) = (actions.open.clone(), key);
            move || open(&key)
        });
        let mut line = div()
            .class("fs-wk-row")
            .flex_row()
            .items_center()
            .w_full()
            .id(format!("fs-wk-row-{}", node.key));
        line = line.child(indent(node.depth, &node.key));
        if look == Look::Quiet {
            line = line.class("fs-wk-row-quiet");
        }
        if here {
            line = line.class("fs-wk-row-here");
        }
        let plain_folder = node.folder && !node.page;
        if plain_folder {
            let toggle = toggle.clone();
            line = line.on_click(move |_| toggle());
        } else if !node.folder {
            let open = open.clone();
            line = line.on_click(move |_| open());
        }
        if node.folder {
            let chevron = if row.open {
                Glyph::Down
            } else {
                Glyph::ChevronRight
            };
            // The open row that is shown has the selection as its ground: the muted tone is gone there (finding B2).
            let tone = if here { Tone::Normal } else { Tone::Muted };
            let arrow = div()
                .w(ARROW)
                .flex_shrink_0()
                .id(format!("fs-wk-arrow-{}", node.key))
                .child(icon_stroked(chevron, ARROW, tone, Some(ARROW_LINE)));
            line = line.child(if plain_folder {
                arrow
            } else {
                arrow.on_click(move |_| toggle())
            });
        } else {
            line = line.child(
                div()
                    .w(ARROW)
                    .flex_shrink_0()
                    .id(format!("fs-wk-arrow-{}", node.key)),
            );
        }
        let glyph = if plain_folder {
            Glyph::Folder
        } else {
            Glyph::FileText
        };
        let kind = match look {
            Look::Icons if plain_folder => icon(glyph, 17.0, Tone::Kind),
            Look::Icons => icon(glyph, 17.0, Tone::Muted),
            // The quiet tree: a small icon before every page and folder (the place where the emoji of a page goes).
            Look::Quiet => icon(glyph, 16.0, Tone::Kind),
        };
        let label = title(&node.label, &node.key, node.depth, width, plain_folder);
        line = if node.folder && node.page {
            // Everything after the arrow, down to the free rest of the row, opens the page; the arrow expands.
            line.child(
                div()
                    .flex_row()
                    .flex_1()
                    .h_full()
                    .items_center()
                    .class("fs-wk-body")
                    .id(format!("fs-wk-title-{}", node.key))
                    .on_click(move |_| open())
                    .child(kind)
                    .child(label),
            )
        } else {
            line.child(kind).child(label)
        };
        column = column.child(line);
    }
    room(rows, look).child(scroll().w_full().h_full().content(column))
}

//! One line of a menu panel: mark, symbol, label, shortcut, arrow; or a separator. The reason of a disabled line is a tooltip.
//!
//! Pattern: Adapter (a `MenuEntry` to elements). Only the outermost element of a line takes a press. The shortcut is shown at the
//! right, never run here (M2). Roles follow WAI-ARIA: `menuitem`, `menuitemcheckbox`, `menuitemradio`, `separator`.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_theme::{ColorToken, ThemeState};
use fs_components_widgets::disabled::{Disabled, Labels, Why};
use fs_components_widgets::{Tone, icon_named_stroked};

use crate::menu::{Mark, MenuEntry};
use crate::menu_view::{Callbacks, Look};
use crate::styles::{
    ARROW_WIDTH, ICON_LEAST_STROKE, ICON_SIZE, ICON_WIDTH, LINE_HEIGHT, MARK_WIDTH, MENU_TEXT,
    ROW_BUTTON, ROW_GAP, SEPARATOR_HEIGHT, SEPARATOR_MARGIN,
};

/// The arrow on a line that opens a next level: to the right, or to the left where the next level folds to the left.
pub(crate) fn arrow(left: bool) -> &'static str {
    if left { "\u{25c2}" } else { "\u{25b8}" }
}

/// What a line shows in the column of marks: a tick for a check or the classic chosen line, a dot for a chosen radio line.
fn mark(entry: &MenuEntry) -> &'static str {
    match (entry.mark, entry.checked) {
        (_, false) => "",
        (Mark::Radio, true) => "\u{25cf}",
        _ => "\u{2713}",
    }
}

fn role(entry: &MenuEntry) -> &'static str {
    match entry.mark {
        Mark::Check => "menuitemcheckbox",
        Mark::Radio => "menuitemradio",
        _ if entry.checked => "menuitemradio",
        _ => "menuitem",
    }
}

/// The thin line between two parts: no press, no focus.
fn separator() -> Div {
    let border = ThemeState::get().color(ColorToken::Border);
    div()
        .class("fs-app-menu-separator")
        .attr("role", "separator")
        .w_full()
        .h(SEPARATOR_HEIGHT)
        .my(SEPARATOR_MARGIN)
        .bg(border)
}

/// A line with a next level opens it, shows an arrow towards the side it opens to, and says `aria-expanded` while it is open; a
/// line in a group with a chosen one leaves room for the mark. A disabled line keeps the size of a free one and is drawn by
/// `Disabled::wrap` (`aria-disabled`, roving tab stop): its reason is a tooltip, never a second line.
#[allow(clippy::too_many_lines)] // one line of a menu: mark, symbol, words, shortcut, arrow and the attributes
pub(crate) fn line(
    index: usize,
    position: usize,
    entry: &MenuEntry,
    (marks, look): (bool, Look),
    calls: &Callbacks,
) -> Div {
    if entry.separator {
        return separator();
    }
    if !entry.row.is_empty() {
        return icon_row(index, entry, calls);
    }
    let (choose, sub, id) = (
        Arc::clone(&calls.choose),
        Arc::clone(&calls.sub),
        entry.id.clone(),
    );
    let opens = !entry.children.is_empty();
    let mut row = div().flex_row().items_center().w_full().h_fit();
    if marks {
        row = row.child(
            div()
                .w(MARK_WIDTH)
                .flex_shrink_0()
                .child(label(mark(entry))),
        );
    }
    if look.icons {
        let mut cell = div()
            .id(format!("fs-app-menu-icon-{}", entry.id))
            .w(ICON_WIDTH)
            .flex_shrink_0();
        if let Some(name) = entry.icon.as_deref() {
            cell = cell.child(icon_named_stroked(
                name,
                ICON_SIZE,
                Tone::Normal,
                Some(ICON_LEAST_STROKE),
            ));
        }
        row = row.child(cell);
    }
    row = row.child(div().flex_grow().flex_col().child(label(&entry.label)));
    if let Some(shortcut) = &entry.shortcut {
        let muted = ThemeState::get().color(ColorToken::TextSecondary);
        row = row.child(
            div().flex_shrink_0().pl(8.0).child(
                text(shortcut)
                    .size(MENU_TEXT)
                    .color(muted)
                    .no_wrap()
                    .no_cursor()
                    .pointer_events_none(),
            ),
        );
    }
    if opens {
        row = row.child(label(arrow(look.arrow_left)));
    } else if look.arrow_room && entry.shortcut.is_some() {
        row = row.child(div().w(ARROW_WIDTH).flex_shrink_0());
    }
    let mut line = div()
        .id(format!("fs-app-menu-entry-{}", entry.id))
        .class("cn-menu-item")
        .class("cn-menubar-item")
        .attr("role", role(entry))
        .attr("tabindex", "-1")
        .attr_when("aria-checked", "true", entry.checked)
        // A check or radio line that is off says so: without the attribute a reader hears no state at all.
        .attr_when("aria-checked", "false", entry.has_mark() && !entry.checked)
        .attr_when("aria-haspopup", "menu", opens)
        .attr_when(
            "aria-expanded",
            "true",
            opens && look.open_sub == Some(position),
        )
        .attr_when(
            "aria-keyshortcuts",
            entry.shortcut_aria.as_deref().unwrap_or(""),
            entry.shortcut_aria.is_some(),
        )
        .w_full()
        .flex_row()
        .items_center()
        // The same as `.cn-menubar-item`: class rules can drop on a rebuilt overlay (a note in `blinc_cn`).
        .px(3.0)
        .child(row);
    // A line is the Explorer's 32 px, fixed (the class paddings of the framework made it 41). A line whose words will wrap is as
    // tall as its text needs (a fixed height would cut the words off). A reason never changes the height.
    #[allow(clippy::cast_precision_loss)] // a label is a few dozen characters
    let wraps = entry.label.chars().count() as f32 * MENU_TEXT * 0.5 > look.width - 20.0;
    line = if wraps {
        line.h_fit()
    } else {
        line.h(LINE_HEIGHT)
    };
    if entry.disabled {
        return lock(line, &entry.id, entry.reason.as_deref(), move || {
            choose(index, &id);
        });
    }
    line.cursor_pointer().on_click(move |ctx| {
        if opens {
            sub(
                index,
                position,
                (ctx.bounds_x, ctx.bounds_y, ctx.bounds_width),
            );
        } else {
            choose(index, &id);
        }
    })
}

/// A disabled line or button: with a reason it is drawn by `Disabled::wrap` (the one place, id `fs-app-menu-entry-<id>`, roving
/// tab stop, the reason as tooltip); without one there is nothing to say, so only `aria-disabled` is set. A press still goes to
/// `press` (the shell refuses a disabled entry and reports its reason itself) and opens the tooltip.
fn lock(
    control: Div,
    id: &str,
    reason: Option<&str>,
    press: impl Fn() + Send + Sync + 'static,
) -> Div {
    match reason {
        Some(reason) => Disabled::new(
            &format!("fs-app-menu-entry-{id}"),
            Why::Now(reason.to_owned()),
            &Labels::default(),
        )
        .roving()
        .swallow(press)
        .wrap(control),
        None => control
            .attr("aria-disabled", "true")
            .on_click(move |_| press()),
    }
}

/// A row of symbol buttons (group `inline_icons`): each button is one command, named by its label for assistive technology
/// (`aria-label`), a locked one has its reason as a tooltip. A press takes the same road as a line (`choose`), so a
/// locked button reports its reason through the shell and does not run. Only the button takes the press.
fn icon_row(index: usize, entry: &MenuEntry, calls: &Callbacks) -> Div {
    let mut row = div()
        .id(format!("fs-app-menu-entry-{}", entry.id))
        .class("fs-app-menu-icon-row")
        .attr("role", "group")
        .flex_row()
        .items_center()
        // Grouped at the left, as the Explorer groups its quick actions, not stretched over the whole width.
        .justify_start()
        .gap_px(ROW_GAP)
        .w_full()
        .h_fit()
        .py(2.0)
        .px(3.0);
    for button in &entry.row {
        let (choose, id) = (Arc::clone(&calls.choose), button.id.clone());
        let mut cell = div()
            .id(format!("fs-app-menu-entry-{}", button.id))
            .class("cn-menu-item")
            .class("cn-menubar-item")
            .attr("role", "menuitem")
            .attr("tabindex", "-1")
            .attr("aria-label", button.label.as_str())
            .attr_when(
                "aria-keyshortcuts",
                button.shortcut_aria.as_deref().unwrap_or(""),
                button.shortcut_aria.is_some(),
            )
            .w(ROW_BUTTON)
            .h(ROW_BUTTON)
            .flex_shrink_0()
            .flex_row()
            .items_center()
            .justify_center();
        if let Some(name) = button.icon.as_deref() {
            cell = cell.child(icon_named_stroked(
                name,
                ICON_SIZE,
                Tone::Normal,
                Some(ICON_LEAST_STROKE),
            ));
        }
        row = row.child(if button.disabled {
            lock(cell, &button.id, button.reason.as_deref(), move || {
                choose(index, &id);
            })
        } else {
            cell.cursor_pointer().on_click(move |_| choose(index, &id))
        });
    }
    row
}

pub(crate) fn label(words: &str) -> blinc_layout::text::Text {
    text(words)
        .size(MENU_TEXT)
        .no_cursor()
        .pointer_events_none()
}

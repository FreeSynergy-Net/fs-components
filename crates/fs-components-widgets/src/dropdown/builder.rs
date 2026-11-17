//! The builder of a dropdown and its overlay.
//!
//! Pattern: Builder. See the module for why this is not `blinc_cn`'s dropdown.

use std::sync::Arc;

use super::panel::{self, Entry, height};
use super::place::{Align, Placed, Rect, place};
use blinc_core::BlincContextState;
use blinc_layout::InstanceKey;
use blinc_layout::click_outside;
use blinc_layout::div::{Div, div};
use blinc_layout::overlay_state::overlay_stack;
use blinc_layout::widgets::overlay_stack::{OverlayBuilder, OverlayHandle};

type Trigger = Arc<dyn Fn(bool) -> Div + Send + Sync>;

/// A dropdown: a trigger (given whether the menu is open) and the entries.
pub struct Dropdown {
    trigger: Trigger,
    entries: Vec<Entry>,
    width: f32,
    align: Align,
    key: InstanceKey,
}

/// A dropdown with this trigger. The key of its state comes from the place of the call and the number of the call in the frame.
#[track_caller]
pub fn dropdown(trigger: impl Fn(bool) -> Div + Send + Sync + 'static) -> Dropdown {
    // Frame log probe: one stateful widget more in this build (a no-op unless `FS_FRAME_LOG` is set).
    fs_components_primitives::frame_probe::count_stateful();
    Dropdown {
        trigger: Arc::new(trigger),
        entries: Vec::new(),
        width: 220.0,
        align: Align::Start,
        key: InstanceKey::new("fs-dropdown"),
    }
}

impl Dropdown {
    /// A line the person can choose.
    #[must_use]
    pub fn item(
        mut self,
        label: impl Into<String>,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        self.entries.push(Entry::Line {
            label: label.into(),
            action: Some(Arc::new(action)),
        });
        self
    }

    /// A line that is shown and cannot be chosen.
    #[must_use]
    pub fn item_disabled(mut self, label: impl Into<String>) -> Self {
        self.entries.push(Entry::Line {
            label: label.into(),
            action: None,
        });
        self
    }

    /// A thin line between groups.
    #[must_use]
    pub fn separator(mut self) -> Self {
        self.entries.push(Entry::Separator);
        self
    }

    /// The width of the panel.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Which edge of the trigger the panel lines up with.
    #[must_use]
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// The trigger, with the panel opening on a press.
    #[must_use]
    pub fn into_element(self) -> Div {
        let key = self.key.get().to_owned();
        let ctx = BlincContextState::get();
        let open = ctx.use_signal_keyed(&key, || false);
        let handle = ctx.use_signal_keyed(&self.key.derive("handle"), || None::<u64>);
        let trigger_id = format!("fs-dropdown-trigger-{key}");
        let (closed, opened) = (Arc::clone(&self.trigger), Arc::clone(&self.trigger));
        let (entries, width, align) = (self.entries, self.width, self.align);
        let inside = trigger_id.clone();
        div()
            .id(trigger_id)
            .attr("tabindex", "0")
            .attr("aria-haspopup", "menu")
            .attr("data-focus-delegate", "true")
            .w_fit()
            .cursor_pointer()
            .child(div().show_or(open, move || opened(true), move || closed(false)))
            .on_click(move |e| {
                if open.get() {
                    if let Some(raw) = handle.get() {
                        OverlayHandle::from_raw(raw).close();
                    }
                    return;
                }
                let anchor = Rect {
                    x: e.bounds_x,
                    y: e.bounds_y,
                    width: e.bounds_width,
                    height: e.bounds_height,
                };
                let window = overlay_stack().lock().map_or((0.0, 0.0), |s| s.viewport());
                let placed = place(anchor, (width, height(&entries)), window, align);
                let shown = open_panel(&entries, &placed, &inside, move || {
                    open.set(false);
                    handle.set(None);
                });
                handle.set(Some(shown.raw()));
                open.set(true);
            })
    }
}

fn open_panel(
    entries: &[Entry],
    placed: &Placed,
    trigger_id: &str,
    on_close: impl Fn() + Send + Sync + 'static,
) -> OverlayHandle {
    let raw = overlay_stack()
        .lock()
        .map_or(0, |s| s.peek_next_handle_id());
    let me = OverlayHandle::from_raw(raw);
    let id = format!("fs-dropdown-panel-{raw}");
    let outside_key = format!("fs-dropdown:{raw}");
    let (list, id_in, size) = (
        entries.to_vec(),
        id.clone(),
        (placed.rect.width, placed.rect.height),
    );
    let flipped = placed.flipped;
    let closing_key = outside_key.clone();
    let shown = OverlayBuilder::dropdown()
        .at(placed.rect.x, placed.rect.y)
        .on_close(move |_| {
            click_outside::unregister_click_outside(&closing_key);
            on_close();
        })
        .content(move || panel::build(&list, &id_in, size, flipped, me))
        .show();
    click_outside::register_click_outside_multi(
        &outside_key,
        &[id, trigger_id.to_owned()],
        move || me.close(),
    );
    shown
}

#[cfg(test)]
mod tests {
    use super::super::panel::{Entry, height};

    #[test]
    fn the_height_is_the_lines_and_the_border() {
        let line = || Entry::Line {
            label: "a".into(),
            action: None,
        };
        assert_eq!(
            height(&[line(), Entry::Separator, line()]),
            32.0 + 9.0 + 32.0 + 2.0
        );
    }
}

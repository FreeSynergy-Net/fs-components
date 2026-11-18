//! The drop zone as an element and its handle.
//!
//! Pattern: Facade + Observer — a clonable handle over the "files are over it" mark; a change draws the zone again through `Live`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blinc_layout::div::{Div, div};
use blinc_layout::event_router::keyboard_focused_node;
use blinc_layout::prelude::text;

use super::labels::Labels;
use crate::Element;
use crate::keys::{Key, bind_keys};
use crate::live::Live;

/// What the zone calls.
type Act = Arc<dyn Fn() + Send + Sync>;

/// A drop zone. Clones share the mark and the callbacks.
#[derive(Clone)]
pub struct DropZone {
    id: String,
    labels: Labels,
    title: Option<String>,
    over: Arc<AtomicBool>,
    live: Live,
    on_pick: Option<Act>,
    on_drop: Option<Act>,
}

impl DropZone {
    /// A zone with the standard words.
    pub fn new(labels: &Labels) -> Self {
        Self {
            id: "drop-zone".to_owned(),
            labels: labels.clone(),
            title: None,
            over: Arc::new(AtomicBool::new(false)),
            live: Live::new(),
            on_pick: None,
            on_drop: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// What the files are ("Photos"): said before the hint, also to a screen reader.
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    /// Call `handler` on Enter, Space or a click: the host opens its file chooser. This is the way that needs no dragging.
    #[must_use]
    pub fn on_pick(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_pick = Some(Arc::new(handler));
        self
    }

    /// Call `handler` when files were dropped on the zone (the paths are the host's to read).
    #[must_use]
    pub fn on_drop(mut self, handler: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_drop = Some(Arc::new(handler));
        self
    }

    /// Whether files hover over the zone.
    pub fn is_over(&self) -> bool {
        self.over.load(Ordering::Relaxed)
    }

    /// Files came over the zone (`true`) or left it; only a change draws again.
    pub fn drag_over(&self, over: bool) {
        if self.over.swap(over, Ordering::Relaxed) != over {
            self.live.refresh_now();
        }
    }

    /// The keyboard or pointer way: ask the host to choose files.
    pub fn pick(&self) {
        if let Some(handler) = &self.on_pick {
            handler();
        }
    }

    /// Files were dropped.
    pub fn dropped(&self) {
        self.drag_over(false);
        if let Some(handler) = &self.on_drop {
            handler();
        }
    }

    /// What a screen reader says: the title, then the hint.
    pub fn spoken(&self) -> String {
        match &self.title {
            Some(title) => format!("{title}. {}", self.labels.hint),
            None => self.labels.hint.clone(),
        }
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.view("drop-zone", move || this.render())
    }

    fn render(&self) -> Div {
        let (keys, click, enter, leave, drop) = (
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        );
        let over = self.is_over();
        let shown = if over {
            self.labels.over.clone()
        } else {
            self.labels.hint.clone()
        };
        let mut words = div().flex_col();
        if let Some(title) = &self.title {
            words = words.child(text(title.clone()).class("fs-drop-zone-text"));
        }
        words = words.child(text(shown).class("fs-drop-zone-text"));
        let mut zone = bind_keys(
            div()
                .id(&self.id)
                .class("fs-drop-zone")
                .attr("role", "button")
                .attr("aria-label", self.spoken())
                .child(words),
            Arc::new(move |event| match event.key {
                Key::Activate | Key::Enter | Key::Space => {
                    keys.pick();
                    true
                }
                _ => false,
            }),
        )
        .on_click(move |ctx| {
            // Enter and Space reach the zone as a click too: the key handler above already picked then.
            if keyboard_focused_node() != Some(ctx.node_id) {
                click.pick();
            }
        })
        .on_file_drag_over(move |_| enter.drag_over(true))
        .on_file_drag_leave(move |_| leave.drag_over(false))
        .on_file_drop(move |_| drop.dropped());
        if over {
            zone = zone.class("fs-drop-zone-over");
        }
        zone
    }
}

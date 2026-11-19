//! `segmented`: a few exclusive choices in one row ("user | system", "list | grid"); exactly one is chosen.
//!
//! Pattern: State (the chosen segment, pure) + Observer (`Live` draws the segments again inside the group, the group itself is
//! not drawn again so the keyboard stays). The group is ONE tab stop (WAI-ARIA radio group): Left and Up, Right and Down move
//! the choice (it follows the keyboard), Home and End jump; a disabled segment is skipped by the keys and ignores the pointer.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_widgets::Element;
use fs_components_widgets::keys::Key;
use fs_components_widgets::live::Live;

use super::activatable;

/// One segment.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// What the caller gets back.
    pub key: String,
    /// What the person reads.
    pub label: String,
    /// Cannot be chosen.
    pub disabled: bool,
}

type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

/// The group. Clones share the choice and the callback.
#[derive(Clone)]
pub struct Segmented {
    id: String,
    segments: Arc<Vec<Segment>>,
    chosen: Arc<Mutex<Option<usize>>>,
    disabled: bool,
    live: Live,
    on_change: Option<OnChange>,
}

impl Segmented {
    /// An empty group; add segments with [`Segmented::segment`].
    pub fn new() -> Self {
        Self {
            id: "segmented".to_owned(),
            segments: Arc::new(Vec::new()),
            chosen: Arc::new(Mutex::new(None)),
            disabled: false,
            live: Live::new(),
            on_change: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Add a segment. The first one that can be chosen is chosen at the start.
    #[must_use]
    pub fn segment(mut self, key: &str, label: &str, disabled: bool) -> Self {
        Arc::make_mut(&mut self.segments).push(Segment {
            key: key.to_owned(),
            label: label.to_owned(),
            disabled,
        });
        let first = self.segments.iter().position(|s| !s.disabled);
        *self.slot() = first;
        self
    }

    /// Nothing is chosen (the first segment is not chosen for the person): the form of a field that has no value yet.
    #[must_use]
    pub fn unchosen(self) -> Self {
        *self.slot() = None;
        self
    }

    /// The whole group cannot be changed.
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Call `handler` with the key of the new choice after a change by the person.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> std::sync::MutexGuard<'_, Option<usize>> {
        self.chosen.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key of the chosen segment.
    pub fn chosen(&self) -> Option<String> {
        let i = (*self.slot())?;
        self.segments.get(i).map(|s| s.key.clone())
    }

    /// Choose from the program (no callback); false when there is no such segment or it is disabled.
    pub fn set(&self, key: &str) -> bool {
        let Some(i) = self
            .segments
            .iter()
            .position(|s| s.key == key && !s.disabled)
        else {
            return false;
        };
        *self.slot() = Some(i);
        self.live.refresh_now();
        true
    }

    /// Choose as the person does; false (nothing happens) when disabled, unknown or already chosen.
    pub fn choose(&self, key: &str) -> bool {
        if self.disabled || self.chosen().as_deref() == Some(key) || !self.set(key) {
            return false;
        }
        if let Some(handler) = &self.on_change {
            handler(key);
        }
        true
    }

    /// Move the choice by `delta` over the segments that can be chosen, without wrapping.
    pub fn step(&self, delta: isize) -> bool {
        let pickable: Vec<usize> = (0..self.segments.len())
            .filter(|&i| !self.segments[i].disabled)
            .collect();
        let at = (*self.slot()).and_then(|n| pickable.iter().position(|&i| i == n));
        let to = at.unwrap_or(0).saturating_add_signed(delta);
        let Some(&i) = pickable.get(to.min(pickable.len().saturating_sub(1))) else {
            return false;
        };
        let key = self.segments[i].key.clone();
        self.choose(&key)
    }

    /// The element.
    pub fn build(&self) -> impl Element + use<> {
        let inner = self.clone();
        let root = div()
            .id(&self.id)
            .class("fs-seg")
            .attr("role", "radiogroup")
            .child(self.live.region("segments", move || inner.render()));
        if self.disabled {
            return root.class("fs-seg-disabled").attr("disabled", "true");
        }
        let keys = self.clone();
        activatable(
            root,
            Arc::new(move |event| {
                match event.key {
                    Key::Left | Key::Up => keys.step(-1),
                    Key::Right | Key::Down => keys.step(1),
                    Key::Home => keys.step(isize::MIN / 2),
                    Key::End => keys.step(isize::MAX / 2),
                    _ => return false,
                };
                true
            }),
            Arc::new(|| {}),
        )
    }

    fn render(&self) -> Div {
        let at = *self.slot();
        let mut row = div().class("fs-seg-options");
        for (i, segment) in self.segments.iter().enumerate() {
            let mut item = div()
                .id(format!("{}-{}", self.id, segment.key))
                .class("fs-seg-item")
                .attr("role", "radio")
                .attr("aria-checked", if at == Some(i) { "true" } else { "false" });
            if at == Some(i) {
                item = item.class("fs-seg-on");
            }
            if segment.disabled || self.disabled {
                item = item.class("fs-seg-off");
            }
            let (this, key) = (self.clone(), segment.key.clone());
            row = row.child(
                item.child(
                    text(segment.label.clone())
                        .id(format!("{}-{}-label", self.id, segment.key))
                        .class("fs-seg-label"),
                )
                .on_click(move |_| {
                    this.choose(&key);
                }),
            );
        }
        row
    }
}

impl Default for Segmented {
    fn default() -> Self {
        Self::new()
    }
}

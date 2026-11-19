//! A stack of disclosures.
//!
//! Pattern: Facade + Observer — a clonable handle over one open mark per section; a change draws the stack again through `Live`.
//! The pure rule (`toggle` in the single way closes the others) is `Marks::press`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};

use super::parts::{Press, header, panel};
use crate::live::Live;

/// One section: a key, a title and what the panel shows when it is open.
#[non_exhaustive]
#[derive(Clone)]
pub struct Section {
    /// What the caller gets back; part of the ids (`<id>-head-<key>`, `<id>-panel-<key>`).
    pub key: String,
    /// The header's words.
    pub title: String,
    content: Arc<dyn Fn() -> Div + Send + Sync>,
}

impl Section {
    /// A section; `content` is built when the panel is open.
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        content: impl Fn() -> Div + Send + Sync + 'static,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            content: Arc::new(content),
        }
    }
}

/// Which sections are open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    open: Vec<bool>,
    single: bool,
}

impl Marks {
    /// All closed.
    pub fn new(len: usize, single: bool) -> Self {
        Self {
            open: vec![false; len],
            single,
        }
    }

    /// Whether section `i` is open.
    pub fn is_open(&self, i: usize) -> bool {
        self.open.get(i).copied().unwrap_or(false)
    }

    /// Whether the header of section `i` cannot be pressed: in the single way the open one cannot close itself (APG).
    pub fn is_inert(&self, i: usize) -> bool {
        self.single && self.is_open(i)
    }

    /// Press the header of section `i`: `Some(now open)`, or `None` when nothing changed (inert or no such section).
    pub fn press(&mut self, i: usize) -> Option<bool> {
        if i >= self.open.len() || self.is_inert(i) {
            return None;
        }
        let now = !self.open[i];
        if self.single {
            self.open.iter_mut().for_each(|o| *o = false);
        }
        self.open[i] = now;
        Some(now)
    }

    /// Open or close section `i` from the program (in the single way an open one closes the others).
    pub fn set(&mut self, i: usize, open: bool) -> bool {
        if i >= self.open.len() || self.is_open(i) == open {
            return false;
        }
        if self.single && open {
            self.open.iter_mut().for_each(|o| *o = false);
        }
        self.open[i] = open;
        true
    }
}

type OnToggle = Arc<dyn Fn(&str, bool) + Send + Sync>;

/// An accordion. Clones share the marks and the callback.
#[derive(Clone)]
pub struct Accordion {
    id: String,
    level: u8,
    sections: Arc<Vec<Section>>,
    marks: Arc<Mutex<Marks>>,
    live: Live,
    on_toggle: Option<OnToggle>,
}

impl Accordion {
    /// An accordion of these sections, all closed; several may be open at once.
    pub fn new(sections: Vec<Section>) -> Self {
        let marks = Marks::new(sections.len(), false);
        Self {
            id: "accordion".to_owned(),
            level: 3,
            sections: Arc::new(sections),
            marks: Arc::new(Mutex::new(marks)),
            live: Live::new(),
            on_toggle: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The level of the headings the headers are (`aria-level`, 1 to 6; 3 unless told otherwise).
    #[must_use]
    pub fn level(mut self, level: u8) -> Self {
        self.level = level.clamp(1, 6);
        self
    }

    /// Only one section is open at a time.
    #[must_use]
    pub fn single(self) -> Self {
        *self.slot() = Marks::new(self.sections.len(), true);
        self
    }

    /// Call `handler` with the key of the section and whether it is open now, after a press.
    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(&str, bool) + Send + Sync + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, Marks> {
        self.marks.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether the section with this key is open.
    pub fn is_open(&self, key: &str) -> bool {
        self.sections
            .iter()
            .position(|s| s.key == key)
            .is_some_and(|i| self.slot().is_open(i))
    }

    /// Open or close a section from the program.
    pub fn set_open(&self, key: &str, open: bool) {
        let Some(i) = self.sections.iter().position(|s| s.key == key) else {
            return;
        };
        if self.slot().set(i, open) {
            self.live.refresh_now();
        }
    }

    fn press(&self, i: usize) {
        let now = self.slot().press(i);
        let Some(now) = now else { return };
        self.live.refresh_now();
        if let Some(handler) = &self.on_toggle {
            handler(&self.sections[i].key, now);
        }
    }

    /// The accordion, to place in a layout.
    pub fn build(&self) -> Div {
        let this = self.clone();
        div()
            .id(&self.id)
            .class("fs-accordion")
            .child(self.live.view("accordion", move || this.render()))
    }

    fn render(&self) -> Div {
        let marks = self.slot().clone();
        let mut stack = div().flex_col();
        for (i, section) in self.sections.iter().enumerate() {
            let head_id = format!("{}-head-{}", self.id, section.key);
            let panel_id = format!("{}-panel-{}", self.id, section.key);
            let (open, inert) = (marks.is_open(i), marks.is_inert(i));
            let press: Press = {
                let this = self.clone();
                Arc::new(move || this.press(i))
            };
            let mut one = div().id(format!("{}-section-{}", self.id, section.key)).class("fs-disclosure");
            let head = header(&head_id, &panel_id, &section.title, open, inert, &press);
            one = one.child(
                div()
                    .id(format!("{}-heading-{}", self.id, section.key))
                    .class("fs-disc-heading")
                    .attr("role", "heading")
                    .attr("aria-level", self.level.to_string())
                    .child(head),
            );
            if open {
                one = one
                    .class("fs-disclosure-open")
                    .child(panel(&panel_id, &head_id, true, (section.content)()));
            }
            stack = stack.child(one);
        }
        stack
    }
}

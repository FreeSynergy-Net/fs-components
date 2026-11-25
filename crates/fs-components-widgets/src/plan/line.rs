//! One task as a line of the day list: a round checkbox, the title, a small line under it, a chip on the right; a line
//! can open in place and show its details under itself.
//!
//! Pattern: Adapter — the data of one task in, an element out; the box and the title are two click targets with two
//! requests (`tick`, `open`), so ticking never opens and opening never ticks. This is the `task-row` block of the contract.
//! WAI-ARIA APG, checkbox: the head is a `checkbox` with `aria-checked`, one tab stop; Space ticks, Enter opens.
//! SONDERFALL: the window delivers Enter and Space as one `Key::Activate` (core keys.rs); it ticks until keys.rs tells them apart.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::chip::{Chip, chip_element};
use super::events::{Events, tell};
use super::model::Detail;
use crate::keys::{Handled, Key, KeyEvent, bind_keys};

/// One line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskLine {
    /// Names the task for the host.
    pub key: String,
    /// The title.
    pub title: String,
    /// A small line under the title (the project, the people).
    pub sub: Option<String>,
    /// The facts on the right (the priority, the due date), each a chip.
    pub facts: Vec<Chip>,
    /// A short code before the title (a task number); drawn small.
    pub code: Option<String>,
    /// The name of the "has a description" mark (drawn only when there is one).
    pub described: Option<String>,
    /// The box is ticked: the task is finished.
    pub checked: bool,
    /// Drawn pale: it cannot be started; `reason` says why in small text.
    pub muted: bool,
    /// Why it waits.
    pub reason: Option<String>,
    /// The details are shown under the line.
    pub open: bool,
    /// The details.
    pub details: Vec<Detail>,
}

impl TaskLine {
    /// A line with a title.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            ..Self::default()
        }
    }

    /// What a key does on the line: Space (and the window's Enter-or-Space) ticks, Enter opens.
    pub fn press(&self, event: KeyEvent, events: &Events) -> Handled {
        match event.key {
            Key::Space | Key::Activate => tell(events.tick.as_ref(), &self.key),
            Key::Enter => tell(events.open.as_ref(), &self.key),
            _ => return false,
        }
        true
    }

    /// The element; `id` prefixes its element ids.
    pub fn build(&self, id: &str, events: &Events) -> Div {
        let (tick, open) = (events.tick.clone(), events.open.clone());
        let (k1, k2) = (self.key.clone(), self.key.clone());
        let mut boxed = div()
            .class("fs-plan-check")
            .id(format!("{id}-{}-check", self.key));
        if self.checked {
            boxed = boxed
                .class("fs-plan-check-on")
                .child(text("\u{2713}").class("fs-plan-check-mark"));
        }
        let mut head = div()
            .class("fs-plan-line-head")
            .flex_row()
            .w_full()
            .id(format!("{id}-{}-head", self.key))
            .attr("role", "checkbox")
            .attr("aria-checked", if self.checked { "true" } else { "false" })
            .attr("aria-label", &self.title);
        let (line, ev) = (self.clone(), events.clone());
        head = bind_keys(head, Arc::new(move |e| line.press(e, &ev)));
        head = head.child(boxed.on_click(move |_| tell(tick.as_ref(), &k1)));
        let main = div()
            .class("fs-plan-line-main")
            .flex_row()
            .flex_1()
            .id(format!("{id}-{}", self.key))
            .child(self.body())
            .child(self.facts_element());
        head = head.child(main.on_click(move |_| tell(open.as_ref(), &k2)));
        let mut line = div().class("fs-plan-line").flex_col().w_full();
        if self.muted {
            line = line.class("fs-plan-muted");
        }
        line = line.child(head);
        if self.open {
            line = line.child(self.more());
        }
        line
    }

    fn body(&self) -> Div {
        let mut body = div().class("fs-plan-line-body").flex_col().flex_1();
        let mut title = div()
            .flex_row()
            .child(text(self.title.clone()).class("fs-plan-line-title"));
        if let Some(code) = &self.code {
            title = div()
                .flex_row()
                .child(text(code.clone()).class("fs-plan-code"))
                .child(title);
        }
        body = body.child(title);
        if let Some(reason) = &self.reason {
            body = body.child(text(reason.clone()).class("fs-plan-line-reason"));
        } else if let Some(sub) = &self.sub {
            body = body.child(text(sub.clone()).class("fs-plan-line-sub"));
        }
        body
    }

    fn facts_element(&self) -> Div {
        let mut facts = div().class("fs-plan-facts").flex_row();
        for chip in &self.facts {
            facts = facts.child(chip_element(chip));
        }
        if let Some(name) = &self.described {
            facts = facts.child(
                div()
                    .id(format!("{}-more", self.key))
                    .attr("role", "img")
                    .attr("aria-label", name)
                    .child(text("\u{2261}").class("fs-plan-line-sub")),
            );
        }
        facts
    }

    fn more(&self) -> Div {
        let mut more = div().class("fs-plan-line-more").flex_col();
        for d in &self.details {
            more = more.child(
                div()
                    .class("fs-plan-detail")
                    .flex_row()
                    .child(text(d.label.clone()).class("fs-plan-detail-label"))
                    .child(text(d.value.clone()).class("fs-plan-detail-value")),
            );
        }
        more
    }
}

//! The task row as a description and its element.
//!
//! Pattern: Builder + Composite - the description is filled by the caller and `build` puts the parts together; the handlers only report.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::Element;
use crate::keys::{Handled, Key, KeyEvent, bind_keys};

type Report = Arc<dyn Fn(&str) + Send + Sync>;

/// How loud the priority chip is drawn; the caller decides, the row does not know what a priority is.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// Quiet.
    #[default]
    Plain,
    /// Orange.
    Warning,
    /// Red.
    Danger,
}

impl Tone {
    const fn class(self) -> &'static str {
        match self {
            Self::Plain => "fs-task-row-prio-plain",
            Self::Warning => "fs-task-row-prio-warning",
            Self::Danger => "fs-task-row-prio-danger",
        }
    }
}

/// One row. Everything in it is what the caller says (no words of its own).
#[non_exhaustive]
#[derive(Clone)]
pub struct TaskRow {
    /// The key of the task; handed back by the handlers and part of the ids.
    pub key: String,
    /// The title.
    pub title: String,
    /// A smaller line under the title.
    pub sub: Option<String>,
    /// Ticked.
    pub done: bool,
    /// Priority: the word and its tone.
    pub prio: Option<(String, Tone)>,
    /// When it is due, as the caller wants it said.
    pub due: Option<String>,
    /// The name of the "has a description" mark (it is drawn only when there is one).
    pub described: Option<String>,
    /// Drawn as the selected row.
    pub selected: bool,
    /// Drawn quiet (blocked, not for now).
    pub muted: bool,
    id: String,
    on_toggle: Option<Report>,
    on_open: Option<Report>,
}

impl TaskRow {
    /// A row for the task `key` with this title.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            id: format!("task-row-{key}"),
            key,
            title: title.into(),
            sub: None,
            done: false,
            prio: None,
            due: None,
            described: None,
            selected: false,
            muted: false,
            on_toggle: None,
            on_open: None,
        }
    }

    /// The id of the row (default `task-row-<key>`); the parts are `<id>-check`, `<id>-prio`, `<id>-due`, `<id>-more`.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// A line under the title.
    #[must_use]
    pub fn subline(mut self, line: impl Into<String>) -> Self {
        self.sub = Some(line.into());
        self
    }

    /// Ticked.
    #[must_use]
    pub const fn done(mut self) -> Self {
        self.done = true;
        self
    }

    /// The priority chip.
    #[must_use]
    pub fn prio(mut self, word: impl Into<String>, tone: Tone) -> Self {
        self.prio = Some((word.into(), tone));
        self
    }

    /// The due date, as it is to be said.
    #[must_use]
    pub fn due(mut self, when: impl Into<String>) -> Self {
        self.due = Some(when.into());
        self
    }

    /// The mark that a description exists; `name` is what a reader of the screen hears.
    #[must_use]
    pub fn described(mut self, name: impl Into<String>) -> Self {
        self.described = Some(name.into());
        self
    }

    /// Drawn as selected.
    #[must_use]
    pub const fn selected(mut self) -> Self {
        self.selected = true;
        self
    }

    /// Drawn quiet.
    #[must_use]
    pub const fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    /// Call `handler` with the key when the person ticks or unticks (the row does not change itself; the data does).
    #[must_use]
    pub fn on_toggle(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    /// Call `handler` with the key when the person opens the task.
    #[must_use]
    pub fn on_open(mut self, handler: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_open = Some(Arc::new(handler));
        self
    }

    fn report(handler: Option<&Report>, key: &str) {
        if let Some(h) = handler {
            h(key);
        }
    }

    /// What a key does: Space and the window's Enter-or-Space tick, Enter opens.
    pub fn press(&self, event: KeyEvent) -> Handled {
        match event.key {
            Key::Space | Key::Activate => Self::report(self.on_toggle.as_ref(), &self.key),
            Key::Enter => Self::report(self.on_open.as_ref(), &self.key),
            _ => return false,
        }
        true
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let mut row = div()
            .id(&self.id)
            .class("fs-task-row")
            .attr("role", "checkbox")
            .attr("aria-checked", if self.done { "true" } else { "false" })
            .attr("aria-label", &self.title);
        for (on, class) in [
            (self.done, "fs-task-row-done"),
            (self.selected, "fs-task-row-selected"),
            (self.muted, "fs-task-row-muted"),
        ] {
            if on {
                row = row.class(class);
            }
        }
        let (keys, open, tick) = (self.clone(), self.clone(), self.clone());
        let mut row = bind_keys(row, Arc::new(move |e| keys.press(e)));
        row = row.child(
            div()
                .id(format!("{}-check", self.id))
                .on_click(move |_| Self::report(tick.on_toggle.as_ref(), &tick.key))
                .child(text(if self.done { "\u{2611}" } else { "\u{2610}" }).class("fs-check-box")),
        );
        row.child(
            div()
                .class("fs-task-row-body")
                .on_click(move |_| Self::report(open.on_open.as_ref(), &open.key))
                .child(self.main())
                .child(self.facts()),
        )
    }

    fn main(&self) -> Div {
        let mut main = div()
            .class("fs-task-row-main")
            .child(text(self.title.clone()).class("fs-task-row-title"));
        if let Some(sub) = &self.sub {
            main = main.child(text(sub.clone()).class("fs-task-row-sub"));
        }
        main
    }

    fn facts(&self) -> Div {
        let mut facts = div().class("fs-task-row-facts");
        if let Some((word, tone)) = &self.prio {
            facts = facts.child(
                div()
                    .id(format!("{}-prio", self.id))
                    .class("fs-task-row-prio")
                    .class(tone.class())
                    .child(text(word.clone())),
            );
        }
        if let Some(due) = &self.due {
            facts = facts.child(
                div()
                    .id(format!("{}-due", self.id))
                    .child(text(due.clone()).class("fs-task-row-due")),
            );
        }
        if let Some(name) = &self.described {
            facts = facts.child(
                div()
                    .id(format!("{}-more", self.id))
                    .attr("role", "img")
                    .attr("aria-label", name)
                    .child(text("\u{2261}").class("fs-task-row-more")),
            );
        }
        facts
    }
}

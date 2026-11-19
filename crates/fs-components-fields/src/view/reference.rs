//! Input of the type `reference`: a line to type into, the candidates under it, a click (or Enter on the only candidate) picks one.
//!
//! Pattern: Adapter over Blinc's `text_input` and our own `Listbox` (the candidates); Strategy for the source ([`ReferenceSource`], the
//! program's own search); Observer (`Live`) for the part under the line. The line is not inside the live area, so a redraw never takes
//! the keyboard from it. The value is the IRI as written; a picked candidate shows its label under the line. A reference with NO
//! source is still typed by its IRI, and the field says that nothing can be suggested (ADR 0097: no silent missing function).
//! Limits: the candidates are picked with the pointer or by Enter when exactly one is left; moving into the list with the arrow keys
//! is not done (the list is its own tab stop: Tab goes there).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::selector::InputRef;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input};
use fs_components_primitives::FieldData;
use fs_components_widgets::Element;
use fs_components_widgets::listbox::{Listbox, Opt};
use fs_components_widgets::live::Live;

use super::{Look, Say};
use crate::kinds::Kind;

/// One thing the source found: its identity and what the reader sees.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The identity (the stored IRI).
    pub iri: String,
    /// What the reader sees.
    pub label: String,
}

impl Candidate {
    /// A candidate.
    pub fn new(iri: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            iri: iri.into(),
            label: label.into(),
        }
    }
}

/// The program's search: what matches what was typed, at most `limit` candidates.
pub trait ReferenceSource: Send + Sync {
    /// The candidates for `typed`.
    fn find(&self, typed: &str, limit: usize) -> Vec<Candidate>;
}

/// How many candidates at most.
const LIMIT: usize = 6;

type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

struct Inner {
    look: Look,
    say: Arc<dyn Say>,
    source: Option<Arc<dyn ReferenceSource>>,
    data: SharedTextInputData,
    _keep: FieldData,
    line: InputRef,
    picked: Mutex<Option<Candidate>>,
    touched: AtomicBool,
    live: Live,
    name: String,
    on_change: Mutex<Option<OnChange>>,
}

/// The input of a `reference` field. Make it once per field and keep it; call [`ReferenceInput::build`] each time the form is drawn.
#[derive(Clone)]
pub struct ReferenceInput {
    inner: Arc<Inner>,
}

impl ReferenceInput {
    /// An empty input with the ids `<name>` (the line), `<name>-list`, `<name>-picked`, `<name>-problem`, `<name>-nosource`.
    pub fn new(
        look: Look,
        say: Arc<dyn Say>,
        name: &str,
        source: Option<Arc<dyn ReferenceSource>>,
    ) -> Self {
        let keep = FieldData::new();
        Self {
            inner: Arc::new(Inner {
                look,
                say,
                source,
                data: keep.text_input(),
                _keep: keep,
                line: InputRef::new(),
                picked: Mutex::new(None),
                touched: AtomicBool::new(false),
                live: Live::new(),
                name: name.to_owned(),
                on_change: Mutex::new(None),
            }),
        }
    }

    /// Call this with the IRI after every change.
    #[must_use]
    pub fn on_change(self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        *self
            .inner
            .on_change
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(f));
        self
    }

    /// Start with this IRI.
    #[must_use]
    pub fn value(self, iri: &str) -> Self {
        let mut d = self
            .inner
            .data
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        iri.trim().clone_into(&mut d.value);
        d.cursor = d.value.chars().count();
        drop(d);
        self
    }

    /// The IRI as typed or picked now (empty if none).
    pub fn text(&self) -> String {
        self.inner
            .data
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .value
            .trim()
            .to_owned()
    }

    /// What is wrong with the text now (an IRI that is not absolute or not an IRI).
    pub fn problem(&self) -> Option<crate::Problem> {
        let f = Kind::Reference.field();
        f.problem(&[self.text()])
    }

    /// The candidate that was picked, if the text is still its IRI.
    pub fn picked(&self) -> Option<Candidate> {
        let now = self.text();
        self.inner
            .picked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .filter(|c| c.iri == now)
    }

    /// The candidates for what is typed now (none without a source or for an empty line).
    pub fn candidates(&self) -> Vec<Candidate> {
        let typed = self.text();
        match (&self.inner.source, typed.is_empty()) {
            (Some(source), false) => source.find(&typed, LIMIT),
            _ => Vec::new(),
        }
    }

    /// Pick a candidate (a click, or Enter on the only one): its IRI becomes the text.
    pub fn pick(&self, candidate: &Candidate) {
        *self
            .inner
            .picked
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(candidate.clone());
        self.inner.line.set_value(candidate.iri.clone());
        self.inner.touched.store(true, Ordering::Relaxed);
        self.changed();
    }

    fn changed(&self) {
        self.inner.live.refresh_now();
        let f = self
            .inner
            .on_change
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(f) = f {
            f(&self.text());
        }
    }

    fn typed(&self) {
        self.inner.touched.store(true, Ordering::Relaxed);
        self.changed();
    }

    fn submitted(&self) {
        let found = self.candidates();
        if let [only] = found.as_slice() {
            self.pick(only);
        }
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let inner = &self.inner;
        let (look, s, pal) = (inner.look, inner.look.style, inner.look.palette());
        let (typing, entering, on) = (self.clone(), self.clone(), self.clone());
        let line = fs_components_primitives::with_field_font(text_input(&inner.data))
            .bind(&inner.line)
            .id(&inner.name)
            .h(s.heights.control)
            .text_size(s.text.body)
            .rounded(s.radii.control)
            .border(1.0, Look::color(pal.border))
            .idle_bg_color(Look::color(pal.field))
            .text_color(Look::color(pal.text))
            .placeholder_color(Look::color(pal.text_weak))
            .cursor_color(Look::color(pal.accent_text))
            .selection_color(Look::color(pal.selected))
            .placeholder(inner.say.say("hint.reference.iri"))
            .w_full()
            .on_change(move |_: &str| typing.typed())
            .on_submit(move |_shift: bool| entering.submitted());
        div()
            .class("fs-field-input")
            .flex_col()
            .gap_px(s.spacing.xs)
            .w_full()
            .child(look.text(
                inner.say.say("field.reference"),
                s.text.caption,
                pal.text_weak,
            ))
            .child(line)
            .child(inner.live.region("under", move || on.under()))
    }

    fn under(&self) -> Div {
        let inner = &self.inner;
        let (s, pal) = (inner.look.style, inner.look.palette());
        let mut column = div().flex_col().gap_px(s.spacing.xs).w_full();
        if inner.source.is_none() {
            // ADR 0097: the missing search is said, not left out
            column = column.child(
                inner
                    .look
                    .text(
                        inner.say.say("notice.reference.no-source"),
                        s.text.caption,
                        pal.text_weak,
                    )
                    .id(format!("{}-nosource", inner.name)),
            );
        }
        if let Some(c) = self.picked() {
            column = column.child(
                inner
                    .look
                    .text(c.label, s.text.body, pal.text)
                    .id(format!("{}-picked", inner.name)),
            );
        }
        let found = self.candidates();
        if !found.is_empty() && self.picked().is_none() {
            let opts: Vec<Opt> = found
                .iter()
                .map(|c| Opt::new(c.iri.clone(), c.label.clone()))
                .collect();
            let on = self.clone();
            let list = Listbox::new(opts)
                .id(&format!("{}-list", inner.name))
                .name(&inner.say.say("field.reference"))
                .on_change(move |picked| {
                    if let Some(c) = on
                        .candidates()
                        .into_iter()
                        .find(|c| picked.first() == Some(&c.iri))
                    {
                        on.pick(&c);
                    }
                });
            column = column.child(list.build());
        }
        let problem = self
            .problem()
            .filter(|_| inner.touched.load(Ordering::Relaxed));
        if let Some(p) = problem {
            column = column.child(
                inner
                    .look
                    .text(inner.say.say(p.key()), s.text.caption, pal.error)
                    .id(format!("{}-problem", inner.name)),
            );
        }
        column
    }
}

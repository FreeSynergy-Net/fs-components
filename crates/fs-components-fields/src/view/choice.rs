//! Input of the type `choice`: one entry of a fixed list, drawn as a segmented control for a few entries and as a list in a popover
//! for more (the same block `field-choice`, the count is a state of it, not a second building block).
//!
//! Pattern: Adapter over our own `Segmented` (up to [`SEGMENTS_UP_TO`] entries) and `Listbox` in a `Popover` (more); Observer
//! (`Live`) for the question under it. The labels are `<label>.<key>` through [`Say`] (ADR 0057); a list with no entry is NOT
//! drawn as a greyed control: the field says so in words (ADR 0097: no silent grey).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_widgets::Element;
use fs_components_widgets::listbox::{Listbox, Opt};
use fs_components_widgets::live::Live;
use fs_components_widgets::popover::{Labels as PopoverLabels, Popover};

use super::{Look, Say};
use crate::controls::Segmented;
use crate::kinds::check_choice;

/// Up to this many entries the choice is a segmented control; more are a list in a popover.
pub const SEGMENTS_UP_TO: usize = 5;

type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

struct Inner {
    look: Look,
    say: Arc<dyn Say>,
    label: String,
    keys: Vec<String>,
    value: Mutex<String>,
    touched: AtomicBool,
    live: Live,
    name: String,
    on_change: Option<OnChange>,
}

/// The input of a `choice` field. Make it once per field and keep it; call [`ChoiceInput::build`] each time the form is drawn.
#[derive(Clone)]
pub struct ChoiceInput {
    inner: Arc<Inner>,
}

impl ChoiceInput {
    /// A choice of `keys` (the entries, in order) for the field whose label key is `label`; entry `k` is called `<label>.<k>`.
    pub fn new(look: Look, say: Arc<dyn Say>, label: &str, keys: Vec<String>) -> Self {
        Self {
            inner: Arc::new(Inner {
                look,
                say,
                label: label.to_owned(),
                keys,
                value: Mutex::new(String::new()),
                touched: AtomicBool::new(false),
                live: Live::new(),
                name: "choice".to_owned(),
                on_change: None,
            }),
        }
    }

    fn edit(self, f: impl FnOnce(&mut Inner)) -> Self {
        let mut inner = Arc::try_unwrap(self.inner)
            .unwrap_or_else(|_| panic!("ChoiceInput is configured before it is built or cloned"));
        f(&mut inner);
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Start with this key (not checked here: a key that is not in the list is a problem to be shown, see [`ChoiceInput::problem`]).
    #[must_use]
    pub fn value(self, key: &str) -> Self {
        *self
            .inner
            .value
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = key.trim().to_owned();
        self
    }

    /// The prefix of the ids: `<name>`, `<name>-problem`, `<name>-empty`, and for the list `<name>-trigger`, `<name>-list`.
    #[must_use]
    pub fn name(self, name: &str) -> Self {
        self.edit(|i| i.name = name.to_owned())
    }

    /// Call this with the key after every change by the person.
    #[must_use]
    pub fn on_change(self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.edit(|i| i.on_change = Some(Arc::new(f)))
    }

    /// The key now (empty if none).
    pub fn text(&self) -> String {
        self.inner
            .value
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Whether the entries are drawn as segments (`true`) or as a list in a popover.
    pub fn is_segmented(&self) -> bool {
        self.inner.keys.len() <= SEGMENTS_UP_TO
    }

    /// What is wrong with the key now: it is not an entry of the list.
    pub fn problem(&self) -> Option<crate::Problem> {
        check_choice(&self.text(), &self.inner.keys)
    }

    fn label_of(&self, key: &str) -> String {
        self.inner.say.say(&format!("{}.{key}", self.inner.label))
    }

    fn chosen(&self, key: &str) {
        *self
            .inner
            .value
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = key.to_owned();
        self.inner.touched.store(true, Ordering::Relaxed);
        self.inner.live.refresh_now();
        if let Some(f) = &self.inner.on_change {
            f(key);
        }
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let inner = &self.inner;
        let (look, s, pal) = (inner.look, inner.look.style, inner.look.palette());
        let mut column = div()
            .class("fs-field-input")
            .flex_col()
            .gap_px(s.spacing.xs)
            .w_full()
            .child(look.text(inner.say.say("field.choice"), s.text.caption, pal.text_weak));
        column = if inner.keys.is_empty() {
            // ADR 0097: nothing to choose from is said, never drawn as a dead control
            column.child(
                look.text(inner.say.say("notice.choice.empty"), s.text.body, pal.error)
                    .id(format!("{}-empty", inner.name)),
            )
        } else if self.is_segmented() {
            column.child(self.segments())
        } else {
            column.child(self.list())
        };
        let on = self.clone();
        column.child(inner.live.region("hint", move || on.hint()))
    }

    fn segments(&self) -> Div {
        let mut seg = Segmented::new().id(&self.inner.name);
        for key in &self.inner.keys {
            seg = seg.segment(key, &self.label_of(key), false);
        }
        let current = self.text();
        if self.inner.keys.contains(&current) {
            seg.set(&current);
        } else {
            // nothing is chosen until the person chooses: the control must not claim the first entry
            seg = seg.unchosen();
        }
        let on = self.clone();
        div().child(seg.on_change(move |k| on.chosen(k)).build())
    }

    fn list(&self) -> Div {
        let (name, on) = (self.inner.name.clone(), self.clone());
        let pop = Popover::new(
            &PopoverLabels::default(),
            &self.inner.say.say("field.choice"),
        )
        .id(&name);
        let opts: Vec<Opt> = self
            .inner
            .keys
            .iter()
            .map(|k| Opt::new(k.clone(), self.label_of(k)))
            .collect();
        let trigger_on = self.clone();
        let list = Listbox::new(opts)
            .id(&format!("{name}-list"))
            .name(&self.inner.say.say("field.choice"));
        let closer = pop.clone();
        let list = list.on_change(move |picked| {
            if let Some(k) = picked.first() {
                on.chosen(k);
            }
            closer.close();
        });
        div().child(pop.build(
            move || {
                let text = trigger_on.text();
                let label = if text.is_empty() {
                    String::new()
                } else {
                    trigger_on.label_of(&text)
                };
                let look = trigger_on.inner.look;
                let (s, pal) = (look.style, look.palette());
                div()
                    .id(format!("{}-trigger", trigger_on.inner.name))
                    .attr("tabindex", "0")
                    .attr("role", "button")
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w(260.0)
                    .h(s.heights.control)
                    .px(s.spacing.s)
                    .rounded(s.radii.control)
                    .border(1.0, Look::color(pal.border))
                    .bg(Look::color(pal.field))
                    .child(look.text(label, s.text.body, pal.text))
                    .child(look.text("\u{25BE}", s.text.body, pal.text_weak))
            },
            move || div().child(list.build()),
        ))
    }

    fn hint(&self) -> Div {
        let inner = &self.inner;
        let (s, pal) = (inner.look.style, inner.look.palette());
        let mut column = div().flex_col().w_full();
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

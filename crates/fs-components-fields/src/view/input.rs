//! Input: the type split into its parts, each its own field, the question and the suggestions below.
//!
//! Pattern: Adapter over Blinc's `text_input` (not `blinc_cn`'s input: its height is one of 32, 40
//! or 48 pixels, the styles need their own) plus Observer (`Live`) for the part under the fields.
//! The fields are not inside the live area, so a redraw never takes the keyboard from the one being
//! typed in. The text is the field's own; this file adds the rule (`BasicField`), the joiner signs,
//! the spreading of a pasted whole (`settle`) and the questions.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize};

use blinc_layout::div::div;
use blinc_layout::selector::{InputRef, TextareaRef};
use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area};
use blinc_layout::widgets::text_input::{InputType, SharedTextInputData, text_input};
use fs_components_primitives::FieldData;
use fs_components_widgets::Element;
use fs_components_widgets::live::Live;

use super::suggestions::hint;
use super::{Look, Say};
use super::entry::{number_wrap, switch_row};
use crate::field::Entry;
use crate::kinds::Kind;
use crate::suggest::Suggest;

/// How many lines a many-line part shows before it scrolls.
const MANY_ROWS: usize = 4;

/// Called with the stored text (`BasicField::join`) after every change.
pub type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

pub(super) struct Inner {
    pub(super) kind: Kind,
    pub(super) look: Look,
    pub(super) say: Arc<dyn Say>,
    pub(super) data: Vec<SharedTextInputData>,
    /// Gives the signals of `data` back when the input is gone.
    _keep: FieldData,
    pub(super) refs: Vec<InputRef>,
    /// The many-line parts (a type with `BasicField::multiline`): these stand in for `data` and `refs`.
    pub(super) areas: Vec<SharedTextAreaState>,
    pub(super) area_refs: Vec<TextareaRef>,
    /// The answer of a yes/no type (`true`, `false`, empty: not answered): it has no text field.
    pub(super) flag: Mutex<String>,
    pub(super) live: Live,
    pub(super) suggest: Option<Arc<dyn Suggest>>,
    pub(super) touched: AtomicBool,
    pub(super) last: AtomicUsize,
    pub(super) on_change: Option<OnChange>,
    pub(super) name: Option<String>,
}

/// The input component of a basic type. Make it once per field on a form and keep it: it holds the
/// text; call [`FieldInput::build`] for the element each time the form is drawn.
#[derive(Clone)]
pub struct FieldInput {
    inner: Arc<Inner>,
}

impl FieldInput {
    /// An empty input of `kind`.
    pub fn new(kind: Kind, look: Look, say: Arc<dyn Say>) -> Self {
        let n = kind.field().segments().len();
        let many = kind.field().multiline();
        let texty = matches!(kind.field().entry(), Entry::Text | Entry::Number { .. });
        let keep = FieldData::new();
        Self {
            inner: Arc::new(Inner {
                kind,
                look,
                say,
                data: (0..n)
                    .filter(|_| texty)
                    .map(|_| keep.text_input())
                    .collect(),
                areas: (0..n).filter(|_| many).map(|_| keep.text_area()).collect(),
                _keep: keep,
                refs: (0..n).filter(|_| texty).map(|_| InputRef::new()).collect(),
                area_refs: (0..n)
                    .filter(|_| many)
                    .map(|_| TextareaRef::new())
                    .collect(),
                flag: Mutex::new(String::new()),
                live: Live::new(),
                suggest: None,
                touched: AtomicBool::new(false),
                last: AtomicUsize::new(0),
                on_change: None,
                name: None,
            }),
        }
    }

    fn edit(self, f: impl FnOnce(&mut Inner)) -> Self {
        let mut inner = Arc::try_unwrap(self.inner)
            .unwrap_or_else(|_| panic!("FieldInput is configured before it is built or cloned"));
        f(&mut inner);
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Start with this stored text (spread over the parts by `split`).
    #[must_use]
    pub fn value(self, text: &str) -> Self {
        let parts = self.inner.kind.field().split(text);
        if let (Entry::Switch, Some(part)) = (self.inner.kind.field().entry(), parts.first()) {
            self.inner.set_part(0, part);
        }
        for (area, part) in self.inner.areas.iter().zip(&parts) {
            area.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_value(part);
        }
        for (data, part) in self.inner.data.iter().zip(parts) {
            let mut d = data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            d.cursor = part.chars().count();
            d.value = part;
        }
        self
    }

    /// A name for the field: the parts get the ids `<name>-<part>` (`mail-local`), for tests and
    /// for automation.
    #[must_use]
    pub fn name(self, name: &str) -> Self {
        self.edit(|i| i.name = Some(name.to_string()))
    }

    /// Ask this for completions (the command "suggest" of the type description).
    #[must_use]
    pub fn suggest(self, source: Arc<dyn Suggest>) -> Self {
        self.edit(|i| i.suggest = Some(source))
    }

    /// Call this with the stored text after every change.
    #[must_use]
    pub fn on_change(self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.edit(|i| i.on_change = Some(Arc::new(f)))
    }

    /// Move the number by `delta` steps, as the arrow keys do (a number type only; nothing happens for another).
    pub fn step(&self, delta: i32) {
        if let Entry::Number { signed } = self.inner.kind.field().entry() {
            self.inner.step(0, signed, delta);
        }
    }

    /// The parts as typed now.
    pub fn parts(&self) -> Vec<String> {
        self.inner.parts()
    }

    /// The stored text now (empty if nothing is given).
    pub fn text(&self) -> String {
        self.inner.kind.field().join(&self.parts())
    }

    /// What is wrong with the text now, whether or not the person has changed anything yet.
    pub fn problem(&self) -> Option<crate::Problem> {
        self.inner.kind.field().problem(&self.parts())
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let inner = &self.inner;
        let (look, kind) = (inner.look, inner.kind);
        let s = look.style;
        let pal = look.palette();
        let field = kind.field();
        let mut row = div()
            .flex_row()
            .items_center()
            .gap_px(s.spacing.xs)
            .w_full();
        for (i, segment) in field.segments().iter().enumerate() {
            let hint_text = inner
                .say
                .say(&format!("hint.{}.{}", field.key(), segment.key));
            let on = Arc::clone(inner);
            if field.multiline() {
                let mut area = text_area(&inner.areas[i])
                    .bind(&inner.area_refs[i])
                    .class(&super::area::area_class(&look))
                    .rows(MANY_ROWS)
                    .font_size(s.text.body)
                    .bg(Look::color(pal.field))
                    .rounded(s.radii.control)
                    .border(1.0, Look::color(pal.border))
                    .placeholder(hint_text)
                    .w_full()
                    .on_change(move |_| on.changed(i));
                if let Some(name) = &inner.name {
                    area = area.id(&format!("{name}-{}", segment.key));
                }
                row = row.child(div().flex_grow_value(1.0).min_w(0.0).child(area));
                continue;
            }
            if let Entry::Switch = field.entry() {
                row = row.child(switch_row(inner, i));
                continue;
            }
            let mut input = fs_components_primitives::with_field_font(text_input(&inner.data[i]))
                .bind(&inner.refs[i]);
            if let Entry::Number { signed } = field.entry() {
                let step = Arc::clone(inner);
                input = input
                    .input_type(InputType::Integer)
                    .on_step(move |delta| step.step(i, signed, delta));
            }
            if let Some(name) = &inner.name {
                input = input.id(&format!("{name}-{}", segment.key));
            }
            let input = input
                .h(s.heights.control)
                .text_size(s.text.body)
                .rounded(s.radii.control)
                .border(1.0, Look::color(pal.border))
                .idle_bg_color(Look::color(pal.field))
                .text_color(Look::color(pal.text))
                .placeholder_color(Look::color(pal.text_weak))
                .cursor_color(Look::color(pal.accent_text))
                .selection_color(Look::color(pal.selected))
                .placeholder(hint_text)
                .w_full()
                .on_change(move |_| on.changed(i));
            let cell = div()
                .flex_grow_value(f32::from(segment.weight))
                .min_w(0.0)
                .child(input);
            row = row.child(match field.entry() {
                Entry::Number { signed } => number_wrap(cell, inner.name.as_deref(), signed, &inner.parts()[i]),
                _ => cell,
            });
            if let Some(sign) = field.joiner(i) {
                row = row.child(look.text(sign, s.text.body, pal.text_weak));
            }
        }
        let on = Arc::clone(inner);
        div()
            .class("fs-field-input")
            .flex_col()
            .gap_px(s.spacing.xs)
            .w_full()
            .child(look.text(
                inner.say.say(&format!("field.{}", field.key())),
                s.text.caption,
                pal.text_weak,
            ))
            .child(row)
            .child(inner.live.region("hint", move || hint(&on)))
    }
}

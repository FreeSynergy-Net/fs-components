//! Input of the type `keywords`: the words as chips, a line to type the next one into.
//!
//! Pattern: Adapter over our own `ChipList` and Blinc's `text_input`; Observer (`Live`) for the question. A comma or Enter turns the
//! typed word into a chip, Backspace on the empty line removes the last chip, a chip can also be removed with Delete (the row of chips)
//! or its button. The word is made as the stored one is (the contract crate: trimmed, NFC, lower case), so the lower case is
//! visible at once. The line is not inside a live area, so a redraw never takes the keyboard from it. Limits: 64 characters a word,
//! 256 words (contract crate); the stored text is the canonical one, sorted ("a, b, c").

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::selector::InputRef;
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input};
use fs_components_primitives::FieldData;
use fs_components_widgets::Element;
use fs_components_widgets::chip::{ChipList, Labels as ChipLabels};
use fs_components_widgets::live::Live;
use fs_types::{BasicValue, Keywords, TypeProblem};

use super::{Look, Say};
use crate::Problem;

type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

/// Most words in one set (the contract crate says 256 too).
pub(super) const MOST: usize = 256;

pub(super) struct Inner {
    pub(super) look: Look,
    pub(super) say: Arc<dyn Say>,
    pub(super) words: Mutex<Vec<String>>,
    pub(super) problem: Mutex<Option<Problem>>,
    pub(super) touched: AtomicBool,
    pub(super) chips: ChipList,
    pub(super) data: SharedTextInputData,
    pub(super) _keep: FieldData,
    pub(super) line: InputRef,
    pub(super) live: Live,
    pub(super) name: String,
    pub(super) on_change: Mutex<Option<OnChange>>,
}

/// The input of a `keywords` field. Make it once per field and keep it; call [`KeywordsInput::build`] each time the form is drawn.
#[derive(Clone)]
pub struct KeywordsInput {
    pub(super) inner: Arc<Inner>,
}

/// A word as the contract crate stores it, or the problem.
pub(super) fn made(raw: &str) -> Result<Option<String>, Problem> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(None);
    }
    match Keywords::parse(t) {
        Ok(k) => Ok(k.iter().next().map(str::to_owned)),
        Err(TypeProblem::Empty) => Ok(None),
        Err(_) => Err(Problem::BadKeyword),
    }
}

impl KeywordsInput {
    /// An empty input whose parts have the ids `<name>` (the line), `<name>-chips`, `<name>-problem`; `new` uses `keywords`.
    pub fn named(look: Look, say: Arc<dyn Say>, name: &str) -> Self {
        let keep = FieldData::new();
        let inner = Arc::new_cyclic(|weak: &std::sync::Weak<Inner>| {
            let gone = weak.clone();
            // a chip's button removed the chip from the row already: the words follow
            let chips = ChipList::new(
                &say.say("field.keywords"),
                &ChipLabels::default(),
                Vec::new(),
            )
            .id(&format!("{name}-chips"))
            .on_remove(move |word| {
                if let Some(inner) = gone.upgrade() {
                    Self { inner }.chip_removed(word);
                }
            });
            Inner {
                look,
                say,
                words: Mutex::new(Vec::new()),
                problem: Mutex::new(None),
                touched: AtomicBool::new(false),
                chips,
                data: keep.text_input(),
                _keep: keep,
                line: InputRef::new(),
                live: Live::new(),
                name: name.to_owned(),
                on_change: Mutex::new(None),
            }
        });
        Self { inner }
    }

    /// An empty input.
    pub fn new(look: Look, say: Arc<dyn Say>) -> Self {
        Self::named(look, say, "keywords")
    }

    /// Call this with the stored text after every change.
    #[must_use]
    pub fn on_change(self, f: impl Fn(&str) + Send + Sync + 'static) -> Self {
        *self
            .inner
            .on_change
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(f));
        self
    }

    fn chip_removed(&self, word: &str) {
        let mut words = self
            .inner
            .words
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(i) = words.iter().position(|w| w == word) {
            words.remove(i);
            drop(words);
            self.inner.touched.store(true, Ordering::Relaxed);
            self.changed();
        }
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let inner = &self.inner;
        let (look, s, pal) = (inner.look, inner.look.style, inner.look.palette());
        let (typing, entering, emptying) = (self.clone(), self.clone(), self.clone());
        let mut line = fs_components_primitives::with_field_font(text_input(&inner.data))
            .bind(&inner.line)
            .id(&inner.name);
        line = line
            .h(s.heights.control)
            .text_size(s.text.body)
            .rounded(s.radii.control)
            .border(1.0, Look::color(pal.border))
            .idle_bg_color(Look::color(pal.field))
            .text_color(Look::color(pal.text))
            .placeholder_color(Look::color(pal.text_weak))
            .cursor_color(Look::color(pal.accent_text))
            .selection_color(Look::color(pal.selected))
            .placeholder(inner.say.say("hint.keywords.words"))
            .w_full()
            .on_change(move |now: &str| typing.typed(now))
            .on_submit(move |_shift: bool| entering.submitted())
            .on_backspace_empty(move || emptying.backspace_empty());
        let on = self.clone();
        div()
            .class("fs-field-input")
            .flex_col()
            .gap_px(s.spacing.xs)
            .w_full()
            .child(look.text(
                inner.say.say("field.keywords"),
                s.text.caption,
                pal.text_weak,
            ))
            .child(inner.chips.build())
            .child(line)
            .child(inner.live.region("hint", move || on.hint()))
    }

    fn hint(&self) -> Div {
        let inner = &self.inner;
        let (s, pal) = (inner.look.style, inner.look.palette());
        let mut column = div().flex_col().w_full();
        if let Some(p) = self
            .problem()
            .filter(|_| inner.touched.load(Ordering::Relaxed))
        {
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

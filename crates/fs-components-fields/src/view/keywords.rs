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
use fs_components_widgets::chip::{Chip, ChipList, Labels as ChipLabels};
use fs_components_widgets::live::Live;
use fs_types::{BasicValue, Keywords, TypeProblem};

use super::{Look, Say};
use crate::Problem;
use crate::kinds::canonical_keywords;

type OnChange = Arc<dyn Fn(&str) + Send + Sync>;

/// Most words in one set (the contract crate says 256 too).
const MOST: usize = 256;

struct Inner {
    look: Look,
    say: Arc<dyn Say>,
    words: Mutex<Vec<String>>,
    problem: Mutex<Option<Problem>>,
    touched: AtomicBool,
    chips: ChipList,
    data: SharedTextInputData,
    _keep: FieldData,
    line: InputRef,
    live: Live,
    name: String,
    on_change: Mutex<Option<OnChange>>,
}

/// The input of a `keywords` field. Make it once per field and keep it; call [`KeywordsInput::build`] each time the form is drawn.
#[derive(Clone)]
pub struct KeywordsInput {
    inner: Arc<Inner>,
}

/// A word as the contract crate stores it, or the problem.
fn made(raw: &str) -> Result<Option<String>, Problem> {
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

    /// Start with this stored (or typed) list.
    #[must_use]
    pub fn value(self, text: &str) -> Self {
        for word in text.split(',') {
            let _ = self.add_word(word, false);
        }
        self
    }

    /// The words as chips now, in the order they were made.
    pub fn words(&self) -> Vec<String> {
        self.inner
            .words
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The stored text now (canonical: sorted, "a, b, c"; empty if none).
    pub fn text(&self) -> String {
        let words = self.words().join(", ");
        canonical_keywords(&words).unwrap_or_default()
    }

    /// The last thing that was refused (a word that is too long, too many words), `None` if all is well.
    pub fn problem(&self) -> Option<Problem> {
        *self
            .inner
            .problem
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Turn `raw` into a chip (as a comma or Enter does); false when nothing was added (blank, a word already there, refused).
    pub fn add_word(&self, raw: &str, touched: bool) -> bool {
        if touched {
            self.inner.touched.store(true, Ordering::Relaxed);
        }
        let word = match made(raw) {
            Ok(Some(w)) => w,
            Ok(None) => return false,
            Err(p) => return self.refuse(p),
        };
        {
            let mut words = self
                .inner
                .words
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if words.contains(&word) {
                return false;
            }
            if words.len() >= MOST {
                drop(words);
                return self.refuse(Problem::TooMany);
            }
            words.push(word.clone());
        }
        *self
            .inner
            .problem
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = None;
        self.inner.chips.add(Chip::new(word.clone(), word));
        self.changed();
        true
    }

    fn refuse(&self, p: Problem) -> bool {
        *self
            .inner
            .problem
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(p);
        self.inner.live.refresh_now();
        false
    }

    /// Remove the word (the chip's button, Delete on the row, Backspace on the empty line); false when it is not there.
    pub fn remove_word(&self, word: &str) -> bool {
        let found = {
            let mut words = self
                .inner
                .words
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            words
                .iter()
                .position(|w| w == word)
                .map(|i| words.remove(i))
                .is_some()
        };
        if found {
            self.inner.chips.remove(word);
            self.inner.touched.store(true, Ordering::Relaxed);
            self.changed();
        }
        found
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

    /// The line changed: every word before a comma becomes a chip, what follows the last comma stays typed.
    fn typed(&self, now: &str) {
        let Some((done, rest)) = now.rsplit_once(',') else {
            return;
        };
        for word in done.split(',') {
            let _ = self.add_word(word, true);
        }
        self.inner.line.set_value(rest.trim_start().to_owned());
    }

    fn submitted(&self) {
        let now = self
            .inner
            .data
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .value
            .clone();
        if self.add_word(&now, true) || now.trim().is_empty() {
            self.inner.line.set_value(String::new());
        }
    }

    fn backspace_empty(&self) {
        if let Some(last) = self.words().last().cloned() {
            self.remove_word(&last);
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

//! The one entry of a component that draws a form: [`FormView`] says which widget it is and makes a [`Mounted`] from a
//! [`FormInput`]; the host keeps the `Mounted` over a rebuild, so the state (selection, search, scroll) stays. Reading comes in as
//! [`FormInput`], writing goes out through [`FormWrite`], texts through `say` (MF2 keys), the look only as a style and the scheme.
//!
//! Pattern: Abstract Factory (`FormView`) producing a handle (`Mounted`); the host holds no list of components. `DrawCtx` is a
//! Parameter Object with Dependency Injection (the date names come in as a field, there is no global access).

use std::sync::Arc;

use blinc_layout::div::Div;
use fs_components_mf2::NeutralDateNames;
use fs_components_styles::StyleId;
use fs_components_words::{DateNames, Locale};

use crate::form_read::FormInput;
use crate::form_write::FormWrite;
use crate::keys::Key;

pub use crate::place_info::PlaceInfo;

/// What the host gives a component besides the data.
#[derive(Clone)]
pub struct DrawCtx {
    /// The place of the component: the view (`month|week|day|agenda`) and the variant (`compact|color-N`) the place asks for. The
    /// host reads them from the place declaration (`view=`, `variant=`); `None` = the contract default of the component, no special case.
    pub place: PlaceInfo,
    /// The style and whether it is dark (asked by the host at every drawing; nothing else about the look is passed).
    pub look: (StyleId, bool),
    /// A translation key in, the sentence out; a missing key comes back as itself.
    pub say: Arc<dyn Fn(&str) -> String + Send + Sync>,
    /// The language of the reader (for the words a component brings itself, like the buttons of a dialog).
    pub locale: Locale,
    /// The write side: every control is drawn locked when `can` says no. [`crate::form_write::NoWrite`] locks all.
    pub write: Arc<dyn FormWrite>,
    /// The form read again: a component calls it after an accepted write to show what the module now says.
    pub reread: Arc<dyn Fn() -> FormInput + Send + Sync>,
    /// The names of weekdays and months, the titles of months and days, and the first day of the week, in the language of
    /// `locale` (CLDR over ICU4X, from the same language package as the texts). The host builds it ONCE per language
    /// (`fs_components_mf2::date_names`); the setting `fs.shell.week-start` comes in as `-u-fw-<day>` of the locale
    /// (`fs_components_words::with_week_start`). A component never holds a table of such words. [`DrawCtx::new`] gives the
    /// neutral names (ISO forms, week on Monday); [`DrawCtx::with_dates`] replaces them.
    pub dates: Arc<dyn DateNames>,
}

impl DrawCtx {
    /// A context with the neutral date names (no language package): what a test or a host without date data gets.
    #[must_use]
    pub fn new(
        place: PlaceInfo,
        look: (StyleId, bool),
        say: Arc<dyn Fn(&str) -> String + Send + Sync>,
        locale: Locale,
        write: Arc<dyn FormWrite>,
        reread: Arc<dyn Fn() -> FormInput + Send + Sync>,
    ) -> Self {
        Self {
            place,
            look,
            say,
            locale,
            write,
            reread,
            dates: Arc::new(NeutralDateNames),
        }
    }

    /// The same context with the date names of the language (Dependency Injection: the host passes them in).
    #[must_use]
    pub fn with_dates(mut self, dates: Arc<dyn DateNames>) -> Self {
        self.dates = dates;
        self
    }
}

/// A component that draws a form.
pub trait FormView: Send + Sync {
    /// The widget this view is: the key `widget` of its block in the contract (a test holds both lists equal).
    fn widget(&self) -> &'static str;

    /// Make the component over `input`.
    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted>;
}

/// A mounted component: the handle that keeps the state over a rebuild of the page.
pub trait Mounted: Send + Sync {
    /// The element for this size.
    fn build(&self, width: f32, height: f32) -> Div;

    /// New rows after the module changed. A state the reader made (selection, search) stays where it still makes sense.
    fn refresh(&self, input: FormInput);

    /// A key the component may act on; `false` = not handled.
    fn key(&self, _key: Key) -> bool {
        false
    }
}

/// The sentence the host gave for `key` (`said`), or `name` when it answered with the key itself (it has no sentence):
/// the one rule "a sentence of the host, else the raw name" for every component that draws names from the data.
pub fn said_or(said: String, key: &str, name: &str) -> String {
    if said == key { name.to_owned() } else { said }
}

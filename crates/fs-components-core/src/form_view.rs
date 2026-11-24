//! The one entry of a component that draws a form: [`FormView`] says which widget it is and makes a [`Mounted`] from a
//! [`FormInput`]; the host keeps the `Mounted` over a rebuild, so the state (selection, search, scroll) stays. Reading comes in as
//! [`FormInput`], writing goes out through [`FormWrite`], texts through `say` (MF2 keys), the look only as a style and the scheme.
//!
//! Pattern: Abstract Factory (`FormView`) producing a handle (`Mounted`); the host holds no list of components.

use std::sync::Arc;

use blinc_layout::div::Div;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

use crate::form_read::FormInput;
use crate::form_write::FormWrite;
use crate::keys::Key;

/// What the host gives a component besides the data.
#[derive(Clone)]
pub struct DrawCtx {
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

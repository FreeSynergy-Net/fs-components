//! The style of a running window follows the settings `fs.look.style` and `fs.look.scheme` (ADR 0074).
//!
//! Pattern: Observer — [`StyleFollows`] is an `fs_settings::OnChange` the shell hands to `Settings::observe`; a change of one of the
//! two keys calls [`crate::set_style`]. The decision (`apply`) is a plain function of key and value, so a test needs no settings
//! store. A value that is no style of ours (a style that is not installed) changes nothing: the window keeps the style it has.

use std::sync::Mutex;
use std::sync::PoisonError;

use fs_components_styles::{StyleId, current_style};
use fs_settings::{Change, OnChange, Value};

/// The full keys of the two settings (owner `fs`, block `look`).
const STYLE_KEY: &str = "fs.look.style";
const SCHEME_KEY: &str = "fs.look.scheme";

/// Applies the style settings to the running window.
#[derive(Debug)]
pub struct StyleFollows {
    css: String,
    dark: Mutex<bool>,
}

impl StyleFollows {
    /// Follows the settings; `css` is the sheet of the host, as for [`crate::style_bundle`]; `dark` the appearance at the start.
    pub fn new(css: &str, dark: bool) -> Self {
        Self {
            css: css.to_owned(),
            dark: Mutex::new(dark),
        }
    }

    /// A setting `key` has `value` now: change the style or the appearance when it is one of ours. True when the window's style was
    /// set (the host builds its view again then).
    pub fn apply(&self, key: &str, value: &Value) -> bool {
        let Value::Choice(choice) = value else {
            return false;
        };
        let mut dark = self.dark.lock().unwrap_or_else(PoisonError::into_inner);
        let id = match key {
            STYLE_KEY => {
                let Some(id) = StyleId::from_name(choice) else {
                    return false;
                };
                id
            }
            SCHEME_KEY => {
                // `dark` and `light` only: another word (a typo) is ignored like a style that is not installed
                *dark = match choice.as_str() {
                    "dark" => true,
                    "light" => false,
                    _ => return false,
                };
                current_style()
            }
            _ => return false,
        };
        crate::set_style(id, *dark, &self.css);
        true
    }
}

impl OnChange for StyleFollows {
    fn changed(&self, change: &Change) {
        self.apply(change.key.as_str(), &change.after.value);
    }
}

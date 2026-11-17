//! The settings every program has through its shell: language, style, light/dark (the first use: switch them LIVE in
//! every program), and the frontend choice (a default the person changes, not a fixed state; b7 to fs-starter O11).
//! More (contrast, font, motion, sound; ADR 0074, ADR 0097 decision 2 "Darstellung") follow as further declarations of
//! their own blocks, not here (draft §9).
//!
//! Pattern: Strategy ([`Declares`]) with the list of styles passed in: the shell knows which styles are installed
//! (fs-components-styles `StyleCatalog`, ADR 0074), this crate does not.

use crate::coded_stub::Coded;
use fs_types::{BasicValue, Language};

use crate::codes::{SettingCode, keys};
use crate::declare::{Declaration, Declares};
use crate::layer::Layer;
use crate::names::{GroupId, Owner, OwnerKind, SettingKey};
use crate::resolve::Effective;
use crate::value::{Kind, Value};

/// `fs.shell.language`: the language of the program's words (BCP 47). Default `en` (end of the ADR 0057 chain); the
/// system layer brings the desktop language.
pub const LANGUAGE: (&str, &str) = ("shell", "language");
/// `fs.look.style`: the style (theme) id.
pub const STYLE: (&str, &str) = ("look", "style");
/// `fs.look.scheme`: `light` or `dark`. No profile value = follow the system (portal `color-scheme`).
pub const SCHEME: (&str, &str) = ("look", "scheme");

/// `fs.shell.frontend`: the frontend the program opens. The block default is the program's `fs:default`, else its
/// standard frontend (ADR 0094 4(b)). The starter takes the person's choice only (layer person/program, or a lock) and
/// keeps its own order: word > choice of the person > `fs:default` > standard frontend (fs-starter `pick`).
pub const FRONTEND: (&str, &str) = ("shell", "frontend");

/// `fs.shell.refresh-ms`: how often the shell asks for changes while a window is open (b7 decision O7: adjustable;
/// see [`crate::Pace`]). Milliseconds, 250 to 60000, default 1000.
pub const REFRESH: (&str, &str) = ("shell", "refresh-ms");

/// The shell's declarations.
#[derive(Clone, Debug)]
pub struct ShellSettings {
    styles: Vec<String>,
    default_style: String,
    frontends: Vec<String>,
    default_frontend: String,
}

impl ShellSettings {
    /// With the installed `styles` (ids) and the frontend's own default style (ADR 0074 line 98: a frontend names its
    /// standard look), the installed `frontends` (ids) and `default_frontend` (`fs:default`, else the standard one).
    #[must_use]
    pub fn new(styles: Vec<String>, default_style: String, frontends: Vec<String>, default_frontend: String) -> Self {
        Self { styles, default_style, frontends, default_frontend }
    }

    /// The key of one of the constants above.
    ///
    /// # Errors
    /// `key-invalid` (never for the constants; a K-test proves it).
    pub fn key(which: (&str, &str)) -> Result<SettingKey, Coded<SettingCode>> {
        SettingKey::new(&Owner::new(OwnerKind::Framework, which.0)?, which.1)
    }
}

impl Declares for ShellSettings {
    fn declarations(&self) -> Result<Vec<Declaration>, Coded<SettingCode>> {
        let english = Language::parse("en")
            .map_err(|_| Coded::new(SettingCode::DefaultInvalid).with_text(keys::KEY, "fs.shell.language"))?;
        Ok(vec![
            Declaration::new(Self::key(LANGUAGE)?, Kind::Language, Value::Language(english), GroupId::new("language")?),
            Declaration::new(
                Self::key(STYLE)?,
                Kind::Choice { keys: self.styles.clone() },
                Value::Choice(self.default_style.clone()),
                GroupId::new("look")?,
            ),
            Declaration::new(
                Self::key(SCHEME)?,
                Kind::Choice { keys: vec!["light".to_owned(), "dark".to_owned()] },
                Value::Choice("light".to_owned()),
                GroupId::new("look")?,
            ),
            Declaration::new(
                Self::key(FRONTEND)?,
                Kind::Choice { keys: self.frontends.clone() },
                Value::Choice(self.default_frontend.clone()),
                GroupId::new("look")?,
            ),
            Declaration::new(
                Self::key(REFRESH)?,
                Kind::Whole { min: 250, max: 60_000 },
                Value::Whole(1000),
                GroupId::new("language")?,
            )
            .per_device(),
        ])
    }
}

/// The person's frontend choice for fs-starter's `Preference`: `Some(id)` only when the person chose (program or general
/// profile) or a lock holds it; `None` lets the starter go on with `fs:default` and the standard frontend.
#[must_use]
pub fn person_choice(e: &Effective) -> Option<String> {
    let chosen = e.from >= Layer::Person || e.lock.is_some();
    match (&e.value, chosen) {
        (Value::Choice(id), true) => Some(id.clone()),
        _ => None,
    }
}

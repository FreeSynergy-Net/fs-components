//! Names: who owns a setting, the setting's key, its group, and a program name (whose profile).
//!
//! Pattern: Value objects (checked on creation, never changed). One form for all of them: lower-case ASCII words of
//! `a-z`, `0-9`, `-`, joined by `.` where allowed; never `..`, `/`, upper case or an empty part.

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};

/// The kinds of building blocks that declare settings (user's word: general/framework, module, theme, app). `Frontend`
/// is a PROPOSAL of the draft (ADR 0094: a frontend package may bring its own settings); `Motion` and `SoundTheme`
/// (ADR 0074) were added by the b7 decision on draft O13.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OwnerKind {
    /// A framework crate of every program (shell, look, session ...). Prefix `fs`.
    Framework,
    /// A module (ADR 0097 decision 3). Prefix `module`.
    Module,
    /// A theme package (ADR 0074). Prefix `theme`.
    Theme,
    /// A program. Prefix `program`.
    Program,
    /// A frontend package (proposal). Prefix `frontend`.
    Frontend,
    /// The motion settings (reduced motion, animation speed; ADR 0074; b7 decision O13). Prefix `motion`.
    Motion,
    /// A sound theme (ADR 0074; b7 decision O13). Prefix `sound`.
    SoundTheme,
}

impl OwnerKind {
    /// Every kind, in the order of the enum.
    pub const ALL: &'static [Self] = &[
        Self::Framework,
        Self::Module,
        Self::Theme,
        Self::Program,
        Self::Frontend,
        Self::Motion,
        Self::SoundTheme,
    ];

    /// The first part of every key of this kind.
    #[must_use]
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Framework => "fs",
            Self::Module => "module",
            Self::Theme => "theme",
            Self::Program => "program",
            Self::Frontend => "frontend",
            Self::Motion => "motion",
            Self::SoundTheme => "sound",
        }
    }
}

/// One word: 1 to 32 bytes of `a-z`, `0-9`, `-`; starts and ends with a letter or digit; no `--`.
fn word_ok(w: &str) -> bool {
    let b = w.as_bytes();
    !b.is_empty()
        && b.len() <= 32
        && b[0].is_ascii_alphanumeric()
        && b[b.len() - 1].is_ascii_alphanumeric()
        && !w.contains("--")
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn invalid(key: &str) -> Coded<SettingCode> {
    Coded::new(SettingCode::KeyInvalid).with_text(keys::KEY, key)
}

/// A building block that declares settings: its kind and its name (`shell`, `vcard`, `calm`, `tasks`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Owner {
    kind: OwnerKind,
    name: String,
}

impl Owner {
    /// The owner `kind`/`name`.
    ///
    /// # Errors
    /// `key-invalid` when `name` is no word.
    pub fn new(kind: OwnerKind, name: &str) -> Result<Self, Coded<SettingCode>> {
        if word_ok(name) {
            Ok(Self {
                kind,
                name: name.to_owned(),
            })
        } else {
            Err(invalid(name))
        }
    }

    /// The kind.
    #[must_use]
    pub fn kind(&self) -> OwnerKind {
        self.kind
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// The full key of a setting: `<prefix>.<owner>.<local>` (`fs.shell.language`, `program.tasks.week-start`). `local` may
/// itself hold dots (`look.contrast`), at most 4 words; the whole key is at most 96 bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SettingKey(String);

impl SettingKey {
    /// The key `local` of `owner`.
    ///
    /// # Errors
    /// `key-invalid` (parameter = the whole key) when a word of `local` is no word or the key is too long.
    pub fn new(owner: &Owner, local: &str) -> Result<Self, Coded<SettingCode>> {
        let full = format!("{}.{}.{local}", owner.kind.prefix(), owner.name);
        let parts: Vec<&str> = local.split('.').collect();
        if parts.len() > 4 || full.len() > 96 || !parts.iter().all(|p| word_ok(p)) {
            return Err(invalid(&full));
        }
        Ok(Self(full))
    }

    /// A key read back from a store: same form, the prefix must be one of [`OwnerKind`].
    ///
    /// # Errors
    /// `key-invalid` when the text is no key.
    pub fn parse(text: &str) -> Result<Self, Coded<SettingCode>> {
        let mut parts = text.splitn(3, '.');
        let (Some(prefix), Some(owner), Some(local)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(invalid(text));
        };
        let kind = OwnerKind::ALL
            .iter()
            .copied()
            .find(|k| k.prefix() == prefix)
            .ok_or_else(|| invalid(text))?;
        Self::new(&Owner::new(kind, owner).map_err(|_| invalid(text))?, local)
    }

    /// The key as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The group a setting is shown in (`look`, `language`, `privacy`): one word.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroupId(String);

impl GroupId {
    /// The group `name`.
    ///
    /// # Errors
    /// `key-invalid` when `name` is no word.
    pub fn new(name: &str) -> Result<Self, Coded<SettingCode>> {
        if word_ok(name) {
            Ok(Self(name.to_owned()))
        } else {
            Err(invalid(name))
        }
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The names Windows reserves for devices: no file or folder may be called so.
fn windows_device(w: &str) -> bool {
    matches!(w, "con" | "prn" | "aux" | "nul")
        || ["com", "lpt"].iter().any(|p| {
            w.strip_prefix(p)
                .is_some_and(|n| matches!(n.as_bytes(), [b'1'..=b'9']))
        })
}

/// The name of a program whose profile is meant (`tasks`), the same form as the folder name of ADR 0057
/// (fs-components-words `setting.rs:37`), at most 32 bytes. It becomes a folder name: no `/`, no `..`. The word `language`
/// is reserved: `freesynergy/language` is the old language file of ADR 0057 (review N4).
///
/// Read tolerantly, written strictly (the principle, as for the persons and times at the border): `fs-texts` and `texts` are the same program, the name is kept as the SHORT
/// form without the leading `fs-`, so `as_str()`, the folder and the owner of its keys are always `texts`. Only ONE prefix is
/// allowed: a name that still begins with `fs-` after the cut (`fs-fs-x`) is invalid, so the result never begins with
/// `fs-` and `new` is idempotent (`new(new(x).as_str())` is the same name). `fs-` alone leaves nothing and is invalid,
/// `language` stays reserved after the cut, and so are the Windows device names (`con`, `prn`, `aux`, `nul`, `com1` to
/// `com9`, `lpt1` to `lpt9`), which cannot be file or folder names there (review A5).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProgramName(String);

impl ProgramName {
    /// The program `name`.
    ///
    /// # Errors
    /// `program-invalid` (parameter = the text as given) when the short name is no word, is `language`, a Windows device name,
    /// or still begins with `fs-` (a second prefix).
    pub fn new(name: &str) -> Result<Self, Coded<SettingCode>> {
        let short = name.strip_prefix("fs-").unwrap_or(name);
        if word_ok(short)
            && short != "language"
            && !short.starts_with("fs-")
            && !windows_device(short)
        {
            Ok(Self(short.to_owned()))
        } else {
            Err(Coded::new(SettingCode::ProgramInvalid).with_text(keys::PROGRAM, name))
        }
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

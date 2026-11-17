//! The standard store: TOML files after the XDG Base Directory Specification.
//!
//! - instance: `$XDG_CONFIG_DIRS` (first absolute entry, else `/etc/xdg`) `/freesynergy/instance.toml`, tables `[values]`
//!   and `[locks]` (key = `{ reason = "…", by = "<actor>", at = <UTC seconds> }`). Read only here (draft O11).
//! - profiles: `$XDG_CONFIG_HOME` (else `$HOME/.config`) `/freesynergy/settings.toml` (general) and
//!   `/freesynergy/<program>/settings.toml`; device place: the same below `$XDG_STATE_HOME` (else `$HOME/.local/state`).
//! - A file holds flat keys only: `"fs.shell.language" = "de"`. 0600, folders 0700, written atomically (`files_write.rs`).
//! - The old language files of ADR 0057 (`freesynergy/language`, `freesynergy/<program>/language`) are READ when the
//!   profile has no `fs.shell.language`, never written (draft O10).
//!
//! As in fs-components-words `setting.rs:25-33`: a relative `$XDG_*` is ignored. Checked before opening: a regular file of
//! at most 64 KiB. One handle serves one program (`program`); [`Store::write`] refuses another program's profile (the
//! crate's promise, not a security border: same user, same files; draft O9).
//!
//! Pattern: Strategy ([`Store`]) + Adapter over the `toml` crate.

use std::ffi::OsString;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::declare::Place;
use crate::layer::Scope;
use crate::lock::Lock;
use crate::names::{ProgramName, SettingKey};
use crate::store::{Instance, Snapshot, Stamp, Store};
use crate::value::Raw;

pub(crate) const MAX_BYTES: u64 = 64 * 1024;

/// The files of one program.
#[derive(Clone, Debug)]
pub struct XdgFiles {
    pub(crate) program: ProgramName,
    pub(crate) config: Option<PathBuf>,
    pub(crate) state: Option<PathBuf>,
    instance: PathBuf,
}

fn absolute(v: Option<OsString>) -> Option<PathBuf> {
    v.map(PathBuf::from).filter(|p| p.is_absolute())
}

pub(crate) fn kind_token(e: &std::io::Error) -> &'static str {
    match e.kind() {
        std::io::ErrorKind::NotFound => "not-found",
        std::io::ErrorKind::PermissionDenied => "permission-denied",
        _ => "other",
    }
}

impl XdgFiles {
    /// From the variables (pass them; tests never read the real environment): `XDG_CONFIG_HOME`, `XDG_STATE_HOME`,
    /// `HOME`, `XDG_CONFIG_DIRS`.
    #[must_use]
    pub fn new(
        program: ProgramName,
        config_home: Option<OsString>,
        state_home: Option<OsString>,
        home: Option<OsString>,
        config_dirs: Option<OsString>,
    ) -> Self {
        let home = absolute(home);
        let config = absolute(config_home).or_else(|| home.as_ref().map(|h| h.join(".config")));
        let state = absolute(state_home).or_else(|| home.as_ref().map(|h| h.join(".local/state")));
        let dirs = config_dirs.and_then(|d| std::env::split_paths(&d).find(|p| p.is_absolute()));
        let instance = dirs
            .unwrap_or_else(|| PathBuf::from("/etc/xdg"))
            .join("freesynergy")
            .join("instance.toml");
        Self {
            program,
            config,
            state,
            instance,
        }
    }

    /// The file of `scope` at `place`.
    ///
    /// # Errors
    /// `no-folder` (`which` = `config` or `state`).
    pub fn file(&self, scope: &Scope, place: Place) -> Result<PathBuf, Coded<SettingCode>> {
        let (base, which) = match place {
            Place::Profile => (&self.config, "config"),
            Place::Device => (&self.state, "state"),
        };
        let base = base
            .as_ref()
            .ok_or_else(|| Coded::new(SettingCode::NoFolder).with_text(keys::WHICH, which))?;
        let dir = base.join("freesynergy");
        Ok(match scope {
            Scope::General => dir.join("settings.toml"),
            Scope::Program(p) => dir.join(p.as_str()).join("settings.toml"),
        })
    }

    fn legacy_language(&self, scope: &Scope) -> Option<PathBuf> {
        let dir = self.config.as_ref()?.join("freesynergy");
        Some(match scope {
            Scope::General => dir.join("language"),
            Scope::Program(p) => dir.join(p.as_str()).join("language"),
        })
    }

    fn stamp_files(&self) -> Vec<PathBuf> {
        let own = Scope::Program(self.program.clone());
        let mut v = vec![self.instance.clone()];
        for scope in [Scope::General, own] {
            v.extend(
                [Place::Profile, Place::Device]
                    .into_iter()
                    .filter_map(|p| self.file(&scope, p).ok()),
            );
            v.extend(self.legacy_language(&scope));
        }
        v
    }
}

/// The text of `file`: `Ok(None)` when it does not exist.
pub(crate) fn read_text(file: &Path) -> Result<Option<String>, Coded<SettingCode>> {
    let shown = file.display().to_string();
    let with_file = |code: SettingCode| Coded::new(code).with_text(keys::FILE, shown.clone());
    match std::fs::metadata(file) {
        Ok(m) if !m.is_file() => return Err(with_file(SettingCode::NotRegular)),
        Ok(m) if m.len() > MAX_BYTES => return Err(with_file(SettingCode::FileTooLarge)),
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(
                with_file(SettingCode::FileUnreadable).with_text(keys::KIND, kind_token(&e))
            );
        }
    }
    let mut text = String::new();
    std::fs::File::open(file)
        .and_then(|f| f.take(MAX_BYTES + 1).read_to_string(&mut text))
        .map_err(|e| {
            with_file(SettingCode::FileUnreadable).with_text(keys::KIND, kind_token(&e))
        })?;
    if text.len() as u64 > MAX_BYTES {
        return Err(with_file(SettingCode::FileTooLarge));
    }
    Ok(Some(text))
}

/// The table of `file`, empty when it does not exist.
pub(crate) fn read_table(file: &Path) -> Result<toml::Table, Coded<SettingCode>> {
    let Some(text) = read_text(file)? else {
        return Ok(toml::Table::new());
    };
    text.parse::<toml::Table>().map_err(|e| {
        let line = e
            .span()
            .map_or(0, |s| text[..s.start].matches('\n').count() + 1);
        Coded::new(SettingCode::FileBroken)
            .with_text(keys::FILE, file.display().to_string())
            .with_number(keys::LINE, line as u64)
    })
}

/// A lock from its inline table; `None` when a field is missing or of another type.
fn lock_of(t: &toml::Table) -> Option<Lock> {
    Some(Lock {
        reason: t.get("reason")?.as_str()?.to_owned(),
        locked_by: t.get("by")?.as_str()?.to_owned(),
        locked_at: t.get("at")?.as_integer()?,
    })
}

impl Store for XdgFiles {
    fn stamp(&self) -> Stamp {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for f in self.stamp_files() {
            f.hash(&mut h);
            if let Ok(m) = std::fs::metadata(&f) {
                (m.len(), m.modified().ok()).hash(&mut h);
            }
        }
        Stamp(h.finish())
    }

    fn instance(&self) -> Instance {
        let mut out = Instance::default();
        match read_table(&self.instance) {
            Err(p) => out.values.problems.push(p),
            Ok(t) => {
                if let Some(toml::Value::Table(values)) = t.get("values") {
                    out.values = crate::files_write::snapshot(values, &self.instance);
                }
                if let Some(toml::Value::Table(locks)) = t.get("locks") {
                    for (k, v) in locks {
                        if let (Ok(key), Some(lock)) =
                            (SettingKey::parse(k), v.as_table().and_then(lock_of))
                        {
                            out.locks.insert(key, lock);
                        } else {
                            out.values.problems.push(
                                Coded::new(SettingCode::WrongType).with_text(keys::KEY, k.clone()),
                            );
                        }
                    }
                }
            }
        }
        out
    }

    fn profile(&self, scope: &Scope, place: Place) -> Snapshot {
        crate::files_write::profile(self, scope, place)
    }

    fn write(
        &self,
        scope: &Scope,
        place: Place,
        key: &SettingKey,
        raw: Option<&Raw>,
    ) -> Result<(), Coded<SettingCode>> {
        crate::files_write::write(self, scope, place, key, raw)
    }
}

impl XdgFiles {
    pub(crate) fn legacy(&self, scope: &Scope) -> Option<PathBuf> {
        self.legacy_language(scope)
    }
}

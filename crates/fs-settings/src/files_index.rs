//! The index of the named profiles in files (Nachzug 17.11.): the names travel with the person, the active profile
//! belongs to the device.
//!
//! - `$XDG_CONFIG_HOME/freesynergy/profiles.toml`: `top = <highest id ever given>`, `[names]` `"Arbeit" = 1`.
//! - `$XDG_STATE_HOME/freesynergy/profiles.toml`: `active = <id>` (0 or missing = the base).
//!
//! Changes go through [`update`]: one read-change-write under the file locks (review N1), so two programs creating a
//! profile at the same moment never get the same id.
//!
//! Pattern: Adapter between `ProfileSet<u64>` and two small TOML files.

use std::path::PathBuf;

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::files::{XdgFiles, kind_token, read_table};
use crate::files_write::{lock_dir, make_dir, replace};
use crate::profile::ProfileSet;
use crate::profile_name::{ProfileName, ProfileRef};
use crate::store::IndexFn;

fn path(base: Option<&PathBuf>, which: &str) -> Result<PathBuf, Coded<SettingCode>> {
    base.map(|b| b.join("freesynergy").join("profiles.toml"))
        .ok_or_else(|| Coded::new(SettingCode::NoFolder).with_text(keys::WHICH, which))
}

/// The two index files (names, active).
pub(crate) fn files(x: &XdgFiles) -> Vec<PathBuf> {
    [
        path(x.config.as_ref(), "config"),
        path(x.state.as_ref(), "state"),
    ]
    .into_iter()
    .filter_map(Result::ok)
    .collect()
}

fn problem(code: SettingCode, name: &str) -> Coded<SettingCode> {
    Coded::new(code)
        .with_text(keys::CAUSE, "unknown")
        .with_text(keys::NAME, name)
}

/// Reads the index; every problem is listed, the base is the fallback.
pub(crate) fn read(x: &XdgFiles) -> (ProfileSet<u64>, Vec<Coded<SettingCode>>) {
    let mut problems = Vec::new();
    let mut set = ProfileSet::new(0u64);
    match path(x.config.as_ref(), "config").and_then(|f| read_table(&f)) {
        Err(p) => problems.push(p),
        Ok(t) => {
            let top = t
                .get("top")
                .and_then(toml::Value::as_integer)
                .and_then(|n| u64::try_from(n).ok())
                .unwrap_or(0);
            let mut names: Vec<(u64, &String)> = t
                .get("names")
                .and_then(toml::Value::as_table)
                .map(|n| {
                    n.iter()
                        .filter_map(|(k, v)| Some((u64::try_from(v.as_integer()?).ok()?, k)))
                        .collect()
                })
                .unwrap_or_default();
            names.sort();
            let mut seen = std::collections::BTreeSet::new();
            for (id, text) in names {
                // id 0 is the base, a second use of an id would share values, an id above `top` was never given
                if id == 0 || id > top || !seen.insert(id) {
                    problems.push(
                        Coded::new(SettingCode::Profile)
                            .with_text(keys::CAUSE, "id-invalid")
                            .with_text(keys::NAME, text.clone()),
                    );
                    continue;
                }
                match ProfileName::new(text)
                    .map_err(|_| ())
                    .and_then(|n| set.create(n.clone(), None).map(|()| n).map_err(|_| ()))
                {
                    Ok(n) => {
                        if let Some(v) = set.get_mut(&ProfileRef::Named(n)) {
                            *v = id;
                        }
                    }
                    Err(()) => problems.push(
                        Coded::new(SettingCode::Profile)
                            .with_text(keys::CAUSE, "name-invalid")
                            .with_text(keys::NAME, text.clone()),
                    ),
                }
            }
            if let Some(v) = set.get_mut(&ProfileRef::Base) {
                *v = top;
            }
        }
    }
    let active = path(x.state.as_ref(), "state").and_then(|f| read_table(&f));
    match active.map(|t| {
        t.get("active")
            .and_then(toml::Value::as_integer)
            .unwrap_or(0)
    }) {
        Err(p) => problems.push(p),
        Ok(0) => {}
        Ok(id) => {
            let name = set
                .names()
                .find(|n| {
                    set.get(&ProfileRef::Named((*n).clone()))
                        == Some(&u64::try_from(id).unwrap_or(0))
                })
                .cloned();
            match name {
                Some(n) => {
                    let _ = set.switch(ProfileRef::Named(n));
                }
                None => problems.push(problem(SettingCode::Profile, &id.to_string())),
            }
        }
    }
    (set, problems)
}

/// The folder of an index file.
fn dir_of(file: &std::path::Path) -> Result<&std::path::Path, Coded<SettingCode>> {
    file.parent().ok_or_else(|| {
        Coded::new(SettingCode::WriteFailed)
            .with_text(keys::FILE, file.display().to_string())
            .with_text(keys::KIND, "other")
    })
}

/// Read-change-write of both index files under the file locks (`.settings.lock` in the config folder first, then in the
/// state folder: a fixed order; one lock when both are one folder, found by the real path). A broken or unreadable index
/// is never overwritten. The two files are replaced one after the other, each atomically: after a crash between them the
/// OLD `active` stands, and an id without a name gives the base and `profile{cause=unknown}` (said, not silent). An entry
/// that was reported as `id-invalid` or `name-invalid` when reading is not written back: it drops out of the file at the
/// next update (review L6, L7).
pub(crate) fn update(
    x: &XdgFiles,
    f: &mut IndexFn<'_>,
) -> Result<ProfileSet<u64>, Coded<SettingCode>> {
    let (config_file, state_file) = (
        path(x.config.as_ref(), "config")?,
        path(x.state.as_ref(), "state")?,
    );
    let (config_dir, state_dir) = (dir_of(&config_file)?, dir_of(&state_file)?);
    let fail = |file: &std::path::Path, kind: &str| {
        Coded::new(SettingCode::WriteFailed)
            .with_text(keys::FILE, file.display().to_string())
            .with_text(keys::KIND, kind)
    };
    make_dir(config_dir).map_err(|e| fail(&config_file, kind_token(&e)))?;
    make_dir(state_dir).map_err(|e| fail(&state_file, kind_token(&e)))?;
    let _config_lock = lock_dir(config_dir).map_err(|e| fail(&config_file, kind_token(&e)))?;
    // The same folder under another spelling (a symlink, `..`) must not be locked twice: the second `flock` of the same
    // process would wait for the first for ever (review L6). Compared by the real path, after both folders exist.
    let same_folder = match (
        std::fs::canonicalize(config_dir),
        std::fs::canonicalize(state_dir),
    ) {
        (Ok(c), Ok(s)) => c == s,
        _ => state_dir == config_dir,
    };
    let _state_lock = if same_folder {
        None
    } else {
        Some(lock_dir(state_dir).map_err(|e| fail(&state_file, kind_token(&e)))?)
    };
    let (mut set, problems) = read(x);
    let unsafe_to_write = |p: &&Coded<SettingCode>| {
        matches!(
            p.code(),
            SettingCode::FileBroken
                | SettingCode::FileUnreadable
                | SettingCode::FileTooLarge
                | SettingCode::NotRegular
        )
    };
    if let Some(p) = problems.iter().find(unsafe_to_write) {
        return Err(p.clone());
    }
    f(&mut set).map_err(|p| crate::settings_profiles::mapped(&p))?;
    let mut names = toml::Table::new();
    for n in set.names() {
        let id = set.get(&ProfileRef::Named(n.clone())).copied().unwrap_or(0);
        names.insert(
            n.as_str().to_owned(),
            toml::Value::Integer(i64::try_from(id).unwrap_or(i64::MAX)),
        );
    }
    let top = set.get(&ProfileRef::Base).copied().unwrap_or(0);
    let mut config = toml::Table::new();
    config.insert(
        "top".to_owned(),
        toml::Value::Integer(i64::try_from(top).unwrap_or(i64::MAX)),
    );
    config.insert("names".to_owned(), toml::Value::Table(names));
    let mut state = toml::Table::new();
    let active = crate::settings::active_id(&set);
    state.insert(
        "active".to_owned(),
        toml::Value::Integer(i64::try_from(active).unwrap_or(0)),
    );
    // One folder means one file: names, `top` and `active` go into it together (two writes would lose the first).
    let tables = if same_folder {
        config.extend(state);
        vec![(&config_file, config)]
    } else {
        vec![(&config_file, config), (&state_file, state)]
    };
    for (file, table) in tables {
        let text = toml::to_string(&table).map_err(|_| fail(file, "other"))?;
        replace(file, text.as_bytes()).map_err(|e| fail(file, kind_token(&e)))?;
    }
    Ok(set)
}

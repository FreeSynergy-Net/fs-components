//! The index of the named profiles in files (Nachzug 17.11.): the names travel with the person, the active profile
//! belongs to the device.
//!
//! - `$XDG_CONFIG_HOME/freesynergy/profiles.toml`: `top = <highest id ever given>`, `[names]` `"Arbeit" = 1`.
//! - `$XDG_STATE_HOME/freesynergy/profiles.toml`: `active = <id>` (0 or missing = the base).
//!
//! Pattern: Adapter between `ProfileSet<u64>` and two small TOML files.

use std::path::PathBuf;

use crate::coded_stub::Coded;

use crate::codes::{SettingCode, keys};
use crate::files::{XdgFiles, kind_token, read_table};
use crate::files_write::replace;
use crate::profile::ProfileSet;
use crate::profile_name::{ProfileName, ProfileRef};

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
            let mut highest = top;
            for (id, text) in names {
                match ProfileName::new(text)
                    .map_err(|_| ())
                    .and_then(|n| set.create(n.clone(), None).map(|()| n).map_err(|_| ()))
                {
                    Ok(n) => {
                        if let Some(v) = set.get_mut(&ProfileRef::Named(n)) {
                            *v = id;
                        }
                        highest = highest.max(id);
                    }
                    Err(()) => problems.push(
                        Coded::new(SettingCode::Profile)
                            .with_text(keys::CAUSE, "name-invalid")
                            .with_text(keys::NAME, text.clone()),
                    ),
                }
            }
            if let Some(v) = set.get_mut(&ProfileRef::Base) {
                *v = highest;
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

/// Writes both index files (whole files: they hold nothing else).
pub(crate) fn write(x: &XdgFiles, set: &ProfileSet<u64>) -> Result<(), Coded<SettingCode>> {
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
    let active = crate::settings::active_id(set);
    let mut state = toml::Table::new();
    state.insert(
        "active".to_owned(),
        toml::Value::Integer(i64::try_from(active).unwrap_or(0)),
    );
    for (file, table) in [
        (path(x.config.as_ref(), "config")?, config),
        (path(x.state.as_ref(), "state")?, state),
    ] {
        let fail = |kind: &str| {
            Coded::new(SettingCode::WriteFailed)
                .with_text(keys::FILE, file.display().to_string())
                .with_text(keys::KIND, kind)
        };
        let text = toml::to_string(&table).map_err(|_| fail("other"))?;
        replace(&file, text.as_bytes()).map_err(|e| fail(kind_token(&e)))?;
    }
    Ok(())
}

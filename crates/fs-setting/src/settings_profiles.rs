//! The person's named profiles, from inside any program: list, create, switch, rename, delete (answer 16, Nachzug
//! 17.11.). A program changes only the person's OWN profiles (the store is the person's; no parameter names another
//! person); deleting clears the profile's values in the general part and in this program's part, other programs' parts
//! stay unread because the id is never reused.
//!
//! Pattern: Facade over [`crate::ProfileSet`] (the rules) and the store (keeping); every change refreshes, so a switch is
//! LIVE: each changed key is told once.

use crate::coded_stub::{Coded, ErrorCode};

use crate::codes::{SettingCode, keys};
use crate::layer::Scope;
use crate::profile::ProfileSet;
use crate::profile_name::{ProfileCode, ProfileName, ProfileRef};
use crate::settings::{Change, Settings};

fn mapped(p: &Coded<ProfileCode>) -> Coded<SettingCode> {
    let name = match p.params().find(|(k, _)| *k == crate::profile_name::NAME) {
        Some((_, crate::coded_stub::Param::Text(t))) => t.clone(),
        _ => String::new(),
    };
    Coded::new(SettingCode::Profile)
        .with_text(keys::CAUSE, p.code().as_str())
        .with_text(keys::NAME, name)
}

fn name(text: &str) -> Result<ProfileName, Coded<SettingCode>> {
    ProfileName::new(text).map_err(|p| mapped(&p))
}

impl Settings {
    /// The person's profiles: names, active one (the values are ids; the base's value is the highest id ever given).
    #[must_use]
    pub fn profiles(&self) -> ProfileSet<u64> {
        self.store.index().0
    }

    fn change_index(
        &self,
        f: impl FnOnce(&mut ProfileSet<u64>) -> Result<(), Coded<ProfileCode>>,
    ) -> Result<Vec<Change>, Coded<SettingCode>> {
        let mut index = self.store.index().0;
        f(&mut index).map_err(|p| mapped(&p))?;
        self.store.write_index(&index)?;
        Ok(self.refresh())
    }

    /// Adds the profile `text`, empty: it inherits everything from the base until something is saved in it.
    ///
    /// # Errors
    /// `profile{cause=name-invalid|exists}`; the store's codes.
    pub fn create_profile(&self, text: &str) -> Result<Vec<Change>, Coded<SettingCode>> {
        let n = name(text)?;
        self.change_index(|i| {
            let next = i.get(&ProfileRef::Base).copied().unwrap_or(0) + 1;
            i.create(n.clone(), None)?;
            if let Some(id) = i.get_mut(&ProfileRef::Named(n)) {
                *id = next;
            }
            if let Some(top) = i.get_mut(&ProfileRef::Base) {
                *top = next;
            }
            Ok(())
        })
    }

    /// Makes `text` the active profile (`None` = the base). Live: the changed keys are told.
    ///
    /// # Errors
    /// `profile{cause=unknown}`; the store's codes.
    pub fn switch_profile(&self, text: Option<&str>) -> Result<Vec<Change>, Coded<SettingCode>> {
        let to = match text {
            Some(t) => ProfileRef::Named(name(t)?),
            None => ProfileRef::Base,
        };
        self.change_index(|i| i.switch(to))
    }

    /// Renames `old` to `new`.
    ///
    /// # Errors
    /// `profile{cause=unknown|exists|name-invalid}`; the store's codes.
    pub fn rename_profile(&self, old: &str, new: &str) -> Result<Vec<Change>, Coded<SettingCode>> {
        let (old, new) = (name(old)?, name(new)?);
        self.change_index(|i| i.rename(&old, new))
    }

    /// Deletes `text`. Was it active, the base is active again and every changed key is told (never silent).
    ///
    /// # Errors
    /// `profile{cause=unknown}`; the store's codes.
    pub fn delete_profile(&self, text: &str) -> Result<Vec<Change>, Coded<SettingCode>> {
        let n = name(text)?;
        let index = self.store.index().0;
        let id = index.get(&ProfileRef::Named(n.clone())).copied();
        let changes = self.change_index(|i| i.delete(&n).map(|_| ()))?;
        if let Some(id) = id {
            self.store.clear(&Scope::General, id)?;
            self.store
                .clear(&Scope::Program(self.program.clone()), id)?;
        }
        let mut more = self.refresh();
        let mut all = changes;
        all.append(&mut more);
        Ok(all)
    }
}

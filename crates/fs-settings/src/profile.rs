//! Named profiles as a reusable building block: one base that always exists ("Allgemein"), any number of named ones
//! ("Arbeit", "Vortrag"), one active, switching, and inheritance (a named profile holds only what differs; the rest
//! comes from the base).
//!
//! User (GESPRAECH-4-wortlaut.md Z. 48, answer 16): "Nein, mach es bitte sofort. … Denn, das Prinzip werden wir noch
//! sehr oft haben. das könnte man da copieren". Therefore generic over the values `V` and free of everything in this
//! crate: settings use `ProfileSet<()>` as their index (names, active); further users: rule presets "wie Outlook"/"wie
//! Apple" (ADR 0097 decision 4), frontend choice, themes, starter.
//!
//! STAND-IN for the data kit `fs_types::profile` (plan G2c item 6, N33; file `crates/fs-types/src/profile.rs` plus one
//! re-export line in `lib.rs`; additive, no new dependency, only `coded`). When the fs-types tag carries it, this file is
//! deleted and the crate re-exports `fs_types::profile` unchanged (as `fault.rs` → `fs_types::coded`).
//!
//! Pattern: Value objects (`ProfileName`, `ProfileRef`) + a small aggregate (`ProfileSet`) that keeps its rules: names
//! unique ignoring case, the base never deleted or renamed, the active profile always exists.

use fs_types::coded::Coded;

use crate::profile_name::{ProfileCode, ProfileName, ProfileRef, fault};

/// The base, the named profiles in the order they were made, and the active one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileSet<V> {
    base: V,
    named: Vec<(ProfileName, V)>,
    active: ProfileRef,
}

impl<V: Clone + Default> ProfileSet<V> {
    /// Only the base, active.
    pub fn new(base: V) -> Self {
        Self {
            base,
            named: Vec::new(),
            active: ProfileRef::Base,
        }
    }

    fn find(&self, name: &ProfileName) -> Option<usize> {
        self.named.iter().position(|(n, _)| n.same(name))
    }

    /// The names, in order.
    pub fn names(&self) -> impl Iterator<Item = &ProfileName> {
        self.named.iter().map(|(n, _)| n)
    }

    /// The active profile.
    pub fn active(&self) -> &ProfileRef {
        &self.active
    }

    /// The values of `which`.
    pub fn get(&self, which: &ProfileRef) -> Option<&V> {
        match which {
            ProfileRef::Base => Some(&self.base),
            ProfileRef::Named(n) => self.find(n).map(|i| &self.named[i].1),
        }
    }

    /// The values of `which`, to change.
    pub fn get_mut(&mut self, which: &ProfileRef) -> Option<&mut V> {
        match which {
            ProfileRef::Base => Some(&mut self.base),
            ProfileRef::Named(n) => self.find(n).map(|i| &mut self.named[i].1),
        }
    }

    /// The lookup order: the active named profile first (if one), then the base. A value missing in a named profile is
    /// inherited from the base.
    pub fn chain(&self) -> Vec<&V> {
        let mut c: Vec<&V> = self.get(&self.active).into_iter().collect();
        if self.active != ProfileRef::Base {
            c.push(&self.base);
        }
        c
    }

    /// Adds `name`, empty (inherits everything) or as a copy of `from`.
    ///
    /// # Errors
    /// `exists`; `unknown` for `from`.
    pub fn create(
        &mut self,
        name: ProfileName,
        from: Option<&ProfileRef>,
    ) -> Result<(), Coded<ProfileCode>> {
        if self.find(&name).is_some() {
            return Err(fault(ProfileCode::Exists, name.as_str()));
        }
        let values = match from {
            None => V::default(),
            Some(r) => self
                .get(r)
                .cloned()
                .ok_or_else(|| fault(ProfileCode::Unknown, label(r)))?,
        };
        self.named.push((name, values));
        Ok(())
    }

    /// Renames `old` to `new` (the same name in other case is allowed).
    ///
    /// # Errors
    /// `unknown`; `exists`.
    pub fn rename(
        &mut self,
        old: &ProfileName,
        new: ProfileName,
    ) -> Result<(), Coded<ProfileCode>> {
        let i = self
            .find(old)
            .ok_or_else(|| fault(ProfileCode::Unknown, old.as_str()))?;
        if self.find(&new).is_some_and(|j| j != i) {
            return Err(fault(ProfileCode::Exists, new.as_str()));
        }
        if self.active == ProfileRef::Named(self.named[i].0.clone()) {
            self.active = ProfileRef::Named(new.clone());
        }
        self.named[i].0 = new;
        Ok(())
    }

    /// Deletes `name`; returns whether it was the active one (then the base is active: the caller tells the person).
    ///
    /// # Errors
    /// `unknown`.
    pub fn delete(&mut self, name: &ProfileName) -> Result<bool, Coded<ProfileCode>> {
        let i = self
            .find(name)
            .ok_or_else(|| fault(ProfileCode::Unknown, name.as_str()))?;
        let (gone, _) = self.named.remove(i);
        let was_active = matches!(&self.active, ProfileRef::Named(a) if a.same(&gone));
        if was_active {
            self.active = ProfileRef::Base;
        }
        Ok(was_active)
    }

    /// Makes `to` the active profile.
    ///
    /// # Errors
    /// `unknown`.
    pub fn switch(&mut self, to: ProfileRef) -> Result<(), Coded<ProfileCode>> {
        if let ProfileRef::Named(n) = to {
            let i = self
                .find(&n)
                .ok_or_else(|| fault(ProfileCode::Unknown, n.as_str()))?;
            self.active = ProfileRef::Named(self.named[i].0.clone());
        } else {
            self.active = ProfileRef::Base;
        }
        Ok(())
    }
}

impl<V: Clone + Default> Default for ProfileSet<V> {
    fn default() -> Self {
        Self::new(V::default())
    }
}

fn label(r: &ProfileRef) -> &str {
    match r {
        ProfileRef::Base => "",
        ProfileRef::Named(n) => n.as_str(),
    }
}

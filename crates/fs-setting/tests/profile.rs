//! K31 to K36 and the first part of K44 (named profiles as a building block: names, set, chain; ENTWURF section 16).
mod support;

use fs_setting::coded_stub::{ErrorCode, message_key};
use fs_setting::{ProfileCode, ProfileName, ProfileRef, ProfileSet};
use std::collections::HashSet;

fn n(t: &str) -> ProfileName {
    ProfileName::new(t).unwrap()
}

fn named(t: &str) -> ProfileRef {
    ProfileRef::Named(n(t))
}

/// K31: names are trimmed, 1 to 64 characters, one line.
#[test]
fn k31_profile_name() {
    assert_eq!(n("Arbeit").as_str(), "Arbeit");
    assert_eq!(n(" Vortrag ").as_str(), "Vortrag");
    assert!(n("Arbeit").same(&n("ARBEIT")));
    for bad in ["", "   ", &"x".repeat(65), "a\nb"] {
        let e = ProfileName::new(bad).unwrap_err();
        assert_eq!(e.code(), ProfileCode::NameInvalid, "{bad:?}");
    }
    assert!(ProfileName::new(&"x".repeat(64)).is_ok());
}

/// K32: a second name that differs only in case exists already.
#[test]
fn k32_create_unique() {
    let mut set = ProfileSet::new(0u64);
    set.create(n("Arbeit"), None).unwrap();
    assert_eq!(
        set.create(n("arbeit"), None).unwrap_err().code(),
        ProfileCode::Exists
    );
    assert_eq!(set.names().count(), 1);
    assert_eq!(
        set.create(n("Kopie"), Some(&named("Fehlt")))
            .unwrap_err()
            .code(),
        ProfileCode::Unknown
    );
}

/// K33: switching to the unknown fails; the case of the name is ignored, the stored spelling is kept.
#[test]
fn k33_switch() {
    let mut set = ProfileSet::new(0u64);
    set.create(n("Arbeit"), None).unwrap();
    assert_eq!(
        set.switch(named("Fehlt")).unwrap_err().code(),
        ProfileCode::Unknown
    );
    assert_eq!(set.active(), &ProfileRef::Base);
    set.switch(named("ARBEIT")).unwrap();
    assert_eq!(set.active(), &named("Arbeit"));
    assert_eq!(
        set.active(),
        &ProfileRef::Named(set.names().next().unwrap().clone())
    );
    set.switch(ProfileRef::Base).unwrap();
    assert_eq!(set.active(), &ProfileRef::Base);
}

/// K34: deleting the active profile answers `true` and the base is active; another one answers `false`.
#[test]
fn k34_delete() {
    let mut set = ProfileSet::new(0u64);
    set.create(n("Arbeit"), None).unwrap();
    set.create(n("Vortrag"), None).unwrap();
    set.switch(named("Arbeit")).unwrap();
    assert!(!set.delete(&n("Vortrag")).unwrap());
    assert_eq!(set.active(), &named("Arbeit"));
    assert!(set.delete(&n("arbeit")).unwrap());
    assert_eq!(set.active(), &ProfileRef::Base);
    assert_eq!(
        set.delete(&n("Arbeit")).unwrap_err().code(),
        ProfileCode::Unknown
    );
}

/// K35: rename onto an existing name fails; onto its own name in another case works; the active one is followed.
#[test]
fn k35_rename() {
    let mut set = ProfileSet::new(0u64);
    set.create(n("Arbeit"), None).unwrap();
    set.create(n("Vortrag"), None).unwrap();
    assert_eq!(
        set.rename(&n("Arbeit"), n("vortrag")).unwrap_err().code(),
        ProfileCode::Exists
    );
    set.rename(&n("Arbeit"), n("ARBEIT")).unwrap();
    set.switch(named("arbeit")).unwrap();
    set.rename(&n("Arbeit"), n("Büro")).unwrap();
    assert_eq!(set.active(), &named("Büro"));
    assert_eq!(
        set.rename(&n("Fehlt"), n("X")).unwrap_err().code(),
        ProfileCode::Unknown
    );
}

/// K36: the chain is the base alone, or the named profile and then the base; a copy starts with the values of its source.
#[test]
fn k36_chain() {
    let mut set = ProfileSet::new(7u64);
    assert_eq!(set.chain(), [&7]);
    set.create(n("Arbeit"), Some(&ProfileRef::Base)).unwrap();
    *set.get_mut(&named("Arbeit")).unwrap() = 9;
    set.switch(named("Arbeit")).unwrap();
    assert_eq!(set.chain(), [&9, &7]);
    set.create(n("Kopie"), Some(&named("Arbeit"))).unwrap();
    assert_eq!(set.get(&named("Kopie")), Some(&9));
}

/// K44 (first part): the profile codes, once each, kebab names, message keys.
#[test]
fn k44_profile_codes() {
    assert_eq!(ProfileCode::ALL.len(), 3);
    let names: HashSet<_> = ProfileCode::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(names.len(), 3);
    for c in ProfileCode::ALL {
        assert_eq!(message_key(*c), format!("profile-error-{}", c.as_str()));
    }
}

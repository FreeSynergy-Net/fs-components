//! The suite `settings`: its id and its rules in order.
//!
//! Pattern: Abstract factory (makes the rules).

use fs_conformance::{Rule, Suite, SuiteId};

/// The suite `settings`.
#[derive(Clone, Copy, Debug, Default)]
pub struct SettingsSuite;

impl<F: super::SettingsFixture + ?Sized> Suite<F> for SettingsSuite {
    fn id(&self) -> SuiteId {
        SuiteId::new("settings")
    }

    fn rules(&self) -> Vec<Box<dyn Rule<F>>> {
        vec![
            Box::new(super::rules_layers::CatalogValid),
            Box::new(super::rules_layers::TextsBoth),
            Box::new(super::rules_layers::StandardKeys),
            Box::new(super::rules_layers::ProgramWins),
            Box::new(super::rules_layers::ThisProgramOnly),
            Box::new(super::rules_layers::AllFollows),
            Box::new(super::rules_layers::Live),
            Box::new(super::rules_layers::BadValueVisible),
            Box::new(super::rules_layers::ResetFollows),
            Box::new(super::rules_lock::LockWins),
            Box::new(super::rules_lock::LockHolds),
            Box::new(super::rules_lock::LockButton),
            Box::new(super::rules_lock::FrontendChoice),
            Box::new(super::rules_profiles::ProfileSwitchLive),
            Box::new(super::rules_profiles::ProfileInherits),
            Box::new(super::rules_profiles::ProfileDeleteActive),
            Box::new(super::rules_profiles::LockAllProfiles),
            Box::new(super::rules_profiles::ProfileNameUnique),
        ]
    }
}

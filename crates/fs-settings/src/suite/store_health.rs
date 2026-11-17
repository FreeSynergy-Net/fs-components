//! The rule `store-health-said`: a store whose storage is damaged says so (`Health::Defaults`, never `Ok`) and refuses writes.
//!
//! Pattern: Strategy (a rule written out, because it may be skipped: only a provider that can damage its storage runs it).

use async_trait::async_trait;
use fs_conformance::{Coded, Cx, Need, Rule, RuleId, Verdict};

use super::StoreFixture;
use super::check::std_key;
use super::codes::{SuiteCode, keys};
use crate::declare::Place;
use crate::layer::Scope;
use crate::standard::STYLE;
use crate::store::Health;
use crate::value::Raw;

/// A broken store says `Defaults` with at least one code, and takes no write.
pub(crate) struct HealthSaid;

#[async_trait(?Send)]
impl<F: StoreFixture + ?Sized> Rule<F> for HealthSaid {
    fn id(&self) -> RuleId {
        RuleId::new("store-health-said")
    }

    async fn check(&self, f: &mut F, cx: &Cx) -> Verdict {
        let Ok(key) = std_key(STYLE) else {
            return Verdict::Fail(Coded::new(SuiteCode::StandardMissing).erase());
        };
        let silent = |step: u64| {
            Verdict::Fail(
                Coded::new(SuiteCode::HealthSilent)
                    .with_text(keys::KEY, key.as_str())
                    .with_number(keys::STEP, step)
                    .erase(),
            )
        };
        let s = f.fresh();
        if s.health() != Health::Ok {
            return silent(1);
        }
        if !f.break_store(&*s) {
            return cx.missing(Need::new("fixture.break-store"));
        }
        match s.health() {
            Health::Defaults(codes) if !codes.is_empty() => {}
            _ => return silent(2),
        }
        let raw = Raw::Str("calm".to_owned());
        if s.write(&Scope::General, Place::Profile, 0, &key, Some(&raw))
            .is_ok()
        {
            return silent(3);
        }
        Verdict::Pass
    }
}

//! Helpers of the rules: each check is a `Result`, the first failure is the verdict.
//!
//! Pattern: Specification (small predicates) + a macro that makes a rule of a body (as fs-starter `suite/check.rs`).

use fs_conformance::Coded;

use super::codes::{SuiteCode, keys};
use crate::layer::Layer;
use crate::names::SettingKey;
use crate::settings::Settings;
use crate::standard::ShellSettings;
use crate::value::Value;

/// The result of one check.
pub(crate) type Checked = Result<(), Coded<SuiteCode>>;

/// A rule: `rule!(Type, "id", |fixture| { ...; Ok(()) })`. The fixture is reset first; the body is a [`Checked`].
macro_rules! rule {
    ($(#[$doc:meta])* $t:ident, $id:literal, |$f:ident| $body:block) => {
        $(#[$doc])*
        pub(crate) struct $t;

        #[async_trait::async_trait(?Send)]
        impl<F: $crate::suite::SettingsFixture + ?Sized> fs_conformance::Rule<F> for $t {
            fn id(&self) -> fs_conformance::RuleId {
                fs_conformance::RuleId::new($id)
            }

            async fn check(&self, $f: &mut F, _cx: &fs_conformance::Cx) -> fs_conformance::Verdict {
                fn body<F: $crate::suite::SettingsFixture + ?Sized>($f: &mut F) -> $crate::suite::check::Checked $body
                $f.reset();
                match body($f) {
                    Ok(()) => fs_conformance::Verdict::Pass,
                    Err(code) => fs_conformance::Verdict::Fail(code.erase()),
                }
            }
        }
    };
}
pub(crate) use rule;

/// A failure with `key` and `step`.
pub(crate) fn fail(c: SuiteCode, key: &SettingKey, step: u64) -> Coded<SuiteCode> {
    Coded::new(c)
        .with_text(keys::KEY, key.as_str())
        .with_number(keys::STEP, step)
}

/// A standard key (the constants are valid, K-test); a bad one fails as `standard-missing`.
pub(crate) fn std_key(which: (&str, &str)) -> Result<SettingKey, Coded<SuiteCode>> {
    ShellSettings::key(which)
        .map_err(|_| Coded::new(SuiteCode::StandardMissing).with_text(keys::KEY, which.1))
}

/// `s` must give `want` from `layer` for `key`, without lock unless `locked`.
pub(crate) fn expect(
    s: &Settings,
    key: &SettingKey,
    want: &Value,
    layer: Layer,
    locked: bool,
    step: u64,
) -> Checked {
    let e = s
        .get(key)
        .ok_or_else(|| fail(SuiteCode::WrongValue, key, step))?;
    if &e.value != want {
        return Err(fail(SuiteCode::WrongValue, key, step));
    }
    if e.from != layer {
        return Err(fail(SuiteCode::WrongLayer, key, step).with_text(keys::LAYER, layer.as_str()));
    }
    if e.lock.is_some() != locked {
        return Err(fail(SuiteCode::LockWrong, key, step));
    }
    Ok(())
}

/// A step that must go through.
pub(crate) fn ok<T>(
    r: Result<T, crate::coded_stub::Coded<crate::SettingCode>>,
    key: &SettingKey,
    step: u64,
) -> Checked {
    r.map(|_| ()).map_err(|e| {
        use crate::coded_stub::ErrorCode;
        fail(SuiteCode::Refused, key, step).with_text(keys::CODE, e.code().as_str())
    })
}

/// A step that must be refused.
pub(crate) fn refused<T>(
    r: Result<T, crate::coded_stub::Coded<crate::SettingCode>>,
    key: &SettingKey,
    step: u64,
) -> Checked {
    if let Ok(_kept) = r {
        Err(fail(SuiteCode::NotRefused, key, step))
    } else {
        Ok(())
    }
}

/// Two choice values of `key`'s kind other than its current value (for a choice key with at least two others), or
/// whole numbers for a whole key. `None` when the kind has no two other values.
pub(crate) fn two_others(s: &Settings, key: &SettingKey) -> Option<(Value, Value)> {
    let d = s.catalog().get(key)?;
    let now = s.get(key)?.value;
    let all: Vec<Value> = match &d.kind {
        crate::value::Kind::Choice { keys } => {
            keys.iter().map(|k| Value::Choice(k.clone())).collect()
        }
        crate::value::Kind::Whole { min, max } => (*min..=*max).take(8).map(Value::Whole).collect(),
        _ => Vec::new(),
    };
    let mut others = all.into_iter().filter(|v| *v != now);
    Some((others.next()?, others.next().unwrap_or(now)))
}

/// A step that must be refused WITH this code and these text parameters (a lost or changed code must not pass).
pub(crate) fn refused_as<T>(
    r: Result<T, crate::coded_stub::Coded<crate::SettingCode>>,
    key: &SettingKey,
    step: u64,
    code: crate::SettingCode,
    params: &[(&str, &str)],
) -> Checked {
    use crate::coded_stub::{ErrorCode, Param};
    let Err(e) = r else {
        return Err(fail(SuiteCode::NotRefused, key, step));
    };
    let has = |k: &str, v: &str| {
        e.params()
            .any(|(n, p)| n == k && *p == Param::Text(v.to_owned()))
    };
    if e.code() == code && params.iter().all(|(k, v)| has(k, v)) {
        Ok(())
    } else {
        Err(fail(SuiteCode::Refused, key, step).with_text(keys::CODE, e.code().as_str()))
    }
}

//! K8, K9 of ENTWURF section 5 on the one version rule (V-3): `semver::Version` for what something IS, `name@<req>` (Cargo
//! grammar, comma) for what it NEEDS.
use fs_components_catalog::{AcceptsCode, ContractRange};
use semver::Version;

fn v(a: u64, b: u64, c: u64) -> Version {
    Version::new(a, b, c)
}

/// K8: a contract version is `semver::Version`: three parts only, no leading zero; `0.2.0 < 0.10.0 < 1.0.0`.
#[test]
fn k8_version_is_three_parts() {
    assert_eq!(Version::parse("0.2.0").unwrap(), v(0, 2, 0));
    for bad in ["0.2", "v0.2.0", "02.1.0", "0.", "", "-1.0.0", "1.x.0"] {
        assert!(Version::parse(bad).is_err(), "{bad}");
    }
    assert!(v(0, 2, 0) < v(0, 10, 0) && v(0, 10, 0) < v(1, 0, 0));
}

/// K9: `tasks@>=0.2.0, <0.3.0` holds 0.2.x only; display is the text; bad texts are `range-invalid{text}`.
#[test]
fn k9_range() {
    let r = ContractRange::parse("tasks@>=0.2.0, <0.3.0").unwrap();
    assert!(r.contains(&v(0, 2, 0)) && r.contains(&v(0, 2, 7)));
    assert!(!r.contains(&v(0, 3, 0)) && !r.contains(&v(0, 1, 9)));
    assert_eq!(r.to_string(), "tasks@>=0.2.0, <0.3.0");
    assert_eq!(r.capability(), "tasks");
    for bad in [
        "tasks >=0.2 <0.3",     // the old spelling with spaces
        "tasks@0.2",            // not a full need
        "tasks@>=0.2.0 <0.3.0", // no comma
        "tasks@*",
        "tasks@^1.0.0-rc.1",
        "Tasks@>=0.2.0, <0.3.0",
        "tasks",
        "ta.sks@^1.0.0",
        "tasks@>=0.2",
    ] {
        let e = ContractRange::parse(bad).unwrap_err();
        assert_eq!(e.code(), AcceptsCode::RangeInvalid, "{bad}");
        assert!(e.params().any(|(k, _)| k == "text"), "{bad}");
    }
}

/// V-3: the rule is Cargo's caret. For 0.x a minor step breaks: `^0.2.0` holds 0.2.5, not 0.3.0 (the old "main equal, minor
/// at least" would let 0.3 through); for 1.x it is "main equal, minor at least".
#[test]
fn v3_caret_is_the_rule_for_zero_x() {
    let r = ContractRange::parse("tasks@^0.2.0").unwrap();
    assert!(r.contains(&v(0, 2, 5)));
    assert!(!r.contains(&v(0, 3, 0)), "0.3 breaks 0.2");
    let one = ContractRange::parse("tasks@^1.2.0").unwrap();
    assert!(one.contains(&v(1, 9, 0)) && !one.contains(&v(2, 0, 0)) && !one.contains(&v(1, 1, 9)));
    // the generator's form for the 0.0.x line
    let line = ContractRange::parse("tasks@>=0.0.1, <0.1.0").unwrap();
    assert!(
        line.contains(&v(0, 0, 9)) && !line.contains(&v(0, 1, 0)) && !line.contains(&v(0, 0, 0))
    );
}

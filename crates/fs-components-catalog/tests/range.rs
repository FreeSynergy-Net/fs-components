//! K8, K9 of ENTWURF section 5: versions and contract ranges.
use fs_components_catalog::{AcceptsCode, ContractRange, Version2};

fn v(a: u32, b: u32) -> Version2 {
    Version2 { major: a, minor: b }
}

/// K8: `Version2` is `major.minor` without a leading zero; `0.2 < 0.10 < 1.0`.
#[test]
fn k8_version2() {
    assert_eq!(Version2::parse("0.2"), Some(v(0, 2)));
    assert_eq!(Version2::parse("10.0"), Some(v(10, 0)));
    for bad in ["v0.2", "0.2.1", "02.1", "0.", "", "-1.0", "1.x"] {
        assert_eq!(Version2::parse(bad), None, "{bad}");
    }
    assert!(v(0, 2) < v(0, 10) && v(0, 10) < v(1, 0));
}

/// K9: `tasks >=0.2 <0.3` holds 0.2 only; display is the text; bad texts are `range-invalid{text}`.
#[test]
fn k9_range() {
    let r = ContractRange::parse("tasks >=0.2 <0.3").unwrap();
    assert!(r.contains(v(0, 2)) && !r.contains(v(0, 3)) && !r.contains(v(0, 1)));
    assert_eq!(r.to_string(), "tasks >=0.2 <0.3");
    assert_eq!(r.capability(), "tasks");
    for bad in [
        "tasks 0.2",
        "tasks >=0.3 <0.2",
        "Tasks >=0.2 <0.3",
        "tasks  >=0.2 <0.3",
        "tasks >0.2 <0.3",
        "tasks >=0.2 <=0.3",
    ] {
        let e = ContractRange::parse(bad).unwrap_err();
        assert_eq!(e.code(), AcceptsCode::RangeInvalid, "{bad}");
        assert!(e.params().any(|(k, _)| k == "text"));
    }
}

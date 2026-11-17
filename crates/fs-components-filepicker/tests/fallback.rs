//! The fallback chain: no dialog gives the marked path field, a refusal and a cancel do not, and the session bus is only looked at.
//!
//! Pattern: Model tests with the fake as primary (error injection stands for "no portal": no connection to any bus is made).
//! Mutants: `portal-denied` also falls back; a cancel falls back; `portal-timeout` does not fall back; the marker not set after a
//! fallback; `session_bus_present` true without the variable or the socket.

use std::path::Path;
use std::sync::Arc;

use fs_components_filepicker::{
    Answer, ChooserError, FakeChooser, FallbackChooser, FileChooser, Labels, Mode, OpenRequest,
    ProblemCode, session_bus_present,
};

fn request() -> OpenRequest {
    OpenRequest::new(Mode::OpenFile, &Labels::default())
}

fn chain(primary: Answer) -> (FallbackChooser, FakeChooser, FakeChooser) {
    let first = FakeChooser::new().answer(primary);
    let field = FakeChooser::new()
        .provisional(true)
        .answer(Answer::Paths(vec!["/typed".into()]));
    (
        FallbackChooser::new(Arc::new(first.clone()), Arc::new(field.clone())),
        first,
        field,
    )
}

#[test]
fn no_portal_goes_to_the_path_field_and_is_marked() {
    for code in [ProblemCode::PortalUnavailable, ProblemCode::PortalTimeout] {
        let (c, first, field) = chain(Answer::Fail(ChooserError::problem(code)));
        assert!(!c.is_provisional());
        let picked = c.open(request()).unwrap();
        assert_eq!(
            picked.first(),
            &std::path::PathBuf::from("/typed"),
            "{code:?}"
        );
        assert_eq!(first.requests().len(), 1);
        assert_eq!(field.requests().len(), 1);
        assert!(c.used_fallback());
        assert!(c.is_provisional());
    }
}

#[test]
fn a_working_portal_is_not_replaced() {
    let (c, _first, field) = chain(Answer::Paths(vec!["/real".into()]));
    assert_eq!(
        c.open(request()).unwrap().first(),
        &std::path::PathBuf::from("/real")
    );
    assert!(field.requests().is_empty());
    assert!(!c.used_fallback());
    assert!(!c.is_provisional());
}

#[test]
fn a_cancel_and_a_refusal_do_not_fall_back() {
    let (c, _f, field) = chain(Answer::Cancel);
    assert_eq!(c.open(request()), Err(ChooserError::Cancelled));
    assert!(field.requests().is_empty());
    let (c, _f, field) = chain(Answer::Fail(ChooserError::problem(
        ProblemCode::PortalDenied,
    )));
    assert_eq!(
        c.open(request()).unwrap_err(),
        ChooserError::problem(ProblemCode::PortalDenied)
    );
    assert!(
        field.requests().is_empty(),
        "a refusal is shown, never worked around"
    );
    assert!(!c.used_fallback());
}

fn no_env(_: &str) -> Option<String> {
    None
}

#[test]
fn the_session_bus_is_only_looked_at() {
    assert!(
        !session_bus_present(&no_env, &|_| true),
        "no variable at all: no bus, whatever exists"
    );
    let address = |k: &str| (k == "DBUS_SESSION_BUS_ADDRESS").then(|| "unix:path=/x".to_owned());
    assert!(session_bus_present(&address, &|_| false));
    let blank = |k: &str| (k == "DBUS_SESSION_BUS_ADDRESS").then(|| "  ".to_owned());
    assert!(!session_bus_present(&blank, &|_| false));
    let runtime = |k: &str| (k == "XDG_RUNTIME_DIR").then(|| "/run/user/1".to_owned());
    assert!(session_bus_present(&runtime, &|p| p == Path::new("/run/user/1/bus")));
    assert!(!session_bus_present(&runtime, &|_| false));
}

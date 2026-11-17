//! The portal adapter without a portal: the mapping of its errors, the limit of the start, and the real "no bus" case in a child
//! process whose environment names a bus that is not there (so no session of anybody is touched).
//!
//! Pattern: Model tests over `map_error` + a child-process test (the same test binary, cleared environment, a bogus bus address).
//! Mutants: a cancel mapped to a problem; a refusal mapped to `portal-unavailable`; an IO error mapped to a cancel; the start
//! limit changed from 5 s; the child ends green without having run its check (the test checks the marker line).
#![cfg(all(feature = "portal", target_os = "linux"))]

use std::process::Command;
use std::time::Duration;

use ashpd::desktop::ResponseError;
use ashpd::{Error, PortalError};
use fs_components_filepicker::portal::{START_LIMIT, map_error};
use fs_components_filepicker::{
    ChooserError, FakeChooser, FallbackChooser, FileChooser, Labels, Mode, OpenRequest,
    PortalChooser, ProblemCode,
};

fn code_of(error: &Error) -> Option<ProblemCode> {
    match map_error(error) {
        ChooserError::Problem(p) => Some(p.code),
        ChooserError::Cancelled => None,
    }
}

#[test]
fn a_cancel_of_the_person_is_no_problem() {
    assert_eq!(
        map_error(&Error::Response(ResponseError::Cancelled)),
        ChooserError::Cancelled
    );
    assert_eq!(
        map_error(&Error::Portal(PortalError::Cancelled("x".into()))),
        ChooserError::Cancelled
    );
}

#[test]
fn refusals_are_denials_not_missing_dialogs() {
    assert_eq!(
        code_of(&Error::Portal(PortalError::NotAllowed("no".into()))),
        Some(ProblemCode::PortalDenied)
    );
    assert_eq!(
        code_of(&Error::Portal(PortalError::InvalidArgument("no".into()))),
        Some(ProblemCode::PortalDenied)
    );
    assert_eq!(
        code_of(&Error::Response(ResponseError::Other)),
        Some(ProblemCode::PortalDenied)
    );
    match map_error(&Error::Portal(PortalError::NotAllowed("why".into()))) {
        ChooserError::Problem(p) => assert!(p.param("detail").is_some_and(|d| d.contains("why"))),
        ChooserError::Cancelled => panic!(),
    }
}

#[test]
fn no_bus_no_portal_no_answer_are_missing_dialogs() {
    assert_eq!(
        code_of(&Error::NoResponse),
        Some(ProblemCode::PortalUnavailable)
    );
}

fn dbus(error: ashpd::zbus::fdo::Error) -> Error {
    Error::Zbus(ashpd::zbus::Error::FDO(Box::new(error)))
}

#[test]
fn a_dbus_refusal_is_a_denial_never_a_missing_dialog() {
    use ashpd::zbus::fdo::Error as F;
    for error in [
        dbus(F::AccessDenied("no".into())),
        dbus(F::AuthFailed("no".into())),
        Error::Portal(PortalError::ZBus(ashpd::zbus::Error::FDO(Box::new(
            F::AccessDenied("no".into()),
        )))),
    ] {
        assert_eq!(code_of(&error), Some(ProblemCode::PortalDenied), "{error}");
    }
    // the chain does not open the path field for it, and the person sees the refusal
    let denied = map_error(&dbus(ashpd::zbus::fdo::Error::AccessDenied("no".into())));
    let field = FakeChooser::new().provisional(true);
    let chain = FallbackChooser::new(
        std::sync::Arc::new(
            FakeChooser::new().answer(fs_components_filepicker::Answer::Fail(denied.clone())),
        ),
        std::sync::Arc::new(field.clone()),
    );
    let request = OpenRequest::new(Mode::OpenFile, &Labels::default());
    assert_eq!(chain.open(request), Err(denied));
    assert_eq!(field.requests().len(), 0, "the path field was not asked");
    assert!(!chain.is_provisional());
}

#[test]
fn only_a_missing_service_or_bus_is_a_missing_dialog() {
    use ashpd::zbus::fdo::Error as F;
    for error in [
        dbus(F::ServiceUnknown("x".into())),
        dbus(F::NameHasNoOwner("x".into())),
        dbus(F::NoReply("x".into())),
        dbus(F::Timeout("x".into())),
        dbus(F::NoServer("x".into())),
        dbus(F::UnknownMethod("x".into())),
        Error::Zbus(ashpd::zbus::Error::Address("x".into())),
        Error::Zbus(ashpd::zbus::Error::InterfaceNotFound),
        Error::PortalNotFound("org.freedesktop.portal.FileChooser".try_into().unwrap()),
    ] {
        assert_eq!(
            code_of(&error),
            Some(ProblemCode::PortalUnavailable),
            "{error}"
        );
    }
}

#[test]
fn every_other_error_is_shown_and_never_falls_back() {
    use ashpd::zbus::fdo::Error as F;
    for error in [
        dbus(F::Failed("x".into())),
        dbus(F::NoMemory("x".into())),
        dbus(F::InvalidArgs("x".into())),
        Error::Zbus(ashpd::zbus::Error::Unsupported),
        Error::Portal(PortalError::NotFound("x".into())),
        Error::Portal(PortalError::Exist("x".into())),
        Error::Portal(PortalError::WindowDestroyed("x".into())),
        Error::InvalidAppID,
        Error::ParseError("x"),
        Error::Portal(PortalError::Failed("x".into())),
    ] {
        match map_error(&error) {
            ChooserError::Problem(p) => {
                assert_eq!(p.code, ProblemCode::PortalFailed, "{error}");
                assert!(!p.code.means_no_dialog());
                assert!(p.param("detail").is_some(), "the text of the error stays");
            }
            ChooserError::Cancelled => panic!("{error} must not be a cancel"),
        }
    }
}

#[test]
fn file_trouble_is_io() {
    let e = Error::IO(std::io::Error::other("disk"));
    assert_eq!(code_of(&e), Some(ProblemCode::Io));
}

#[test]
fn the_start_of_the_portal_may_take_five_seconds() {
    assert_eq!(START_LIMIT, Duration::from_secs(5));
}

const MARKER: &str = "FS_FP_CHILD_CHECKED";

/// The body of the child: runs only when the parent started this binary with the marker variable.
#[test]
fn child_without_a_bus() {
    if std::env::var_os("FS_FP_CHILD").is_none() {
        return;
    }
    let request = OpenRequest::new(Mode::OpenFile, &Labels::default());
    let error = PortalChooser::new().open(request.clone()).unwrap_err();
    match &error {
        ChooserError::Problem(p) if p.code.means_no_dialog() => {}
        other => panic!("no bus must mean no dialog, got {other:?}"),
    }
    // and the chain goes on to the path field (a fake here)
    let field = FakeChooser::new()
        .provisional(true)
        .answer(fs_components_filepicker::Answer::Cancel);
    let chain = FallbackChooser::new(
        std::sync::Arc::new(PortalChooser::new()),
        std::sync::Arc::new(field.clone()),
    );
    assert_eq!(chain.open(request), Err(ChooserError::Cancelled));
    assert_eq!(
        field.requests().len(),
        1,
        "the fallback must have been asked"
    );
    assert!(chain.is_provisional());
    println!("{MARKER}");
}

#[test]
fn without_a_bus_the_portal_says_so_and_the_chain_falls_back() {
    let out = Command::new(std::env::current_exe().unwrap())
        .args([
            "child_without_a_bus",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env_clear()
        .env("FS_FP_CHILD", "1")
        // a bus that is not there: no session is touched, and zbus has nothing else to try
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/nonexistent-fs-filepicker-test/bus",
        )
        .output()
        .expect("the child must start");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "child failed: {stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains(MARKER),
        "the child did not run its check: {stdout}"
    );
}

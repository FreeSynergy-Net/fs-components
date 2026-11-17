//! The chooser is never called on the caller's thread: a worker thread, the answer by callback, a gate that proves the caller
//! does not wait.
//!
//! Pattern: Model tests over the worker with the fake chooser (thread ids, a closed gate). Mutants: `open` called on the calling
//! thread (the test with the closed gate then blocks and the thread ids are equal); the answer dropped; a panic not delivered; the
//! button waits for the dialog; a second click starts a second dialog while one is open.

use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use fs_components_filepicker::ui::{FilePickButton, Phase};
use fs_components_filepicker::worker::pick_in_worker;
use fs_components_filepicker::{
    Answer, ChooserError, FakeChooser, FileChooser, Gate, Labels, Mode, OpenRequest, Picked,
    ProblemCode,
};

const WAIT: Duration = Duration::from_secs(10);

fn request() -> OpenRequest {
    OpenRequest::new(Mode::OpenFile, &Labels::default())
}

#[test]
fn the_call_returns_while_the_dialog_is_still_open() {
    let gate = Gate::default();
    let fake = FakeChooser::new()
        .gated(&gate)
        .answer(Answer::Paths(vec!["/a".into()]));
    let (tx, rx) = channel();
    let handle = pick_in_worker(Arc::new(fake.clone()), request(), move |a| {
        tx.send(a).unwrap();
    });
    // we are here although the fake waits at the closed gate: the caller did not wait for the dialog
    assert!(matches!(
        rx.recv_timeout(Duration::from_millis(100)),
        Err(RecvTimeoutError::Timeout)
    ));
    gate.open();
    let answer = rx.recv_timeout(WAIT).expect("the answer must arrive");
    assert_eq!(answer.unwrap().first(), &std::path::PathBuf::from("/a"));
    handle.unwrap().join().unwrap();
}

#[test]
fn the_chooser_runs_on_another_thread_than_the_caller() {
    let fake = FakeChooser::new().answer(Answer::Cancel);
    let (tx, rx) = channel();
    pick_in_worker(Arc::new(fake.clone()), request(), move |a| {
        tx.send(a).unwrap();
    });
    assert_eq!(rx.recv_timeout(WAIT).unwrap(), Err(ChooserError::Cancelled));
    let threads = fake.threads();
    assert_eq!(threads.len(), 1);
    assert_ne!(threads[0], std::thread::current().id());
}

struct Panics;
impl FileChooser for Panics {
    fn open(&self, _: OpenRequest) -> Result<Picked, ChooserError> {
        panic!("boom");
    }
}

#[test]
fn a_panic_of_the_chooser_is_delivered_as_a_problem() {
    let (tx, rx) = channel();
    pick_in_worker(Arc::new(Panics), request(), move |a| tx.send(a).unwrap());
    match rx.recv_timeout(WAIT).unwrap() {
        Err(ChooserError::Problem(p)) => assert_eq!(p.code, ProblemCode::Io),
        other => panic!("expected a problem, got {other:?}"),
    }
}

fn wait_for(button: &FilePickButton, done: impl Fn(&Phase) -> bool) -> Phase {
    let end = std::time::Instant::now() + WAIT;
    loop {
        let phase = button.phase();
        if done(&phase) {
            return phase;
        }
        assert!(std::time::Instant::now() < end, "timeout, phase {phase:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn the_button_starts_the_dialog_at_once_and_shows_the_choice() {
    let gate = Gate::default();
    let fake = FakeChooser::new()
        .gated(&gate)
        .answer(Answer::Paths(vec!["/x/y.vcf".into()]));
    let seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let button = FilePickButton::new(&Labels::default(), Arc::new(fake), request())
        .on_picked(move |p| log.lock().unwrap().push(p.paths().to_vec()));
    assert_eq!(button.phase(), Phase::Idle);
    assert!(button.activate());
    assert_eq!(button.phase(), Phase::Waiting);
    assert!(
        !button.activate(),
        "a second click while the dialog is open must not start another"
    );
    gate.open();
    let phase = wait_for(&button, |p| matches!(p, Phase::Chosen(_)));
    assert_eq!(phase, Phase::Chosen(vec!["/x/y.vcf".into()]));
    assert_eq!(
        *seen.lock().unwrap(),
        vec![vec![std::path::PathBuf::from("/x/y.vcf")]]
    );
}

#[test]
fn a_cancel_leaves_the_button_as_it_was_and_a_problem_is_shown() {
    let fake = FakeChooser::new()
        .answer(Answer::Paths(vec!["/first".into()]))
        .answer(Answer::Cancel)
        .answer(Answer::Fail(ChooserError::problem(
            ProblemCode::PortalDenied,
        )));
    let button = FilePickButton::new(&Labels::default(), Arc::new(fake), request());
    assert!(button.activate());
    wait_for(&button, |p| matches!(p, Phase::Chosen(_)));
    assert!(button.activate());
    let after_cancel = wait_for(&button, |p| *p != Phase::Waiting);
    assert_eq!(
        after_cancel,
        Phase::Chosen(vec!["/first".into()]),
        "a cancel changes nothing"
    );
    assert!(button.activate());
    match wait_for(&button, |p| *p != Phase::Waiting) {
        Phase::Failed(p) => assert_eq!(p.code, ProblemCode::PortalDenied),
        other => panic!("a problem must be shown, got {other:?}"),
    }
}

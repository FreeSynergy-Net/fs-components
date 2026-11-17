//! The fallback path field: a typed path is looked at (exists, right kind, readable), a refused path stays open with its problem,
//! the chooser is marked provisional, cancel reaches the waiting caller.
//!
//! Pattern: Model tests over `validate_path`, `PathPrompt` and `PathFieldChooser` on a scratch folder (no window). Mutants: a missing
//! file accepted; a folder accepted as file or the other way round; an unreadable file accepted (skipped when running as root);
//! a refused path closes the question; `submit` without a question succeeds; the chooser not provisional; cancel answers with a path.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use fs_components_filepicker::{
    ChooserError, FileChooser, Labels, Mode, OpenRequest, PathFieldChooser, PathPrompt,
    ProblemCode, validate_path,
};

/// A scratch folder that removes itself (exactly its own path, no pattern).
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("fs-filepicker-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn code(result: Result<PathBuf, fs_components_filepicker::Problem>) -> ProblemCode {
    result.unwrap_err().code
}

#[test]
fn an_existing_readable_file_is_taken() {
    let s = Scratch::new("file");
    let f = s.path().join("a.vcf");
    fs::write(&f, "x").unwrap();
    assert_eq!(
        validate_path(&format!("  {}  ", f.display()), Mode::OpenFile).unwrap(),
        f
    );
    assert_eq!(
        validate_path(&f.display().to_string(), Mode::OpenFiles).unwrap(),
        f
    );
}

#[test]
fn a_missing_path_names_itself_in_the_problem() {
    let s = Scratch::new("missing");
    let f = s.path().join("nope");
    let p = validate_path(&f.display().to_string(), Mode::OpenFile).unwrap_err();
    assert_eq!(p.code, ProblemCode::PathMissing);
    assert_eq!(p.param("path"), Some(f.display().to_string().as_str()));
    assert_eq!(
        code(validate_path("   ", Mode::OpenFile)),
        ProblemCode::PathMissing
    );
}

#[test]
fn the_kind_must_fit_the_mode() {
    let s = Scratch::new("kind");
    let f = s.path().join("a");
    fs::write(&f, "x").unwrap();
    assert_eq!(
        code(validate_path(
            &s.path().display().to_string(),
            Mode::OpenFile
        )),
        ProblemCode::WrongKind
    );
    assert_eq!(
        code(validate_path(&f.display().to_string(), Mode::Folder)),
        ProblemCode::WrongKind
    );
    assert_eq!(
        code(validate_path(
            &s.path().display().to_string(),
            Mode::SaveFile
        )),
        ProblemCode::WrongKind
    );
    assert_eq!(
        validate_path(&s.path().display().to_string(), Mode::Folder).unwrap(),
        s.path()
    );
}

#[test]
fn saving_needs_an_existing_folder_but_not_an_existing_file() {
    let s = Scratch::new("save");
    let target = s.path().join("new.vcf");
    assert_eq!(
        validate_path(&target.display().to_string(), Mode::SaveFile).unwrap(),
        target
    );
    let in_missing = s.path().join("no-dir").join("x");
    assert_eq!(
        code(validate_path(
            &in_missing.display().to_string(),
            Mode::SaveFile
        )),
        ProblemCode::PathMissing
    );
}

#[cfg(unix)]
#[test]
fn an_unreadable_file_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let s = Scratch::new("unreadable");
    let f = s.path().join("secret");
    fs::write(&f, "x").unwrap();
    fs::set_permissions(&f, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::File::open(&f).is_ok() {
        return; // running as root: the file system lets us read it, so the rule cannot be shown here
    }
    assert_eq!(
        code(validate_path(&f.display().to_string(), Mode::OpenFile)),
        ProblemCode::PathUnreadable
    );
}

fn ask(
    chooser: &PathFieldChooser,
) -> std::sync::mpsc::Receiver<Result<fs_components_filepicker::Picked, ChooserError>> {
    let (tx, rx) = channel();
    let c = chooser.clone();
    std::thread::spawn(move || {
        tx.send(c.open(OpenRequest::new(Mode::OpenFile, &Labels::default())))
            .unwrap();
    });
    let end = std::time::Instant::now() + Duration::from_secs(10);
    while chooser.prompt().pending().is_none() {
        assert!(
            std::time::Instant::now() < end,
            "the question never arrived"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    rx
}

#[test]
fn the_question_waits_in_the_prompt_and_the_answer_reaches_the_caller() {
    let s = Scratch::new("ask");
    let f = s.path().join("a");
    fs::write(&f, "x").unwrap();
    let prompt = PathPrompt::new();
    let chooser = PathFieldChooser::new(&prompt);
    let rx = ask(&chooser);
    assert_eq!(prompt.pending().unwrap().mode, Mode::OpenFile);
    prompt.submit(&f.display().to_string()).unwrap();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap()
            .first(),
        &f
    );
    assert!(prompt.pending().is_none());
}

#[test]
fn a_refused_path_keeps_the_question_open_with_its_problem() {
    let s = Scratch::new("refused");
    let f = s.path().join("a");
    fs::write(&f, "x").unwrap();
    let prompt = PathPrompt::new();
    let chooser = PathFieldChooser::new(&prompt);
    let rx = ask(&chooser);
    let problem = prompt.submit("/definitely/not/here").unwrap_err();
    assert_eq!(problem.code, ProblemCode::PathMissing);
    assert_eq!(prompt.problem(), Some(problem));
    assert!(
        prompt.pending().is_some(),
        "the person may type another path"
    );
    prompt.submit(&f.display().to_string()).unwrap();
    assert!(rx.recv_timeout(Duration::from_secs(10)).unwrap().is_ok());
    assert!(prompt.problem().is_none());
}

#[test]
fn cancel_reaches_the_caller_as_a_cancel() {
    let prompt = PathPrompt::new();
    let chooser = PathFieldChooser::new(&prompt);
    let rx = ask(&chooser);
    prompt.cancel();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(10)).unwrap(),
        Err(ChooserError::Cancelled)
    );
}

#[test]
fn a_new_question_cancels_the_old_one() {
    let prompt = PathPrompt::new();
    let chooser = PathFieldChooser::new(&prompt);
    let first = ask(&chooser);
    let second = {
        let (tx, rx) = channel();
        let c = chooser.clone();
        std::thread::spawn(move || {
            tx.send(c.open(OpenRequest::new(Mode::Folder, &Labels::default())))
                .unwrap()
        });
        rx
    };
    assert_eq!(
        first.recv_timeout(Duration::from_secs(10)).unwrap(),
        Err(ChooserError::Cancelled)
    );
    prompt.cancel();
    assert_eq!(
        second.recv_timeout(Duration::from_secs(10)).unwrap(),
        Err(ChooserError::Cancelled)
    );
}

#[test]
fn nothing_is_taken_while_no_question_is_open() {
    let prompt = PathPrompt::new();
    assert_eq!(prompt.submit("/tmp").unwrap_err().code, ProblemCode::Io);
}

#[test]
fn the_path_field_says_it_is_provisional_and_notifies_the_view() {
    let prompt = PathPrompt::new();
    assert!(PathFieldChooser::new(&prompt).is_provisional());
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let seen = std::sync::Arc::clone(&count);
    prompt.on_change(move || {
        seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    let rx = ask(&PathFieldChooser::new(&prompt));
    prompt.cancel();
    let _ = rx.recv_timeout(Duration::from_secs(10));
    assert!(
        count.load(std::sync::atomic::Ordering::SeqCst) >= 2,
        "parked and cancelled"
    );
}

//! The fake chooser and the request: paths, several files, cancel, errors that stay visible.
//!
//! Pattern: Model tests over the fake (no dialog, no window). Mutants (each must turn a test here red): `Answer::Cancel` mapped
//! to a problem; an empty script answered with `Cancelled` instead of a problem; `Picked` kept empty; the title of `OpenRequest::new`
//! taken from the wrong mode; `multiple` true for `OpenFile`.

use std::path::PathBuf;

use fs_components_filepicker::{
    Answer, ChooserError, FakeChooser, FileChooser, Filter, Labels, Mode, OpenRequest, ProblemCode,
};
use fs_components_words::{Locale, Localized};

fn request(mode: Mode) -> OpenRequest {
    OpenRequest::new(mode, &Labels::default())
}

#[test]
fn one_path_comes_back() {
    let fake = FakeChooser::new().answer(Answer::Paths(vec![PathBuf::from("/a/b.vcf")]));
    let picked = fake.open(request(Mode::OpenFile)).unwrap();
    assert_eq!(picked.paths(), [PathBuf::from("/a/b.vcf")]);
    assert_eq!(picked.first(), &PathBuf::from("/a/b.vcf"));
}

#[test]
fn several_paths_keep_their_order() {
    let paths = vec![PathBuf::from("/x/2"), PathBuf::from("/x/1"), PathBuf::from("/x/3")];
    let fake = FakeChooser::new().answer(Answer::Paths(paths.clone()));
    let picked = fake.open(request(Mode::OpenFiles)).unwrap();
    assert_eq!(picked.paths(), paths.as_slice());
}

#[test]
fn a_cancel_is_a_cancel_and_no_problem() {
    let fake = FakeChooser::new().answer(Answer::Cancel);
    let error = fake.open(request(Mode::OpenFile)).unwrap_err();
    assert!(error.is_cancelled());
    assert_eq!(error, ChooserError::Cancelled);
}

#[test]
fn an_error_is_passed_on_with_code_and_parameters() {
    let problem = fs_components_filepicker::Problem::new(ProblemCode::PortalTimeout).with("limit-ms", "5000");
    let fake = FakeChooser::new().answer(Answer::Fail(problem.clone().into()));
    match fake.open(request(Mode::Folder)).unwrap_err() {
        ChooserError::Problem(p) => {
            assert_eq!(p.code, ProblemCode::PortalTimeout);
            assert_eq!(p.param("limit-ms"), Some("5000"));
            assert_eq!(p.to_string(), "portal-timeout limit-ms=5000");
        }
        ChooserError::Cancelled => panic!("a problem must not look like a cancel"),
    }
}

#[test]
fn an_empty_script_is_a_problem_never_a_silent_cancel() {
    let error = FakeChooser::new().open(request(Mode::OpenFile)).unwrap_err();
    assert!(!error.is_cancelled());
    let empty = FakeChooser::new().answer(Answer::Paths(Vec::new()));
    assert!(!empty.open(request(Mode::OpenFile)).unwrap_err().is_cancelled());
}

#[test]
fn the_fake_remembers_what_it_was_asked() {
    let fake = FakeChooser::new().answer(Answer::Cancel);
    let asked = request(Mode::SaveFile)
        .filter(Filter::new("Contacts").pattern("*.vcf").media_type("text/vcard"))
        .initial_dir("/home/x")
        .suggested_name("contacts.vcf");
    let _ = fake.open(asked.clone());
    assert_eq!(fake.requests(), vec![asked]);
}

#[test]
fn the_standard_title_follows_the_mode_and_the_language() {
    let en = Labels::default();
    assert_eq!(request(Mode::OpenFile).title, en.title_open_file);
    assert_eq!(request(Mode::OpenFiles).title, en.title_open_files);
    assert_eq!(request(Mode::SaveFile).title, en.title_save_file);
    assert_eq!(request(Mode::Folder).title, en.title_folder);
    assert_ne!(en.title_open_file, en.title_folder);
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(OpenRequest::new(Mode::Folder, &de).title, "Ordner wählen");
    assert_eq!(request(Mode::Folder).title("Mein Titel").title, "Mein Titel");
}

#[test]
fn only_the_several_files_mode_is_multiple() {
    assert!(request(Mode::OpenFiles).multiple());
    for mode in [Mode::OpenFile, Mode::SaveFile, Mode::Folder] {
        assert!(!request(mode).multiple(), "{mode:?}");
    }
}

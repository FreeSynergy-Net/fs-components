//! The check of a typed path for the path field: exists, right kind, readable.
//!
//! Pattern: Specification (one pure function over the file system; each rule has a problem code). A path that is typed is not
//! trusted: it is looked at, and a problem is a code with the path as parameter.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::problem::{Problem, ProblemCode};
use crate::request::Mode;

fn missing(path: &Path) -> Problem {
    Problem::new(ProblemCode::PathMissing).with("path", path.display().to_string())
}

fn from_io(path: &Path, error: &std::io::Error) -> Problem {
    let shown = path.display().to_string();
    match error.kind() {
        ErrorKind::NotFound => missing(path),
        ErrorKind::PermissionDenied => Problem::new(ProblemCode::PathUnreadable).with("path", shown),
        _ => Problem::new(ProblemCode::Io)
            .with("path", shown)
            .with("detail", error.to_string()),
    }
}

fn wrong_kind(path: &Path) -> Problem {
    Problem::new(ProblemCode::WrongKind).with("path", path.display().to_string())
}

/// The path of `text` for `mode`, or the problem. Blanks around the text are ignored; an empty text is `path-missing`.
///
/// # Errors
/// `path-missing`, `path-unreadable`, `wrong-kind` or `io`, with the path as parameter.
pub fn validate_path(text: &str, mode: Mode) -> Result<PathBuf, Problem> {
    let path = PathBuf::from(text.trim());
    if path.as_os_str().is_empty() {
        return Err(missing(&path));
    }
    match mode {
        Mode::SaveFile => {
            if path.is_dir() {
                return Err(wrong_kind(&path));
            }
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let meta = fs::metadata(parent).map_err(|e| from_io(parent, &e))?;
            if !meta.is_dir() {
                return Err(wrong_kind(parent));
            }
            Ok(path)
        }
        Mode::Folder => {
            let meta = fs::metadata(&path).map_err(|e| from_io(&path, &e))?;
            if !meta.is_dir() {
                return Err(wrong_kind(&path));
            }
            fs::read_dir(&path).map_err(|e| from_io(&path, &e))?;
            Ok(path)
        }
        _ => {
            let meta = fs::metadata(&path).map_err(|e| from_io(&path, &e))?;
            if !meta.is_file() {
                return Err(wrong_kind(&path));
            }
            fs::File::open(&path).map_err(|e| from_io(&path, &e))?;
            Ok(path)
        }
    }
}

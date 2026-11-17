//! What is asked of a file chooser and what comes back.
//!
//! Pattern: Value objects with a builder (`OpenRequest::new(..).filter(..)`). The only way to a path is [`Picked`], which the
//! implementations make from a dialog; the type has no public constructor from a string, so a path from another source
//! (a message, a file, a program argument) cannot be passed off as a choice of the person.

use std::path::PathBuf;

use crate::labels::Labels;

/// What the person is asked to choose.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// One existing file.
    OpenFile,
    /// One or more existing files.
    OpenFiles,
    /// A place and a name for a file that is written (it may not exist yet).
    SaveFile,
    /// One folder.
    Folder,
}

/// A kind of file the dialog offers: a name for the person and the kinds, as patterns (`*.vcf`) and/or media types
/// (`text/vcard`). A filter with neither is dropped by the translation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    /// The name shown in the dialog ("Contacts"); already in the person's language.
    pub name: String,
    /// Glob patterns (`*.vcf`).
    pub patterns: Vec<String>,
    /// Media types (`text/vcard`, `application/jscontact+json`).
    pub media_types: Vec<String>,
}

impl Filter {
    /// A filter named `name` with no kinds yet.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            patterns: Vec::new(),
            media_types: Vec::new(),
        }
    }

    /// Add a glob pattern.
    #[must_use]
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.patterns.push(pattern.to_owned());
        self
    }

    /// Add a media type.
    #[must_use]
    pub fn media_type(mut self, media_type: &str) -> Self {
        self.media_types.push(media_type.to_owned());
        self
    }
}

/// One request to a chooser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenRequest {
    /// The title of the dialog, in the person's language. [`OpenRequest::new`] takes the standard one from the MF2 texts of
    /// this crate (`filepicker-title-*`); a caller with a better one resolves its own key and calls [`OpenRequest::title`].
    pub title: String,
    /// What is chosen.
    pub mode: Mode,
    /// The kinds of file offered (not used for [`Mode::Folder`]).
    pub filters: Vec<Filter>,
    /// The folder the dialog starts in, if any.
    pub initial_dir: Option<PathBuf>,
    /// The name suggested for [`Mode::SaveFile`].
    pub suggested_name: Option<String>,
}

impl OpenRequest {
    /// A request for `mode` with the standard title of `labels`.
    pub fn new(mode: Mode, labels: &Labels) -> Self {
        Self {
            title: labels.title(mode).to_owned(),
            mode,
            filters: Vec::new(),
            initial_dir: None,
            suggested_name: None,
        }
    }

    /// Several files may be chosen.
    pub fn multiple(&self) -> bool {
        self.mode == Mode::OpenFiles
    }

    /// Another title (resolved by the caller).
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        title.clone_into(&mut self.title);
        self
    }

    /// Offer one more kind of file.
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Start in `dir`.
    #[must_use]
    pub fn initial_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.initial_dir = Some(dir.into());
        self
    }

    /// Suggest `name` when saving.
    #[must_use]
    pub fn suggested_name(mut self, name: &str) -> Self {
        self.suggested_name = Some(name.to_owned());
        self
    }
}

/// What the person chose: one path, or several for [`Mode::OpenFiles`]. Never empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picked {
    paths: Vec<PathBuf>,
}

impl Picked {
    /// Made by the implementations of the chooser only (a dialog, the path field, the fake of a test).
    pub(crate) fn from_dialog(paths: Vec<PathBuf>) -> Option<Self> {
        (!paths.is_empty()).then_some(Self { paths })
    }

    /// The chosen paths, in the order of the dialog.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// The first (for the modes with one path, the only one).
    pub fn first(&self) -> &PathBuf {
        &self.paths[0]
    }
}

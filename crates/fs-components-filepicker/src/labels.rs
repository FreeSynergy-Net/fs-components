//! The words of the file chooser, in two languages.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.
//! Problems are made into a sentence here from their code and parameters, nowhere else.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

use crate::problem::Problem;
use crate::request::Mode;

/// The two language files; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
pub const TEXTS: &[Source] = &[
    Source {
        language: "en",
        text: include_str!("texts/en.mf2"),
    },
    Source {
        language: "de",
        text: include_str!("texts/de.mf2"),
    },
];

/// The words, made for one language.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Dialog title for one file.
    pub title_open_file: String,
    /// Dialog title for several files.
    pub title_open_files: String,
    /// Dialog title for saving.
    pub title_save_file: String,
    /// Dialog title for a folder.
    pub title_folder: String,
    /// Button: choose one file.
    pub button_open_file: String,
    /// Button: choose several files.
    pub button_open_files: String,
    /// Button: choose where to save.
    pub button_save_file: String,
    /// Button: choose a folder.
    pub button_folder: String,
    /// Shown while the dialog is open.
    pub waiting: String,
    /// Shown while nothing is chosen.
    pub nothing: String,
    /// The marker of the path field: it is a stand-in.
    pub provisional: String,
    /// Placeholder of the path field.
    pub path_placeholder: String,
    /// Button of the path field: take the path.
    pub ok: String,
    /// Button of the path field: leave.
    pub cancel: String,
    /// The language the words were made for (problems are formatted in it).
    pub locale: Locale,
}

impl Labels {
    /// The standard title for `mode`.
    pub fn title(&self, mode: Mode) -> &str {
        match mode {
            Mode::OpenFile => &self.title_open_file,
            Mode::OpenFiles => &self.title_open_files,
            Mode::SaveFile => &self.title_save_file,
            Mode::Folder => &self.title_folder,
        }
    }

    /// The standard button word for `mode`.
    pub fn button(&self, mode: Mode) -> &str {
        match mode {
            Mode::OpenFile => &self.button_open_file,
            Mode::OpenFiles => &self.button_open_files,
            Mode::SaveFile => &self.button_save_file,
            Mode::Folder => &self.button_folder,
        }
    }

    /// The sentence for a problem, from its code and parameters (a missing parameter shows as empty, never a panic).
    pub fn problem(&self, problem: &Problem) -> String {
        let mut args = Args::new();
        for (name, value) in &problem.params {
            args = args.text(name, value.as_str());
        }
        let id = format!("filepicker-problem-{}", problem.code.as_str());
        Mf2Texts::new(TEXTS, &self.locale)
            .say(&TextId::new(&id), &args)
            .text()
            .to_owned()
    }
}

impl Localized for Labels {
    /// The words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            title_open_file: t.plain("filepicker-title-open-file"),
            title_open_files: t.plain("filepicker-title-open-files"),
            title_save_file: t.plain("filepicker-title-save-file"),
            title_folder: t.plain("filepicker-title-folder"),
            button_open_file: t.plain("filepicker-button-open-file"),
            button_open_files: t.plain("filepicker-button-open-files"),
            button_save_file: t.plain("filepicker-button-save-file"),
            button_folder: t.plain("filepicker-button-folder"),
            waiting: t.plain("filepicker-waiting"),
            nothing: t.plain("filepicker-nothing"),
            provisional: t.plain("filepicker-provisional"),
            path_placeholder: t.plain("filepicker-path-placeholder"),
            ok: t.plain("filepicker-ok"),
            cancel: t.plain("filepicker-cancel"),
            locale: locale.clone(),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}

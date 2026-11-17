//! The codes of the settings crate, with their parameter keys. Message key: `setting-error-` and the name.
//!
//! Pattern: none, an enum with total `match`es (as fs-starter `codes.rs`). A code is never renamed or reused once
//! released; a new one is appended to the enum AND to `ALL` (K-test: each variant exactly once).

use crate::coded_stub::ErrorCode;

/// What the settings crate found wrong or must say, without a sentence.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingCode {
    /// A key, owner or group name does not have the form (parameter `key`).
    KeyInvalid,
    /// A program name does not have the form (parameter `program`).
    ProgramInvalid,
    /// A key is declared twice (parameter `key`).
    DuplicateKey,
    /// A declared default does not fit its own kind (parameter `key`).
    DefaultInvalid,
    /// A key that no block declared was asked or saved (parameter `key`).
    NotDeclared,
    /// A stored value has another type than declared (parameters `key`, `layer`).
    WrongType,
    /// A whole number outside the declared range (parameters `key`, `layer`).
    OutOfRange,
    /// A choice that is not in the declared list (parameters `key`, `layer`).
    UnknownChoice,
    /// No well-formed BCP 47 language tag (parameters `key`, `layer`).
    BadLanguage,
    /// A text longer than declared (parameters `key`, `layer`).
    TextTooLong,
    /// Changing a locked key was refused, for EVERYONE, the locker too (parameters `key`, `reason`, `by`).
    Locked,
    /// The instance holds a lock without a value that fits (a file edited by hand): the lock holds nothing, listed
    /// (parameter `key`).
    LockRefused,
    /// A program profile holds a key this program does not declare; kept, listed (parameters `key`, `layer`). A hint.
    UnknownKey,
    /// A settings file could not be read (parameters `file`, `kind`).
    FileUnreadable,
    /// A settings file is no TOML of flat keys (parameters `file`, `line` when known).
    FileBroken,
    /// A settings file is larger than 64 KiB (parameter `file`).
    FileTooLarge,
    /// The path is no regular file (a pipe, a device, a folder) (parameter `file`).
    NotRegular,
    /// A settings file could not be written; nothing changed (parameters `file`, `kind`).
    WriteFailed,
    /// No folder to keep settings in (no absolute `$XDG_*`/`$HOME`) (parameter `which`). Defaults are used.
    NoFolder,
    /// Locking, unlocking or setting an instance default without the right `can_lock` (parameter `key`).
    NoRight,
    /// A profile operation was refused (parameters `cause` = the profile code name: `name-invalid`, `exists`,
    /// `unknown`; `name`). Also: an active profile id without a name was read (`cause` = `unknown`); the base is used.
    Profile,
    /// A lock without a reason (parameter `key`).
    ReasonMissing,
}

impl ErrorCode for SettingCode {
    const PREFIX: &'static str = "setting-error-";
    const ALL: &'static [Self] = &[
        Self::KeyInvalid,
        Self::ProgramInvalid,
        Self::DuplicateKey,
        Self::DefaultInvalid,
        Self::NotDeclared,
        Self::WrongType,
        Self::OutOfRange,
        Self::UnknownChoice,
        Self::BadLanguage,
        Self::TextTooLong,
        Self::Locked,
        Self::LockRefused,
        Self::UnknownKey,
        Self::FileUnreadable,
        Self::FileBroken,
        Self::FileTooLarge,
        Self::NotRegular,
        Self::WriteFailed,
        Self::NoFolder,
        Self::NoRight,
        Self::Profile,
        Self::ReasonMissing,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::KeyInvalid => "key-invalid",
            Self::ProgramInvalid => "program-invalid",
            Self::DuplicateKey => "duplicate-key",
            Self::DefaultInvalid => "default-invalid",
            Self::NotDeclared => "not-declared",
            Self::WrongType => "wrong-type",
            Self::OutOfRange => "out-of-range",
            Self::UnknownChoice => "unknown-choice",
            Self::BadLanguage => "bad-language",
            Self::TextTooLong => "text-too-long",
            Self::Locked => "locked",
            Self::LockRefused => "lock-refused",
            Self::UnknownKey => "unknown-key",
            Self::FileUnreadable => "file-unreadable",
            Self::FileBroken => "file-broken",
            Self::FileTooLarge => "file-too-large",
            Self::NotRegular => "not-regular",
            Self::WriteFailed => "write-failed",
            Self::NoFolder => "no-folder",
            Self::NoRight => "no-right",
            Self::Profile => "profile",
            Self::ReasonMissing => "reason-missing",
        }
    }
}

/// The parameter keys of the codes.
pub mod keys {
    /// A setting key (`fs.shell.language`), or the bad name itself for `key-invalid`.
    pub const KEY: &str = "key";
    /// A program name.
    pub const PROGRAM: &str = "program";
    /// A layer token (`default`, `instance`, `system`, `person`, `program`, `named-person`, `named-program`;
    /// [`crate::Layer::as_str`]).
    pub const LAYER: &str = "layer";
    /// The reason an admin gave for a lock, as written.
    pub const REASON: &str = "reason";
    /// A file path.
    pub const FILE: &str = "file";
    /// A line number in a file.
    pub const LINE: &str = "line";
    /// The class of a system failure (`not-found`, `permission-denied`, `other`).
    pub const KIND: &str = "kind";
    /// Which folder is missing (`config`, `state`, `config-dirs`).
    pub const WHICH: &str = "which";
    /// Who locked (an actor reference of fs-access, never an AI acting as a person).
    pub const BY: &str = "by";
    /// The code name behind `profile`.
    pub const CAUSE: &str = "cause";
    /// A profile name, as the person wrote it.
    pub const NAME: &str = "name";
}

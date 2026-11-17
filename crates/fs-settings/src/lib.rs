//! `fs-settings`: settings as an interface of their own. Every building block (framework crate, module, theme, frontend,
//! program) DECLARES its settings, typed; a program reads the effective value through layers and is told LIVE when it
//! changes (no restart).
//!
//! Layers, lowest first ([`Layer`]): the block's default < the instance default (admin) < the system (desktop language,
//! light/dark: read, never written) < the person's general profile < the program's profile. A LOCK (any setting; who may
//! lock say the rights, `can_lock`) wins over every layer; while locked NOBODY changes the value, the locker included; it
//! is always visible with reason, who and when ([`Effective::lock`]); never silent.
//! NAMED PROFILES ("Arbeit", "Vortrag"; answer 16): one is active per device; it beats the base profiles as a whole and
//! inherits what it lacks ([`ProfileSet`], a reusable data kit, stand-in for `fs_types::profile`). A lock holds in every
//! profile. "Device" is no layer but a place: a key declared [`Place::Device`] keeps its profile values in the device
//! the device part of the store, all others in the profile part.
//!
//! Rules of the crate:
//! - Two ways to save, the two buttons of the user's word: [`Target::ThisProgram`] and [`Target::AllPrograms`]. There is
//!   no parameter that names another program: a program can never overwrite another program's settings.
//! - Typed, no JSON, no blob (ADR 0086): a store keeps primitive values only ([`Raw`]: yes/no, whole number, text).
//! - Nothing is silent: a value that does not fit its declaration, a refused lock, a broken file is a code with
//!   parameters in [`Settings::problems`], never a sentence (ADR 0057; the program words it in its MessageFormat 2 files),
//!   and the next lower layer is used.
//! - Secrets are no settings: there is no kind for them; keys, passwords and tokens belong to the vault.
//! - No AI path here: this crate acts for the program the person uses. Changing settings over MCP goes through fs-connect
//!   with the AI's own, marked actor (draft O9), never as the person.
//! - Whether an AI may take part at all is a RIGHT in fs-access (an invitation as a mode of fs-admission, with STOP AI
//!   and the consent of all), never a setting: no setting can switch it on, no lock can replace it, and an AI never holds
//!   `can_lock` (see [`GrantedLockRights`]).
//! - Every value is a default the person may change; nothing here is a fixed state.
//!
//! Pattern: Strategy ([`SettingsStore`], [`SystemSource`], [`OnChange`]) + Observer (live changes) + Value objects (keys, values,
//! codes) + pure function [`resolve`] (the order of the layers in one place).

#![deny(missing_docs)]
#![warn(clippy::pedantic)]

// Probe stand-in for `fs_types::coded` (replaced after the fs-types lift; a copy of fs-conformance `fault.rs` at fs-types 339aa24).
mod admin;
pub mod coded_stub;
mod codes;
mod declare;
#[cfg(feature = "xdg-files")]
mod files;
#[cfg(feature = "xdg-files")]
mod files_index;
#[cfg(feature = "xdg-files")]
mod files_write;
mod index_step;
mod layer;
mod lock;
mod memory;
mod memory_instance;
mod names;
mod pace;
mod profile;
mod profile_name;
mod resolve;
mod settings;
mod settings_profiles;
mod standard;
mod store;
mod texts;
mod value;

#[cfg(feature = "conformance")]
pub mod suite;

pub use admin::{Admin, InstanceLogEntry, InstanceStore, LockChange, Stampd, system_clock};
pub use coded_stub::Coded;
pub use codes::{SettingCode, keys};
pub use declare::{Catalog, Declaration, Declares, Place};
#[cfg(feature = "xdg-files")]
pub use files::XdgFiles;
pub use index_step::index_step_ok;
pub use layer::{Layer, Scope, Target};
pub use lock::{GrantedLockRights, Lock, LockGrant, LockRights, LockView, NoLockRights, lock_view};
pub use memory::MemoryStore;
pub use names::{GroupId, Owner, OwnerKind, ProgramName, SettingKey};
pub use pace::Pace;
pub use profile::ProfileSet;
pub use profile_name::{ProfileCode, ProfileName, ProfileRef};
pub use resolve::{Effective, Sources, resolve};
pub use settings::{Change, OnChange, Settings};
pub use standard::{FRONTEND, LANGUAGE, REFRESH, SCHEME, STYLE, ShellSettings, person_choice};
pub use store::{
    Health, IndexFn, Instance, NoSystem, SettingsStore, Snapshot, Stamp, SystemSource,
};
pub use texts::{TEXT_DE, TEXT_EN};
pub use value::{Kind, Raw, Value};

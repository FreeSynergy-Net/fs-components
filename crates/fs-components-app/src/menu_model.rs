//! The command model as menus: the adapter from `fs-components-commands` to [`Menu`] and [`MenuEntry`].
//!
//! Pattern: Adapter. A program that describes its commands and an arrangement (`fs-components-commands`) gets the title-bar menu
//! from [`Menu::from_model`]; the words come from the program's MF2 texts (ADR 0057) through [`ModelTexts`], the shortcut
//! is shown for the platform and language. Programs that build [`MenuEntry`] by hand are unchanged; this is one more way in.

use fs_components_commands::{Command, KeyNames, Platform, Resolved, TextKey, Toggle};

use fs_components_words::Locale;

use crate::menu::{Menu, MenuEntry};
use crate::standard::say;
use crate::texts::shell_texts;

/// What turns the keys of the model into what a line shows.
pub struct ModelTexts<'a> {
    /// The text of a key, in the language of the menu (from the program's MF2 texts).
    pub say: &'a dyn Fn(&TextKey) -> String,
    /// The platform the shortcuts are shown for (`Platform::current()` for the real thing).
    pub platform: Platform,
    /// The words of the modifier keys in that language.
    pub names: &'a KeyNames,
}

impl Menu {
    /// A menu from resolved items (`fs_components_commands::resolve`). A placeholder is skipped: a menu has no place for
    /// something that is no command. A submenu gets the id `fs.submenu.<position>` (its position in the tree).
    pub fn from_model(
        label: impl Into<String>,
        items: &[Resolved],
        texts: &ModelTexts<'_>,
    ) -> Self {
        Self::new(label, entries(items, "", texts, false))
    }
}

/// The lines of a context menu: like [`Menu::from_model`], but a row of symbols (`inline_icons`) stays one line with a row.
/// Item and entry keep the same position (the caller has dropped placeholders), so a press on a line names its item.
pub(crate) fn context_entries(items: &[Resolved], texts: &ModelTexts<'_>) -> Vec<MenuEntry> {
    entries(items, "", texts, true)
}

fn entries(items: &[Resolved], prefix: &str, texts: &ModelTexts<'_>, rows: bool) -> Vec<MenuEntry> {
    let mut out = Vec::new();
    for (position, item) in items.iter().enumerate() {
        match item {
            Resolved::Command(command) => out.push(command_entry(command, texts)),
            Resolved::Submenu {
                label,
                icon,
                children,
            } => {
                let path = format!("{prefix}{position}");
                let mut entry = MenuEntry::submenu(
                    format!("fs.submenu.{path}"),
                    (texts.say)(label),
                    entries(children, &format!("{path}-"), texts, rows),
                );
                entry.icon.clone_from(icon);
                out.push(entry);
            }
            Resolved::Separator => out.push(MenuEntry::separator()),
            Resolved::IconRow(row) => {
                let buttons = row.iter().map(|c| command_entry(c, texts));
                if rows {
                    out.push(MenuEntry::icon_row(
                        format!("fs.row.{prefix}{position}"),
                        buttons.collect(),
                    ));
                } else {
                    out.extend(buttons);
                }
            }
            _ => {}
        }
    }
    out
}

fn command_entry(command: &Command, texts: &ModelTexts<'_>) -> MenuEntry {
    let mut entry = MenuEntry::new(command.id.as_str(), (texts.say)(&command.label));
    entry.icon.clone_from(&command.icon);
    if let Some(first) = command.shortcuts.first() {
        entry = entry.shortcut(
            first.display(texts.platform, texts.names),
            first.aria_resolved(texts.platform),
        );
    }
    entry = match &command.toggle {
        Toggle::Check { on } => entry.check(*on),
        Toggle::Radio { on, .. } => entry.radio(*on),
        _ => entry,
    };
    match command.state.reason() {
        Some(reason) => entry.disabled_because((texts.say)(reason)),
        None => entry,
    }
}

/// The shell's own words (`shell-*` keys: the standard menus, "More options") in `locale`, as a function a program calls first in
/// the `say` of its [`ModelTexts`]: `|key| if key.as_str().starts_with("shell-") { shell(key) } else { mine(key) }`. A key the
/// shell does not know comes back as itself.
pub fn shell_words(locale: &Locale) -> impl Fn(&TextKey) -> String + use<> {
    let texts = shell_texts(locale);
    move |key| {
        let said = fs_components_words::Words::say(
            &texts,
            &key.as_str().into(),
            &fs_components_words::Args::new(),
        );
        if said.is_found() {
            said.text().to_owned()
        } else {
            key.as_str().to_owned()
        }
    }
}

/// The words of the modifier keys in `locale`, from the shell's MF2 texts (`shell-key-*`); a language without a file comes in
/// English (ADR 0057).
pub fn key_names(locale: &Locale) -> KeyNames {
    let texts = shell_texts(locale);
    KeyNames::new(
        say(&texts, "shell-key-control"),
        say(&texts, "shell-key-alt"),
        say(&texts, "shell-key-shift"),
        say(&texts, "shell-key-meta"),
    )
    .key("Delete", say(&texts, "shell-key-delete"))
    .key("Backspace", say(&texts, "shell-key-backspace"))
    .key("PageUp", say(&texts, "shell-key-pageup"))
    .key("PageDown", say(&texts, "shell-key-pagedown"))
}

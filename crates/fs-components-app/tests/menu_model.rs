//! The command model as menus: adapter, separators, marks, symbols, shortcuts, reasons; the real view, no pixels.

#[path = "menu_common/mod.rs"]
mod menu_common;
#[path = "menu_model_common/mod.rs"]
mod menu_model_common;

use fs_components_app::commands::{
    Arrangement, Command, Commands, MenuItem, Platform, Problem, resolve,
};
use fs_components_app::{Mark, Menu, MenuEntry};
use menu_common::Harness;
use menu_model_common::{app, file_menu};

#[test]
fn the_model_becomes_entries_with_all_the_new_fields() {
    let menu = file_menu(true, Platform::Linux);
    assert_eq!(menu.label, "Datei");
    let kinds: Vec<_> = menu.entries.iter().map(|e| (e.separator, e.mark)).collect();
    assert_eq!(
        kinds.len(),
        11,
        "three, separator, two radio, separator, check, separator, submenu, quit"
    );
    let new = &menu.entries[0];
    assert_eq!(new.id, "document-new");
    assert_eq!(new.label, "Neu");
    assert_eq!(new.icon.as_deref(), Some("document-new"));
    assert_eq!(new.shortcut.as_deref(), Some("Strg+N"));
    assert_eq!(
        new.shortcut_aria.as_deref(),
        Some("Control+N"),
        "aria-keyshortcuts: Primary resolved"
    );
    let save = &menu.entries[2];
    assert!(save.disabled);
    assert_eq!(save.reason.as_deref(), Some("Nichts zu speichern"));
    assert!(menu.entries[3].separator);
    assert_eq!(
        (menu.entries[4].mark, menu.entries[4].checked),
        (Mark::Radio, true)
    );
    assert_eq!(
        (menu.entries[5].mark, menu.entries[5].checked),
        (Mark::Radio, false)
    );
    let hidden = &menu.entries[7];
    assert_eq!((hidden.mark, hidden.checked), (Mark::Check, true));
    assert_eq!(hidden.shortcut.as_deref(), Some("Strg+Umschalt+H"));
    let more = &menu.entries[9];
    assert_eq!(more.id, "fs.submenu.9");
    assert_eq!(more.children.len(), 3);
    assert!(more.children[1].separator);
    assert_eq!(more.children[2].shortcut.as_deref(), Some("Entf"));
}

#[test]
fn the_shortcut_is_shown_for_the_platform() {
    let mac = file_menu(false, Platform::Mac);
    assert_eq!(mac.entries[0].shortcut.as_deref(), Some("\u{2318}N"));
    assert_eq!(
        mac.entries[0].shortcut_aria.as_deref(),
        Some("Meta+N"),
        "Cmd is Meta for assistive technology"
    );
    assert_eq!(mac.entries[5].shortcut_aria.as_deref(), None);
    let win = file_menu(false, Platform::Windows);
    assert_eq!(win.entries[0].shortcut.as_deref(), Some("Ctrl+N"));
}

#[test]
fn a_placeholder_is_skipped_and_a_bad_model_does_not_resolve() {
    let list = Commands::new().with(Command::new("a", "a"));
    let tree = Arrangement::new(
        "m",
        vec![
            MenuItem::command("a"),
            MenuItem::Placeholder("search".into()),
        ],
    );
    let items = resolve(&tree, &list).unwrap();
    let say = menu_model_common::say(false);
    let names = fs_components_app::commands::KeyNames::english();
    let texts = fs_components_app::ModelTexts {
        say: &say,
        platform: Platform::Linux,
        names: &names,
    };
    assert_eq!(Menu::from_model("M", &items, &texts).entries.len(), 1);
    let bad = Arrangement::new("m", vec![MenuItem::Separator, MenuItem::command("a")]);
    assert_eq!(
        resolve(&bad, &list),
        Err(vec![Problem::SeparatorAtEdge { path: vec![0] }])
    );
}

#[test]
fn several_separators_are_no_duplicate_id_and_are_no_choice() {
    let widget = app(false);
    assert_eq!(
        widget.with_shell(|s| s.menus().len()),
        2,
        "set_menus took a menu with four separators"
    );
    let chosen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let sink = std::sync::Arc::clone(&chosen);
    widget
        .set_menus(vec![file_menu(false, Platform::Linux)], move |id| {
            sink.lock().unwrap().push(id.to_owned());
        })
        .unwrap();
    assert!(!widget.choose(""), "a separator is no choice");
    assert!(!widget.choose("document-save"), "disabled");
    assert!(widget.choose("document-new"));
    assert_eq!(*chosen.lock().unwrap(), vec!["document-new".to_owned()]);
}

#[test]
fn old_entries_look_and_behave_as_before() {
    let entry = MenuEntry::new("a", "A").checked(true);
    assert_eq!(entry.mark, Mark::Plain);
    assert!(entry.checked && !entry.separator);
    assert!(entry.icon.is_none() && entry.shortcut.is_none() && entry.reason.is_none());
}

#[test]
fn a_disabled_line_with_a_reason_is_as_high_as_a_free_one_and_says_why_only_in_a_tooltip() {
    let mut harness = Harness::new(app(false), 800.0, 500.0);
    harness.click("fs-app-menu-0");
    assert!(harness.has("fs-app-menu-panel"));
    for id in [
        "document-new",
        "document-save",
        "view-list",
        "files.hidden",
        "application-quit",
    ] {
        assert!(harness.has(&format!("fs-app-menu-entry-{id}")), "{id}");
    }
    let (_, _, _, new_h) = harness.bounds("fs-app-menu-entry-document-new");
    let (_, _, _, save_h) = harness.bounds("fs-app-menu-entry-document-save");
    assert!(
        (save_h - new_h).abs() < 0.5,
        "no second line for the reason: {save_h} == {new_h}"
    );
    assert_eq!(
        harness
            .attr("fs-app-menu-entry-document-save", "aria-disabled")
            .as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.attr("fs-app-menu-entry-document-save", "disabled"),
        None,
        "never the HTML attribute"
    );
    assert_eq!(
        harness.attr("fs-app-menu-entry-document-save", "aria-description"),
        None
    );
    assert_eq!(
        harness
            .attr("fs-app-menu-entry-document-save", "aria-describedby")
            .as_deref(),
        Some("fs-app-menu-entry-document-save-why-bubble")
    );
    let (_, _, panel_w, _) = harness.bounds("fs-app-menu-panel");
    assert!(panel_w >= 200.0);
}

#[test]
fn the_submenu_opens_beside_with_its_own_lines() {
    let mut harness = Harness::new(app(false), 800.0, 500.0);
    harness.click("fs-app-menu-0");
    harness.click("fs-app-menu-entry-fs.submenu.9");
    assert!(harness.has("fs-app-menu-sub-panel"));
    assert!(harness.has("fs-app-menu-entry-edit-copy"));
    assert!(harness.has("fs-app-menu-entry-edit-delete"));
}

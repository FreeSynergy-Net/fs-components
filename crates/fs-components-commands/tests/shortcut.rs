//! Tests: parser of the shortcut text (`aria-keyshortcuts` + `Primary`), round trip, display per platform and language.

use fs_components_commands::{KeyNames, Platform, Shortcut, ShortcutError};

fn s(text: &str) -> Shortcut {
    Shortcut::parse(text).unwrap()
}

#[test]
fn parses_modifiers_and_key_and_normalises_the_letter() {
    let sc = s("Control+Shift+p");
    assert!(sc.modifiers().control && sc.modifiers().shift);
    assert!(!sc.modifiers().alt && !sc.modifiers().meta && !sc.modifiers().primary);
    assert_eq!(sc.key(), "P");
    assert!(s("Primary+S").modifiers().primary);
    assert!(s("Alt+x").modifiers().alt);
    assert!(s("Meta+x").modifiers().meta);
}

#[test]
fn named_keys_and_function_keys() {
    assert_eq!(s("F1").key(), "F1");
    assert_eq!(s("f12").key(), "F12");
    assert_eq!(s("Alt+ArrowDown").key(), "ArrowDown");
    assert_eq!(s("delete").key(), "Delete");
    assert_eq!(
        Shortcut::parse("F25"),
        Err(ShortcutError::UnknownKey("F25".into()))
    );
    assert_eq!(
        Shortcut::parse("F0"),
        Err(ShortcutError::UnknownKey("F0".into()))
    );
    assert_eq!(s("F24").key(), "F24");
}

#[test]
fn the_plus_key() {
    let plus = s("Primary++");
    assert_eq!(plus.key(), "+");
    assert!(plus.modifiers().primary);
    assert_eq!(s("+").key(), "+");
    assert_eq!(s("Control+Shift++").aria(), "Control+Shift++");
    assert_eq!(Shortcut::parse("Control+"), Err(ShortcutError::NoKey));
}

#[test]
fn errors() {
    assert_eq!(Shortcut::parse(""), Err(ShortcutError::Empty));
    assert_eq!(Shortcut::parse("  "), Err(ShortcutError::Empty));
    assert_eq!(Shortcut::parse("Control+Shift"), Err(ShortcutError::NoKey));
    assert_eq!(
        Shortcut::parse("Control+Control+S"),
        Err(ShortcutError::DuplicateModifier("Control".into()))
    );
    assert_eq!(
        Shortcut::parse("Control+S+T"),
        Err(ShortcutError::TwoKeys("T".into()))
    );
    assert_eq!(
        Shortcut::parse("Control+Fnord"),
        Err(ShortcutError::UnknownKey("Fnord".into()))
    );
}

#[test]
fn round_trip_in_a_fixed_modifier_order() {
    for text in [
        "Primary+Shift+P",
        "Control+Alt+Delete",
        "Shift+Meta+ArrowLeft",
        "F5",
        "Primary+Control+Alt+Shift+Meta+Q",
        "Primary++",
    ] {
        assert_eq!(s(text).aria(), text);
        assert_eq!(s(&s(text).aria()), s(text));
    }
    assert_eq!(s("Shift+Control+s").aria(), "Control+Shift+S");
}

#[test]
fn a_list_is_separated_by_spaces() {
    let list = Shortcut::parse_list("Control+S  Meta+S").unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[1].aria(), "Meta+S");
    assert_eq!(Shortcut::parse_list("").unwrap().len(), 0);
    assert!(Shortcut::parse_list("Control+S Control").is_err());
}

#[test]
fn primary_is_control_off_the_mac_and_command_on_it() {
    let p = s("Primary+S");
    assert!(p.same_as(&s("Control+S"), Platform::Linux));
    assert!(p.same_as(&s("Control+S"), Platform::Windows));
    assert!(!p.same_as(&s("Control+S"), Platform::Mac));
    assert!(p.same_as(&s("Meta+S"), Platform::Mac));
    assert!(!p.same_as(&s("Meta+S"), Platform::Linux));
    assert!(!p.same_as(&s("Primary+T"), Platform::Linux));
    assert!(!p.same_as(&s("Primary+Shift+S"), Platform::Linux));
    assert!(!p.same_as(&s("Primary+Alt+S"), Platform::Linux));
}

#[test]
fn display_per_platform_and_language() {
    let sc = s("Primary+Shift+P");
    assert_eq!(
        sc.display(Platform::Linux, &KeyNames::german()),
        "Strg+Umschalt+P"
    );
    assert_eq!(
        sc.display(Platform::Windows, &KeyNames::english()),
        "Ctrl+Shift+P"
    );
    assert_eq!(
        sc.display(Platform::Mac, &KeyNames::german()),
        "\u{21e7}\u{2318}P"
    );
    assert_eq!(
        s("Control+Alt+Meta+Shift+X").display(Platform::Mac, &KeyNames::english()),
        "\u{2303}\u{2325}\u{21e7}\u{2318}X"
    );
    assert_eq!(
        s("Control+Alt+Meta+Shift+X").display(Platform::Linux, &KeyNames::english()),
        "Ctrl+Alt+Shift+Super+X"
    );
}

#[test]
fn display_of_named_keys() {
    let en = KeyNames::english();
    let de = KeyNames::german();
    assert_eq!(s("Delete").display(Platform::Linux, &de), "Entf");
    assert_eq!(s("Delete").display(Platform::Linux, &en), "Del");
    assert_eq!(s("Escape").display(Platform::Linux, &en), "Esc");
    assert_eq!(
        s("Alt+ArrowUp").display(Platform::Linux, &en),
        "Alt+\u{2191}"
    );
    assert_eq!(s("ArrowDown").display(Platform::Linux, &en), "\u{2193}");
    assert_eq!(s("ArrowLeft").display(Platform::Linux, &en), "\u{2190}");
    assert_eq!(s("ArrowRight").display(Platform::Linux, &en), "\u{2192}");
    assert_eq!(s("F2").display(Platform::Linux, &en), "F2");
    assert_eq!(s("Enter").display(Platform::Linux, &en), "Enter");
    let custom = KeyNames::english().key("Enter", "Eingabe");
    assert_eq!(s("Enter").display(Platform::Linux, &custom), "Eingabe");
}

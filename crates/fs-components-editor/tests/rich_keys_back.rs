//! Backspace giving a shortcut back, and Up and Down over a rule, in the rich editor session.
//!
//! Tests: model tests, no window.

use fs_components_editor::Doc;
use fs_components_editor::rich::{Key, Session};

fn page() -> Session {
    Session::new(&Doc::new(vec![String::new()]))
}

fn keys(s: &mut Session, text: &str) {
    for c in text.chars() {
        s.key(if c == '\n' { Key::Enter } else { Key::Char(c) });
    }
}

fn md(s: &Session) -> String {
    s.live_document().to_markdown()
}

fn text_before_cursor(s: &Session) -> String {
    let c = s.cursor();
    s.editor().text(&c.path)[..c.at].to_owned()
}

#[test]
fn backspace_gives_a_task_shortcut_back_as_the_text_typed_and_nothing_more() {
    for (shortcut, back) in [
        ("[] ", "[] "),
        ("[x] ", "[x] "),
        ("## ", "## "),
        ("- ", "- "),
        ("1. ", "1. "),
    ] {
        let mut s = page();
        keys(&mut s, shortcut);
        s.key(Key::Backspace);
        assert_eq!(
            md(&s),
            format!("{back}\n"),
            "{shortcut:?}: only what was typed, no list that was never typed"
        );
        assert!(
            matches!(
                s.editor().node(&s.cursor().path),
                Some(fs_components_editor::rich::Node::Para(_))
            ),
            "{shortcut:?}: a paragraph again"
        );
    }
}

#[test]
fn a_shortcut_is_given_back_only_straight_after_it_not_after_the_cursor_walked() {
    for walk in [
        [Key::Right, Key::Left],
        [Key::Left, Key::Right],
        [Key::Up, Key::Down],
        [Key::Home, Key::End],
    ] {
        let mut s = page();
        keys(&mut s, "## ");
        for k in walk {
            s.key(k);
        }
        s.key(Key::Backspace);
        assert!(
            !md(&s).starts_with("##"),
            "after {walk:?} Backspace must not give the shortcut back: {:?}",
            md(&s)
        );
    }
    // Counter-case: straight after, it does.
    let mut s = page();
    keys(&mut s, "## ");
    s.key(Key::Backspace);
    assert!(md(&s).starts_with("##"));
}

#[test]
fn up_and_down_walk_over_a_rule_and_do_not_stop_on_it() {
    let mut s = Session::new(&Doc::parse("above\n\n---\n\nbelow\n"));
    s.key(Key::Down);
    assert_eq!(
        text_before_cursor(&s),
        "",
        "start of the paragraph below, not the rule"
    );
    assert_eq!(s.editor().text(&s.cursor().path), "below");
    s.key(Key::Up);
    assert_eq!(s.editor().text(&s.cursor().path), "above");
    // Counter-case: Left and Right do stop on the rule (Backspace below it takes it away).
    s.key(Key::End);
    s.key(Key::Right);
    assert_eq!(s.editor().text(&s.cursor().path), "");
}

//! The cursor and the keys of the rich editor session: typing a page, walking over the blocks, Enter and Backspace in lists,
//! quotes and code. The keys are the ones a person presses; the page is what comes out.
//!
//! Tests: model tests, no window.

use fs_components_editor::Doc;
use fs_components_editor::rich::{Key, Session};

fn page() -> Session {
    Session::new(&Doc::new(vec![String::new()]))
}

/// Press the keys for `text` (`\n` is Enter).
fn keys(s: &mut Session, text: &str) {
    for c in text.chars() {
        s.key(if c == '\n' { Key::Enter } else { Key::Char(c) });
    }
}

fn md(s: &Session) -> String {
    s.document().to_markdown()
}

fn text_before_cursor(s: &Session) -> String {
    let c = s.cursor();
    s.editor().text(&c.path)[..c.at].to_owned()
}

#[test]
fn typing_a_page_with_shortcuts_makes_the_blocks() {
    let mut s = page();
    keys(
        &mut s,
        "## Plan\nThings to do:\n[] write\nagenda\n\n```rust\nfn main() {}\n\n---\n> said\n\n1. one\ntwo",
    );
    assert_eq!(
        md(&s),
        "## Plan\n\nThings to do:\n\n- [ ] write\n- [ ] agenda\n\n```rust\nfn main() {}\n```\n\n---\n\n> said\n\n1. one\n2. two\n"
    );
}

#[test]
fn arrows_walk_over_blocks_and_stop_at_the_ends() {
    let mut s = Session::new(&Doc::parse("# Head\n\n- a\n- b\n\n---\n\nlast\n"));
    s.key(Key::Left);
    assert_eq!(s.cursor().at, 0, "nothing before the first character");
    // Right over every character and block boundary to the very end, then no further.
    for _ in 0..40 {
        s.key(Key::Right);
    }
    let end = s.cursor().clone();
    assert_eq!(s.editor().text(&end.path), "last");
    assert_eq!(end.at, 4, "the end of the last leaf");
    s.key(Key::Right);
    assert_eq!(s.cursor(), &end, "Right at the end of the page stays");
    // Back again: Left crosses the boundaries.
    let mut seen = vec![end.path.clone()];
    for _ in 0..40 {
        s.key(Key::Left);
        if seen.last() != Some(&s.cursor().path) {
            seen.push(s.cursor().path.clone());
        }
    }
    assert_eq!(
        seen.len(),
        5,
        "last, rule, item b, item a, heading: {seen:?}"
    );
    assert_eq!(s.cursor().path, vec![0]);
    assert_eq!(s.cursor().at, 0);
    assert_eq!(s.take_ops().len(), 0, "moving the cursor changes nothing");
}

#[test]
fn up_and_down_keep_the_column_over_short_lines() {
    let mut s = Session::new(&Doc::parse("abcdef\n\nab\n\nabcdef\n"));
    for _ in 0..4 {
        s.key(Key::Right);
    }
    assert_eq!(s.cursor().at, 4);
    s.key(Key::Down);
    assert_eq!(
        (s.cursor().path.clone(), s.cursor().at),
        (vec![1], 2),
        "the short line takes the cursor to its end"
    );
    s.key(Key::Down);
    assert_eq!(
        (s.cursor().path.clone(), s.cursor().at),
        (vec![2], 4),
        "the column is remembered"
    );
    s.key(Key::Up);
    s.key(Key::Up);
    assert_eq!((s.cursor().path.clone(), s.cursor().at), (vec![0], 4));
    s.key(Key::Up);
    assert_eq!(s.cursor().path, vec![0], "up from the first line stays");
    s.key(Key::End);
    assert_eq!(s.cursor().at, 6);
    s.key(Key::Home);
    assert_eq!(s.cursor().at, 0);
}

#[test]
fn up_and_down_move_between_the_lines_of_a_code_block() {
    let mut s = Session::new(&Doc::parse("```\nabc\nde\n```\n"));
    s.key(Key::End);
    assert_eq!(s.cursor().at, 3);
    s.key(Key::Down);
    assert_eq!(s.cursor().at, 6, "the second line, clamped to its end");
    s.key(Key::Home);
    assert_eq!(s.cursor().at, 4);
    s.key(Key::Down);
    assert_eq!(s.cursor().at, 6, "no line below: the end of the text");
}

#[test]
fn backspace_walks_back_through_a_task_item() {
    let mut s = Session::new(&Doc::parse("before\n\n- [x] item\n"));
    s.key(Key::Down);
    assert_eq!(s.cursor().path, vec![1, 0, 0]);
    s.key(Key::Backspace);
    assert_eq!(
        md(&s),
        "before\n\n- item\n",
        "first the task box goes, the item stays"
    );
    s.key(Key::Backspace);
    assert_eq!(md(&s), "before\n\nitem\n", "then the item leaves the list");
    s.key(Key::Backspace);
    assert_eq!(
        md(&s),
        "beforeitem\n",
        "then the paragraph joins the one above"
    );
    assert_eq!(text_before_cursor(&s), "before");
    // Counter-case: Backspace at the very start of the page does nothing.
    let mut first = Session::new(&Doc::parse("only\n"));
    first.key(Key::Backspace);
    assert_eq!(md(&first), "only\n");
    assert_eq!(first.take_ops().len(), 0);
}

#[test]
fn backspace_in_the_middle_of_a_list_splits_it_around_the_paragraph() {
    let mut s = Session::new(&Doc::parse("1. a\n2. b\n3. c\n"));
    s.key(Key::Down);
    s.key(Key::Backspace);
    assert_eq!(
        md(&s),
        "1. a\n\nb\n\n3. c\n",
        "the items before and after stay lists; the numbers go on"
    );
}

#[test]
fn enter_on_an_empty_quoted_paragraph_leaves_the_quote() {
    let mut s = page();
    keys(&mut s, "> a\nb\n\nafter");
    assert_eq!(md(&s), "> a\n>\n> b\n\nafter\n");
    // Backspace at the start of the second quoted paragraph joins it to the first.
    let mut t = Session::new(&Doc::parse("> a\n>\n> b\n"));
    t.key(Key::Down);
    t.key(Key::Backspace);
    assert_eq!(md(&t), "> ab\n");
    // Backspace at the start of the first quoted paragraph takes it out of the quote.
    let mut u = Session::new(&Doc::parse("> a\n>\n> b\n"));
    u.key(Key::Backspace);
    assert_eq!(md(&u), "a\n\n> b\n");
}

#[test]
fn a_rule_goes_with_backspace_and_enter_opens_a_paragraph_after_it() {
    let mut s = Session::new(&Doc::parse("a\n\n---\n\nb\n"));
    s.key(Key::Right);
    s.key(Key::Right);
    assert_eq!(
        s.cursor().path,
        vec![1],
        "Right stops on the rule (Up and Down do not)"
    );
    s.key(Key::Enter);
    assert_eq!(md(&s), "a\n\n---\n\n\n\nb\n");
    let mut t = Session::new(&Doc::parse("a\n\n---\n\nb\n"));
    t.key(Key::Right);
    t.key(Key::Right);
    t.key(Key::Backspace);
    assert_eq!(md(&t), "a\n\nb\n");
    assert_eq!(text_before_cursor(&t), "a");
    // Backspace at the start of the paragraph under a rule removes the rule and the cursor stays.
    let mut u = Session::new(&Doc::parse("a\n\n---\n\nb\n"));
    u.key(Key::Down);
    u.key(Key::Backspace);
    assert_eq!(md(&u), "a\n\nb\n");
    assert_eq!(u.cursor().path, vec![1]);
}

#[test]
fn a_click_on_the_box_toggles_the_task_and_moves_nothing_else() {
    let mut s = Session::new(&Doc::parse("- [ ] a\n- [ ] b\n"));
    s.click_task(&[0, 1, 0]);
    assert_eq!(md(&s), "- [ ] a\n- [x] b\n");
    s.click_task(&[0, 0, 0]);
    s.click_task(&[0, 0, 0]);
    assert_eq!(md(&s), "- [ ] a\n- [x] b\n", "twice is none");
    let ops = s.take_ops();
    assert_eq!(ops.len(), 1, "one block, one operation");
}

#[test]
fn enter_at_the_start_of_text_opens_an_empty_block_above() {
    let mut s = Session::new(&Doc::parse("## Head\n\n- [ ] item\n"));
    s.key(Key::Enter);
    assert_eq!(md(&s), "\n\n## Head\n\n- [ ] item\n");
    assert_eq!(
        s.cursor().path,
        vec![1],
        "the cursor stays with the heading"
    );
    let mut t = Session::new(&Doc::parse("- [ ] item\n"));
    t.key(Key::Enter);
    assert_eq!(md(&t), "- [ ] \n- [ ] item\n", "an empty open task above");
}

#[test]
fn multibyte_keys_do_not_cut_a_character() {
    let mut s = page();
    keys(&mut s, "Gr\u{fc}\u{df}e \u{e4}");
    s.key(Key::Left);
    s.key(Key::Left);
    s.key(Key::Backspace);
    assert_eq!(
        md(&s),
        "Gr\u{fc}\u{df} \u{e4}\n",
        "the e before the space goes, whole characters stay"
    );
    for _ in 0..12 {
        s.key(Key::Left);
        s.key(Key::Up);
        s.key(Key::Home);
    }
    assert_eq!(s.cursor().at, 0);
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

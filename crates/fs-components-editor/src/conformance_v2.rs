//! The cases of contract version 2 of the conformance collection: Backspace, the task box, and the shortcuts.
//!
//! Pattern: Template method — [`check`] is called by [`crate::conformance::check`] with the same maker of editors; the shortcut
//! cases run only for an editor that says [`TextEditor::input_rules`].
//!
//! Every case has a counter-case that must not change anything, so a test that cannot fail does not pass by itself.

use crate::{Doc, Edit, TextEditor, apply};

type Make<'a> = &'a dyn Fn() -> Box<dyn TextEditor>;

/// Load `text`, do `steps`, and check the operations give exactly the document of the editor; the document's blocks.
fn run(make: Make, text: &str, steps: &[Edit]) -> Vec<String> {
    // The text of "an empty page with one empty block" is the empty text; `Doc::parse` of it has no block at all.
    let doc = if text.trim().is_empty() {
        Doc::new(vec![String::new()])
    } else {
        Doc::parse(text)
    };
    let mut editor = make();
    editor.load(&doc);
    for step in steps {
        editor.edit(step);
    }
    let ops = editor.take_ops();
    let mut replayed = doc.clone();
    apply(&mut replayed, &ops).expect("operations fit the document");
    assert_eq!(
        replayed,
        editor.document(),
        "{steps:?}: operations do not give the document"
    );
    assert_eq!(
        ops.is_empty(),
        editor.document() == doc,
        "{steps:?}: operations and change disagree"
    );
    editor.document().blocks
}

fn back(block: usize, at: usize) -> Edit {
    Edit::Backspace { block, at }
}

fn typed(block: usize, at: usize, text: &str) -> Edit {
    Edit::Type {
        block,
        at,
        text: text.into(),
    }
}

/// The cases of version 2 against the editors `make` gives.
///
/// # Panics
/// With the case when one fails.
pub(crate) fn check(make: Make) {
    let name = make().name();
    let b = |t: &[&str]| t.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    // Backspace: a character goes, at the start of a block it joins the one above, at the very start nothing happens.
    assert_eq!(
        run(make, "abc\n\ndef\n", &[back(0, 3)]),
        b(&["ab", "def"]),
        "{name}: Backspace takes a character"
    );
    assert_eq!(
        run(make, "abc\n\ndef\n", &[back(1, 0)]),
        b(&["abcdef"]),
        "{name}: Backspace joins the block"
    );
    assert_eq!(
        run(make, "abc\n\ndef\n", &[back(0, 0)]),
        b(&["abc", "def"]),
        "{name}: Backspace at the very start"
    );
    // The task box: one line of a list, counter-case a paragraph.
    let tasks = "- [ ] a\n- [x] b\n";
    assert_eq!(
        run(make, tasks, &[Edit::ToggleTask { block: 0, at: 2 }]),
        b(&["- [x] a\n- [x] b"]),
        "{name}: tick"
    );
    assert_eq!(
        run(make, tasks, &[Edit::ToggleTask { block: 0, at: 10 }]),
        b(&["- [ ] a\n- [ ] b"]),
        "{name}: untick"
    );
    let twice = [
        Edit::ToggleTask { block: 0, at: 2 },
        Edit::ToggleTask { block: 0, at: 2 },
    ];
    assert_eq!(
        run(make, tasks, &twice),
        b(&["- [ ] a\n- [x] b"]),
        "{name}: twice is none"
    );
    assert_eq!(
        run(make, "plain\n", &[Edit::ToggleTask { block: 0, at: 2 }]),
        b(&["plain"]),
        "{name}: no box, no change"
    );
    if make().input_rules() {
        shortcuts(make, name);
    }
}

/// The shortcuts fire on the Space (or Enter) and not otherwise.
fn shortcuts(make: Make, name: &str) {
    let b = |t: &[&str]| t.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let empty = "\n";
    // The result differs from what was typed exactly where a rule fired.
    for (keys, want) in [
        ("[] todo", "- [ ] todo"),
        ("[x] done", "- [x] done"),
        ("- item", "- item"),
        ("1. first", "1. first"),
        ("> said", "> said"),
        ("## Title", "## Title"),
    ] {
        assert_eq!(
            run(make, empty, &[typed(0, 0, keys)]),
            b(&[want]),
            "{name}: {keys:?}"
        );
    }
    // Counter-cases: no Space, shortcut not at the start, seven hashes: plain text, nothing turned.
    for keys in ["[]todo", "a [] b", "####### x", "-item"] {
        let got = run(make, empty, &[typed(0, 0, keys)]);
        assert_eq!(got, b(&[keys]), "{name}: {keys:?} must stay text");
    }
    // Enter turns a fence line into a code block and `---` into a rule with a paragraph after it.
    let fence = [typed(0, 0, "```rust"), Edit::Enter { block: 0, at: 7 }];
    assert_eq!(
        run(make, empty, &fence),
        b(&["```rust\n\n```"]),
        "{name}: fence"
    );
    let rule = [typed(0, 0, "---"), Edit::Enter { block: 0, at: 3 }];
    assert_eq!(run(make, empty, &rule), b(&["---", ""]), "{name}: rule");
    // Backspace right after a shortcut gives the typed characters back; without the shortcut it takes one block level.
    let undone = [typed(0, 0, "## "), back(0, 3), typed(0, 3, "x")];
    assert_eq!(
        run(make, empty, &undone),
        b(&["## x"]),
        "{name}: the shortcut is given back"
    );
    let not_undone = [typed(0, 0, "## "), typed(0, 3, "y"), back(0, 3)];
    assert_eq!(
        run(make, empty, &not_undone),
        b(&["y"]),
        "{name}: later, the heading only becomes a paragraph"
    );
}

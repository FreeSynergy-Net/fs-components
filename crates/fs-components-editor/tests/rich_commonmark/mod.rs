//! Shared by the `CommonMark` tests of the saved form: the texts and the pulldown-cmark event shapes.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// The texts that look like a marker, and plain ones.
pub const TEXTS: &[&str] = &[
    "## x",
    "# x",
    "###### x",
    "#",
    "##",
    "#\tx",
    "- x",
    "-",
    "-\tx",
    "* x",
    "+ x",
    "1. x",
    "12. x",
    "123456789. x",
    "1) x",
    "1.",
    "1)",
    "1.\tx",
    "1)\tx",
    "> x",
    ">x",
    "---",
    "***",
    "___",
    "```rust",
    "~~~",
    "[x] x",
    "[ ] x",
    "\\## x",
    "\\- x",
    "\\1. x",
    "1\\. x",
    "1\\) x",
    "plain",
    "a - b",
    "2026",
    "1.x",
    "1234567890. x",
    "_ _ _",
    "* * *",
    "- - -",
    "-  -  -",
    "_\t_\t_",
    "* * * *",
    "<!-- x",
    "<!--",
    "<script>",
    "<SCRIPT src=x>",
    "<pre>x",
    "<style",
    "<textarea>",
    "<?php x",
    "<!DOCTYPE html>",
    "<![CDATA[x",
    "<div>x",
    "</div>",
    "<DIV class=a>",
    "<table>",
    "<hr/>",
    "<p>",
    "[a]: b",
    "[a]: b 'c'",
    "[a]:",
    "a [b] c",
    "<3 you",
];

/// The events of `source` as `P(text)`; anything but a paragraph shows as its tag. `paragraphs` is what the text of the
/// paragraph should be (one for the single case).
pub fn shape(source: &str) -> Vec<String> {
    // Both readers, without and with the GFM extensions, must see the same.
    let plain = shape_with(source, Options::empty());
    let gfm = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_FOOTNOTES;
    assert_eq!(
        plain,
        shape_with(source, gfm),
        "plain and GFM differ for {source:?}"
    );
    plain
}

fn shape_with(source: &str, options: Options) -> Vec<String> {
    let mut out = Vec::new();
    let mut last_text = false;
    for e in Parser::new_ext(source, options) {
        let was_text = matches!(e, Event::Text(_));
        match e {
            Event::Start(Tag::Paragraph) => out.push("P[".to_owned()),
            Event::End(TagEnd::Paragraph) => out.push("]".to_owned()),
            // GFM cuts a text at `~`: neighbouring texts are one text.
            Event::Text(t) if last_text => {
                if let Some(l) = out.last_mut() {
                    l.push_str(&t);
                }
            }
            Event::Text(t) => out.push(t.to_string()),
            Event::Start(t) => out.push(format!("\u{27e8}{t:?}\u{27e9}")),
            Event::End(t) => out.push(format!("\u{27e8}/{t:?}\u{27e9}")),
            other => out.push(format!("{other:?}")),
        }
        last_text = was_text;
    }
    out
}

/// The text of the one paragraph in `source`, or the reason it is not one.
pub fn paragraph_text(source: &str) -> Result<String, Vec<String>> {
    let s = shape(source);
    if s.first().map(String::as_str) == Some("P[") && s.last().map(String::as_str) == Some("]") {
        let inner = &s[1..s.len() - 1];
        if inner.iter().all(|e| {
            !e.starts_with('\u{27e8}') && !e.starts_with("Start") && !e.starts_with("Code")
        }) {
            return Ok(inner.concat());
        }
    }
    Err(s)
}

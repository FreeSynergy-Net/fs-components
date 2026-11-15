//! The pictures for the acceptance of 0.0.20 (U1 to U8, tree title, narrow Classic), drawn the way the program draws them:
//! through `snapshot` (window styles, the theme of the chosen style).
//!
//! Tests: one test that only draws; it keeps pictures when `FS_COMP_SHOTS` names a directory (else it does nothing). The style
//! is chosen with `FS_ABNAHME_STYLE` (`craft`, `admin`, `simple`, `apple`, `windows`; none = the platform default), because a
//! process has one style; the style is part of every file name.

mod common;

use std::sync::Arc;

use common::draw::{Scratch, fonts_work};
use fs_components_workbenches::{
    Block, Document, DocumentSource, EditableText, Entry, Hit, Inline, Layout, LinkTarget,
    ListItem, Picture, SourceError, Step, snapshot,
};

const LONG: &str = "Kapitel eins mit einem sehr langen Titel, der nicht in den Baum passt";

/// Three pages built as blocks by hand (the in-memory source of the other tests reads no Markdown).
struct Pages;

impl DocumentSource for Pages {
    fn entries(&self, _prefix: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(vec![
            Entry::new("handbuch.md", "Handbuch"),
            Entry::new("handbuch/kapitel.md", LONG),
            Entry::new("notizen.md", "Notizen"),
        ])
    }
    fn read(&self, path: &str) -> Result<Document, SourceError> {
        let h = |level, t: &str| Block::Heading {
            level,
            text: Inline::plain(t),
        };
        let p = |t: &str| Block::Paragraph(Inline::plain(t));
        let item = |t: &str, done| ListItem::new(vec![p(t)]).task(done);
        let cell = |t: &str| Inline::plain(t);
        let blocks = match path {
            "handbuch.md" => vec![
                h(1, "Handbuch"),
                p(
                    "Ein Absatz mit einem Satz, der lang genug ist, um die Schriftgroesse zu zeigen.",
                ),
                h(2, "Aufgaben"),
                Block::List {
                    ordered: false,
                    start: 1,
                    items: vec![
                        item("offene Aufgabe", false),
                        item("erledigte Aufgabe", true),
                    ],
                },
                h(2, "Tabelle"),
                Block::Table {
                    align: vec![],
                    rows: vec![
                        vec![cell("Name"), cell("Fertig")],
                        vec![cell("Handlungshoheit"), cell("[ ]")],
                        vec![cell("Zweite Zeile"), cell("[x]")],
                    ],
                },
                h(2, "Code"),
                Block::Code {
                    lang: String::new(),
                    text: "cool".into(),
                },
                Block::Code {
                    lang: String::new(),
                    text: "fn main() {\n    println!(\"hallo\");\n}".into(),
                },
                p("Schluss."),
            ],
            "handbuch/kapitel.md" => vec![h(1, LONG), p("Text.")],
            _ => vec![h(1, "Notizen"), p("Text.")],
        };
        let mut doc = Document::new(path, blocks);
        doc.title = match path {
            "handbuch.md" => "Handbuch".into(),
            "handbuch/kapitel.md" => LONG.into(),
            _ => "Notizen".into(),
        };
        Ok(doc)
    }
    fn resolve(&self, _from: &str, href: &str) -> LinkTarget {
        LinkTarget::Missing(href.into())
    }
    fn find(&self, _term: &str) -> Result<Vec<Hit>, SourceError> {
        Ok(Vec::new())
    }
    fn can_edit(&self, _path: &str) -> bool {
        true
    }
    fn source_text(&self, _path: &str) -> Result<EditableText, SourceError> {
        Ok(EditableText::new(PAGE.to_owned(), Some("1".into())))
    }
}

const PAGE: &str = "# Handbuch\n\nEin Absatz mit `Code im Text`.\n\n## Aufgaben\n\n- [ ] offene Aufgabe\n- [x] erledigte Aufgabe\n\n## Code\n\n```\ncool\n```\n\nSchluss.\n";

fn shoot(layout: Layout, w: u32, h: u32, steps: Vec<Step>, full: bool, name: &str) {
    let Some(dir) = std::env::var_os("FS_COMP_SHOTS") else {
        return;
    };
    let style = std::env::var("FS_ABNAHME_STYLE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "standard".into());
    fs_components_render::use_style(fs_components_styles::StyleId::from_name(&style));
    let tmp = Scratch::new();
    assert!(fonts_work(&tmp), "no fonts or no graphics");
    let source = Pages;
    let mut picture = Picture::new(Arc::new(source), w, h);
    picture.layout = layout;
    picture.full_width = full;
    picture.steps = steps;
    let file = tmp.file(name);
    snapshot(picture, &file).expect("picture");
    std::fs::create_dir_all(&dir).unwrap();
    let out = format!("{style}-{name}");
    std::fs::copy(&file, std::path::Path::new(&dir).join(out)).unwrap();
}

fn open() -> Vec<Step> {
    vec![Step::Open("handbuch.md".into())]
}

fn tree_open() -> Vec<Step> {
    vec![
        Step::Open("handbuch.md".into()),
        Step::Expand("handbuch.md".into()),
    ]
}

#[test]
fn draw_the_pictures_of_the_acceptance() {
    use Layout::{Classic, Confluence, Notion};
    // U1/U2: boxes in a list and in a table cell; U7: table text; U8: code blocks.
    shoot(
        Notion,
        1000,
        1100,
        open(),
        false,
        "notion-u1u2u7u8-seite.png",
    );
    shoot(
        Confluence,
        1000,
        1100,
        open(),
        false,
        "confluence-u1u2u7u8-seite.png",
    );
    // U3: the writing surface.
    let mut edit = open();
    edit.push(Step::Edit);
    shoot(
        Notion,
        1000,
        800,
        edit,
        false,
        "notion-u3-schreibflaeche.png",
    );
    // U4/U5: the page with its outline at the right edge.
    shoot(Notion, 1000, 500, open(), false, "notion-u4u5-rand.png");
    // U6: width of the text.
    shoot(
        Notion,
        1900,
        700,
        open(),
        false,
        "notion-u6-spalte-1900.png",
    );
    shoot(
        Notion,
        1900,
        700,
        open(),
        true,
        "notion-u6-volle-breite-1900.png",
    );
    shoot(
        Notion,
        1300,
        700,
        open(),
        false,
        "notion-u6-spalte-1300.png",
    );
    // The tree with a long title (the cut one; the tooltip appears on hover and cannot be drawn offscreen).
    let tree = tree_open();
    shoot(Notion, 1000, 500, tree.clone(), false, "notion-baum.png");
    shoot(Confluence, 1000, 500, tree, false, "confluence-baum.png");
    // A narrow window: the tree is narrow, so a title is cut ("Handb...").
    shoot(
        Notion,
        560,
        400,
        tree_open(),
        false,
        "notion-baum-schmal-560.png",
    );
    shoot(
        Confluence,
        560,
        400,
        tree_open(),
        false,
        "confluence-baum-schmal-560.png",
    );
    shoot(
        Notion,
        380,
        400,
        tree_open(),
        false,
        "notion-baum-schmal-380.png",
    );
    // Classic, narrow.
    shoot(Classic, 900, 1100, open(), false, "classic-900.png");
    shoot(Classic, 480, 1300, open(), false, "classic-480.png");
}

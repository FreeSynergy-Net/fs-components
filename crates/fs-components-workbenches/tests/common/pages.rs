//! The three pages of the acceptance pictures, built as blocks by hand (the in-memory source of the other tests reads no
//! Markdown).

use fs_components_workbenches::{
    Block, Document, DocumentSource, EditableText, Entry, Hit, Inline, LinkTarget, ListItem,
    SourceError,
};

pub const LONG: &str = "Kapitel eins mit einem sehr langen Titel, der nicht in den Baum passt";

/// Three pages built as blocks by hand (the in-memory source of the other tests reads no Markdown).
pub struct Pages;

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

/// A picture of the wiki in `layout`, drawn the way the program draws it in the style `style` (`None`: the platform default);
/// `None` when the machine has no fonts or no graphics. A process has one style, so a test file uses one style only.
pub fn wiki_picture(
    style: Option<&str>,
    layout: fs_components_workbenches::Layout,
    (w, h): (u32, u32),
    name: &str,
) -> Option<image::RgbaImage> {
    use super::draw::{Scratch, fonts_work, pixels};
    use fs_components_workbenches::{Picture, Step, snapshot};
    fs_components_render::use_style(style.and_then(fs_components_styles::StyleId::from_name));
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let mut picture = Picture::new(std::sync::Arc::new(Pages), w, h);
    picture.layout = layout;
    picture.steps = vec![Step::Open("handbuch.md".into())];
    let file = tmp.file(name);
    snapshot(picture, &file).ok()?;
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).ok()?;
        let out = format!("{}-{layout:?}-{name}", style.unwrap_or("standard"));
        std::fs::copy(&file, std::path::Path::new(&dir).join(out.to_lowercase())).ok()?;
    }
    Some(pixels(&file))
}

/// The page ground: a pixel at the bottom right of the page, where nothing is drawn.
pub fn page_ground(img: &image::RgbaImage) -> [u8; 4] {
    img.get_pixel(img.width() - 60, img.height() - 20).0
}

/// The tallest run of rows (at x = `x`, right of the text) whose pixel stands off `ground` by 8 or more in a channel: a block
/// with a ground of its own (a code block is about 40 px high; a table line is 1 px).
pub fn tallest_ground_run(img: &image::RgbaImage, x: u32, ground: [u8; 4]) -> u32 {
    let off = |y: u32| (0..3).any(|c| img.get_pixel(x, y)[c].abs_diff(ground[c]) >= 8);
    let (mut best, mut run) = (0, 0);
    for y in 0..img.height() {
        run = if off(y) { run + 1 } else { 0 };
        best = best.max(run);
    }
    best
}

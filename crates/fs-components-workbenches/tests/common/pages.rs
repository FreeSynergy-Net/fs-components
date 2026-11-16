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
            // Only two boxes: the picture of the edge of an empty box has nothing else to mistake for it.
            "boxes.md" => vec![Block::List {
                ordered: false,
                start: 1,
                items: vec![item("offen", false), item("erledigt", true)],
            }],
            "handbuch/kapitel.md" => vec![h(1, LONG), p("Text.")],
            _ => vec![h(1, "Notizen"), p("Text.")],
        };
        let mut doc = Document::new(path, blocks);
        doc.title = match path {
            "handbuch.md" => "Handbuch".into(),
            "boxes.md" => "Boxen".into(),
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
/// `None` only when the machine has no fonts or no graphics (the probe of Blinc's own text says so). A process has one style, so
/// a test file uses one style only. An error while drawing, on a machine that does draw, is a failure of the test, never a skip.
pub fn wiki_picture(
    style: Option<&str>,
    layout: fs_components_workbenches::Layout,
    size: (u32, u32),
    name: &str,
) -> Option<image::RgbaImage> {
    wiki_picture_steps(
        style,
        layout,
        size,
        vec![fs_components_workbenches::Step::Open("handbuch.md".into())],
        name,
    )
}

/// Like [`wiki_picture`] with the steps the picture takes (open, expand, edit ...).
pub fn wiki_picture_steps(
    style: Option<&str>,
    layout: fs_components_workbenches::Layout,
    (w, h): (u32, u32),
    steps: Vec<fs_components_workbenches::Step>,
    name: &str,
) -> Option<image::RgbaImage> {
    use super::draw::{Scratch, fonts_work, pixels};
    use fs_components_workbenches::{Picture, snapshot};
    fs_components_render::use_style(style.and_then(fs_components_styles::StyleId::from_name));
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let mut picture = Picture::new(std::sync::Arc::new(Pages), w, h);
    picture.layout = layout;
    picture.steps = steps;
    let file = tmp.file(name);
    snapshot(picture, &file)
        .expect("the picture of the wiki is drawn (fonts and graphics work here)");
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).expect("the directory of the pictures");
        let out = format!("{}-{layout:?}-{name}", style.unwrap_or("standard"));
        std::fs::copy(&file, std::path::Path::new(&dir).join(out.to_lowercase()))
            .expect("the picture is kept");
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

fn rgba(p: image::Rgba<u8>) -> fs_components_styles::Rgba {
    fs_components_styles::Rgba {
        r: p[0],
        g: p[1],
        b: p[2],
        a: 255,
    }
}

/// WCAG contrast of two pixels.
pub fn contrast(a: image::Rgba<u8>, b: image::Rgba<u8>) -> f32 {
    rgba(a).contrast(rgba(b))
}

/// The colour of the edge of the first empty box of the page `boxes.md` (two tasks and nothing else below the head): in the first
/// row below the head (from `from` on, below the head of the page; right of the side column) that has five or more pixels off the ground, the colour most of
/// them have (the corners are round, so a few pixels of the row are softer).
pub fn empty_box_edge(img: &image::RgbaImage, ground: [u8; 4], from: u32) -> Option<[u8; 4]> {
    let off = |p: [u8; 4]| (0..3).any(|c| p[c].abs_diff(ground[c]) > 3);
    for y in from..img.height() {
        let mut seen: Vec<([u8; 4], u32)> = Vec::new();
        for x in 301..420 {
            let p = img.get_pixel(x, y).0;
            if !off(p) {
                continue;
            }
            match seen.iter_mut().find(|(c, _)| *c == p) {
                Some((_, n)) => *n += 1,
                None => seen.push((p, 1)),
            }
        }
        if seen.iter().map(|(_, n)| n).sum::<u32>() >= 5 {
            return seen.into_iter().max_by_key(|(_, n)| *n).map(|(c, _)| c);
        }
    }
    None
}

/// The top and bottom row (y0, y1) of the row of the tree that stands on a ground of its own (the selected one), looked at in
/// column `x` and below `from`; `ground` is the pixel of an ordinary row.
pub fn selected_row(
    img: &image::RgbaImage,
    x: u32,
    from: u32,
    ground: [u8; 4],
) -> Option<(u32, u32)> {
    let on = |y: u32| img.get_pixel(x, y).0[..3] != ground[..3];
    let top = (from..img.height()).find(|&y| on(y))?;
    let bottom = (top..img.height()).find(|&y| !on(y))? - 1;
    Some((top, bottom))
}

/// How many pixels of the box `xs` x `ys` reach the contrast `least` against `ground`.
pub fn strong_in(
    img: &image::RgbaImage,
    xs: std::ops::Range<u32>,
    ys: std::ops::RangeInclusive<u32>,
    ground: image::Rgba<u8>,
    least: f32,
) -> usize {
    ys.flat_map(|y| xs.clone().map(move |x| (x, y)))
        .filter(|&(x, y)| contrast(*img.get_pixel(x, y), ground) >= least)
        .count()
}

/// The first and last column of ink (a pixel off `ground` by 60 or more in the sum of channels) in rows `ys`, columns `xs`.
pub fn ink_extent(
    img: &image::RgbaImage,
    xs: std::ops::Range<u32>,
    ys: std::ops::Range<u32>,
    ground: [u8; 4],
) -> Option<(u32, u32)> {
    let has = |x: u32| {
        ys.clone().any(|y| {
            let p = img.get_pixel(x, y).0;
            (0..3)
                .map(|c| i32::from(p[c].abs_diff(ground[c])))
                .sum::<i32>()
                > 60
        })
    };
    let first = xs.clone().find(|&x| has(x))?;
    let last = xs.rev().find(|&x| has(x))?;
    Some((first, last))
}

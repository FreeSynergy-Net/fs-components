//! Finding U6 of the user test of 2026-11-14: the way to the page stood at the far left while the text stood in a narrow column in
//! the middle of a wide window. The way now starts where the text starts, and the page can be switched to "full width" (a setting
//! of the page in Notion's page menu, `menus.md`, `states.md`, `dialogs.md` of the Notion dossier; the dossier names no width in
//! pixels, so the 720 px column is our choice).
//!
//! Tests: pixel tests on the pictures of the Notion layout (skipped without graphics or fonts).

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, dark, fonts_work, pixels};
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

fn shoot(width: u32, full: bool, name: &str) -> Option<image::RgbaImage> {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let source = Editable::with(&[("a.md", "# Das ist eine Testseite.\n\nEin Absatz.\n")]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let mut picture = Picture::new(Arc::new(source), width, 700);
    picture.layout = Layout::Notion;
    picture.full_width = full;
    picture.steps = vec![Step::Open("a.md".into())];
    let file = tmp.file(name);
    if snapshot(picture, &file).is_err() {
        eprintln!("skipped: no graphics");
        return None;
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(&file, std::path::Path::new(&dir).join(name)).unwrap();
    }
    Some(pixels(&file))
}

/// The leftmost dark pixel right of the sidebar (x 270 and more) in the rows `y0..y1`.
fn left_ink(img: &image::RgbaImage, y0: u32, y1: u32) -> Option<u32> {
    (270..img.width()).find(|&x| (y0..y1).any(|y| dark(*img.get_pixel(x, y))))
}

#[test]
fn the_way_to_the_page_starts_where_the_text_starts() {
    for width in [1900, 1300] {
        let Some(img) = shoot(width, false, &format!("u6-breite-{width}.png")) else {
            return;
        };
        let way = left_ink(&img, 10, 34).expect("the way to the page");
        let title = left_ink(&img, 215, 260).expect("the title");
        assert!(
            way.abs_diff(title) <= 4,
            "at {width} px: the way starts at x {way}, the title at x {title}"
        );
    }
}

#[test]
fn full_width_gives_the_text_the_whole_width_and_the_column_stays_the_default() {
    let (Some(column), Some(wide)) = (
        shoot(1900, false, "u6-spalte.png"),
        shoot(1900, true, "u6-volle-breite.png"),
    ) else {
        return;
    };
    let (in_column, in_full) = (
        left_ink(&column, 215, 260).unwrap(),
        left_ink(&wide, 215, 260).unwrap(),
    );
    assert!(
        in_column > 600,
        "the default column is centred (title at x {in_column})"
    );
    assert!(
        in_full < 330,
        "full width starts beside the sidebar (title at x {in_full})"
    );
    let way = left_ink(&wide, 10, 34).unwrap();
    assert!(
        way.abs_diff(in_full) <= 4,
        "the way starts with the text in full width too ({way} and {in_full})"
    );
}

/// A page with headings of two levels (the Editable test source reads only the title as a heading).
struct Outlined;

impl fs_components_workbenches::DocumentSource for Outlined {
    fn entries(
        &self,
        _prefix: &str,
    ) -> Result<Vec<fs_components_workbenches::Entry>, fs_components_workbenches::SourceError> {
        Ok(vec![fs_components_workbenches::Entry::new("a.md", "T")])
    }
    fn read(
        &self,
        path: &str,
    ) -> Result<fs_components_workbenches::Document, fs_components_workbenches::SourceError> {
        use fs_components_workbenches::{Block, Document, Inline};
        let h = |level, t: &str| Block::Heading {
            level,
            text: Inline::plain(t),
        };
        let mut doc = Document::new(
            path,
            vec![
                h(1, "T"),
                h(2, "Eins"),
                Block::Paragraph(Inline::plain("text")),
                h(2, "Zwei"),
                h(3, "Drei"),
            ],
        );
        doc.title = "T".into();
        Ok(doc)
    }
    fn resolve(&self, _from: &str, href: &str) -> fs_components_workbenches::LinkTarget {
        fs_components_workbenches::LinkTarget::Missing(href.into())
    }
    fn find(
        &self,
        _term: &str,
    ) -> Result<Vec<fs_components_workbenches::Hit>, fs_components_workbenches::SourceError> {
        Ok(Vec::new())
    }
}

/// Finding U5: the marks of the outline at the right edge were cut off by the edge of the window. Each mark is drawn whole
/// (a mark of a heading of level 2 is 22 px long) and a few pixels stand between it and the edge.
#[test]
fn the_marks_of_the_outline_at_the_right_edge_are_whole() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let mut picture = Picture::new(Arc::new(Outlined), 1400, 500);
    picture.layout = Layout::Notion;
    picture.steps = vec![Step::Open("a.md".into())];
    let file = tmp.file("u5-markierungen.png");
    if snapshot(picture, &file).is_err() {
        eprintln!("skipped: no graphics");
        return;
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(
            &file,
            std::path::Path::new(&dir).join("u5-markierungen.png"),
        )
        .unwrap();
    }
    let img = pixels(&file);
    // A mark is a short line of the border colour in the last 60 columns, below the top bar.
    let grey = |x: u32, y: u32| {
        let p = img.get_pixel(x, y);
        p[0] < 235 && p[0] > 120 && p[0].abs_diff(p[1]) < 8
    };
    let marks: Vec<(u32, u32, u32)> = (60..img.height())
        .filter_map(|y| {
            let xs: Vec<u32> = (img.width() - 80..img.width())
                .filter(|&x| grey(x, y))
                .collect();
            Some((y, *xs.first()?, *xs.last()?))
        })
        .collect();
    assert!(marks.len() >= 3, "the marks are drawn: {marks:?}");
    for (y, left, right) in &marks {
        assert!(
            right - left >= 9,
            "row {y}: the mark is whole ({left}..{right})"
        );
        assert!(
            img.width() - right >= 8,
            "row {y}: the mark keeps off the edge of the window (ends at {right})"
        );
    }
}

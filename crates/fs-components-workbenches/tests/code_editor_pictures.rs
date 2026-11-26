//! Pictures of the code editor (E5 of the editor draft): a Markdown page (headings, a list, two code fences) with the preview beside
//! it (variant `split`), light and dark, the locked variant with its reason, and the two changes of a reading (a new media type, a
//! withdrawn `can`). Blinc keeps one theme per process, so each picture is drawn by this same test binary started again with
//! `FS_RENDER_SCHEME` set (the parent only looks at the pixels). Pictures stay local: `FS_COMP_SHOTS` names the folder, else the
//! target folder of cargo; they are never part of the repository.
//!
//! Tests: pixel probes on `draw_to_png` of the mounted `View` (no window; skipped without graphics). Run with
//! `--features syntect` for the colours of the text.

#![allow(clippy::cast_possible_truncation)] // pixel arithmetic in a test

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_core::states::Load;
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_styles::StyleId;
use fs_components_words::Locale;
use fs_components_workbenches::code_editor::View;

const PAGE: &str = "# FreeSynergy\n\nA short page.\n\n## Install\n\n- one\n- two\n- three\n\n```rust\nfn main() {\n    println!(\"hi\"); // greeting\n}\n```\n\n## Config\n\n```toml\n[server]\nport = 8080\n```\n";
const SIZE: (u32, u32) = (1000, 640);

struct Host(AtomicBool);

impl FormWrite for Host {
    fn can(&self, channel: &str, _: Option<&str>) -> bool {
        channel == "edit" && !self.0.load(Ordering::Relaxed)
    }
    fn send(&self, _: &str, _: Option<&str>, _: &[WriteArg]) -> Result<WriteDone, WriteRefused> {
        Ok(WriteDone::default())
    }
}

fn input(media_type: &str, state: &str) -> FormInput {
    FormInput {
        form: "single-record@1.1.0".to_owned(),
        rows: vec![FormRow::of(
            "page-1",
            &[
                ("label", "README.md"),
                ("content", PAGE),
                ("media-type", media_type),
                ("state", state),
            ],
        )],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn say(key: &str) -> String {
    match key {
        "code-editor.language" => "Language: {media-type}".to_owned(),
        "code-editor.preview" => "Preview".to_owned(),
        "code-editor.save" => "Save".to_owned(),
        "code-editor.discard" => "Discard".to_owned(),
        "code-editor.readonly.archived" => {
            "This page is archived and cannot be changed.".to_owned()
        }
        other => other.to_owned(),
    }
}

/// Draw one scene into `path` (child process).
fn draw_scene(scene: &str, path: &std::path::Path) -> Result<(), RenderError> {
    let locked_from_start = scene == "locked";
    let host = Arc::new(Host(AtomicBool::new(locked_from_start)));
    let (w, h) = SIZE;
    let canvas = Canvas::new(w, h, fs_components_workbenches::styles()).with_window_styles();
    let scene = scene.to_owned();
    draw_to_png(
        &canvas,
        move |w, h| {
            let write: Arc<dyn FormWrite> = host.clone();
            let now = if locked_from_start { "archived" } else { "" };
            let first = if scene == "switched" {
                "text/x-nothing"
            } else {
                "text/markdown"
            };
            let ctx = DrawCtx::new(
                fs_components_core::form_view::PlaceInfo::default(),
                (StyleId::Admin, false),
                Arc::new(say),
                Locale::english(),
                write,
                Arc::new(move || input("text/markdown", now)),
            );
            let mounted = View.mount(input(first, now), ctx);
            match scene.as_str() {
                "switched" => mounted.refresh(input("text/markdown", "")),
                "revoked" => {
                    host.0.store(true, Ordering::Relaxed);
                    mounted.refresh(input("text/markdown", "archived"));
                }
                _ => {}
            }
            // the host gives the ground of the window; a picture of the widget alone has none, so it gets the theme ground
            blinc_layout::div::div()
                .w(w)
                .h(h)
                .bg(blinc_theme::ThemeState::get().color(blinc_theme::ColorToken::Background))
                .child(mounted.build(w, h))
        },
        || {},
        path,
    )
}

/// The child: draws the scene named by `FS_E5_SCENE` into `FS_E5_FILE`; without them it does nothing.
#[test]
fn child_draws_one_scene() {
    let (Some(scene), Some(file)) = (
        std::env::var_os("FS_E5_SCENE"),
        std::env::var_os("FS_E5_FILE"),
    ) else {
        return;
    };
    match draw_scene(&scene.to_string_lossy(), &PathBuf::from(file)) {
        Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
        other => other.unwrap(),
    }
}

fn shots() -> PathBuf {
    let dir = std::env::var_os("FS_COMP_SHOTS")
        .map_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")), PathBuf::from);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Draw `scene` in `scheme` by starting this binary again; `None` without graphics.
fn picture(scene: &str, scheme: &str) -> Option<image::RgbaImage> {
    let file = shots().join(format!("code-editor-{scene}-{scheme}.png"));
    let _ = std::fs::remove_file(&file);
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_draws_one_scene", "--nocapture"])
        .env("FS_RENDER_SCHEME", scheme)
        .env("FS_E5_SCENE", scene)
        .env("FS_E5_FILE", &file)
        .status()
        .unwrap();
    assert!(status.success(), "{scene} {scheme}: the child failed");
    if !file.exists() {
        return None;
    }
    Some(image::open(&file).unwrap().to_rgba8())
}

fn luma(p: [u8; 4]) -> u32 {
    (u32::from(p[0]) * 3 + u32::from(p[1]) * 6 + u32::from(p[2])) / 10
}

/// The mean brightness of the left part (the text) of the picture.
fn ground(img: &image::RgbaImage) -> u32 {
    let (w, h) = img.dimensions();
    let (mut sum, mut n) = (0u64, 0u64);
    for y in (0..h).step_by(4) {
        for x in (0..w / 3).step_by(4) {
            sum += u64::from(luma(img.get_pixel(x, y).0));
            n += 1;
        }
    }
    (sum / n) as u32
}

/// Pixels with a clear colour (not grey): the coloured words of the text.
fn coloured(img: &image::RgbaImage) -> usize {
    img.pixels()
        .filter(|p| {
            let [r, g, b, a] = p.0;
            a > 200 && r.max(g).max(b) - r.min(g).min(b) > 70
        })
        .count()
}

/// The pixels that differ between two pictures of one size.
fn differing(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    assert_eq!(a.dimensions(), b.dimensions());
    a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count()
}

#[test]
fn the_pages_are_drawn_light_and_dark_and_the_variants_differ() {
    let Some(split_light) = picture("split", "light") else {
        eprintln!("skipped: no graphics");
        return;
    };
    let split_dark = picture("split", "dark").unwrap();
    assert!(
        ground(&split_dark) + 60 < ground(&split_light),
        "dark is not dark"
    );
    if cfg!(feature = "syntect") {
        assert!(coloured(&split_light) > 300, "the text has colours");
        assert!(coloured(&split_dark) > 300, "the text has colours (dark)");
    }
    let locked = picture("locked", "light").unwrap();
    let locked_dark = picture("locked", "dark").unwrap();
    assert!(
        ground(&locked) > 200,
        "the locked text is on the light ground of the theme, not on a dark one"
    );
    assert!(ground(&locked_dark) < 120);
    assert!(
        differing(&locked, &split_light) > 500,
        "locked looks different (reason, buttons)"
    );
    assert!(differing(&locked_dark, &split_dark) > 500);
    // a withdrawn `can` after a reading is drawn like a text that was locked from the start
    let revoked = picture("revoked", "light").unwrap();
    assert!(
        differing(&revoked, &split_light) > 500,
        "the withdrawn right shows"
    );
    assert!(differing(&revoked, &locked) < differing(&revoked, &split_light));
    // a new media type after a reading (no preview before, a preview after) is the page of the new type
    let switched = picture("switched", "light").unwrap();
    assert!(
        differing(&switched, &split_light) < differing(&switched, &revoked),
        "the page after the switch is the Markdown page with its preview"
    );
}

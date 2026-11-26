//! Pictures of `detail-page` (people B): the page of one person read from `single-record@1.2.0`, a head with the round face (initials),
//! the fields as cards, TWO e-mails as two cards under one label, a field without a value drawn not at all; light and dark. Blinc keeps one
//! theme per process, so each picture is drawn by this same test binary started again with `FS_RENDER_SCHEME` set. Pictures stay local:
//! `FS_COMP_SHOTS` names the folder, else the target folder of cargo; they are never part of the repository.
//!
//! Tests: pixel probes on `draw_to_png` of the mounted `View` (no window; skipped without graphics).
//!
//! Mutants (each must turn a test red):
//! - the second e-mail is dropped                   -> `two_emails_draw_more_than_one_and_the_dark_page_is_dark`
//! - the dark scheme draws on the light ground      -> `two_emails_draw_more_than_one_and_the_dark_page_is_dark`

#![allow(clippy::cast_possible_truncation)] // pixel arithmetic in a test

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_styles::StyleId;
use fs_components_widgets::detail_page::form_view::{FIELDS, View};
use fs_components_words::Locale;

const SIZE: (u32, u32) = (640, 720);

fn field(name: &str, block: &str, label: &str) -> FieldInfo {
    FieldInfo {
        name: name.to_owned(),
        block: block.to_owned(),
        label: label.to_owned(),
    }
}

fn input(emails: &[&str]) -> FormInput {
    let map = vec![(
        FIELDS.to_owned(),
        ["organizations", "emails", "phones", "birthday"]
            .map(str::to_owned)
            .to_vec(),
    )];
    let values = emails
        .iter()
        .map(|e| ("emails", *e))
        .chain([("phones", "+234 1 555 0142"), ("birthday", "1988-03-14")]);
    let row = FormRow::from_values("amara", Vec::new(), &map, values)
        .with_values("label", &[("name", "Amara Okafor")])
        .with_values("summary", &[("nickname", "Engineer, Lagos")]);
    FormInput {
        form: "single-record@1.2.0".to_owned(),
        rows: vec![row],
        channels: vec![ChannelInfo {
            name: FIELDS.to_owned(),
            fields: vec![
                // `organizations` has no value: no card, nothing slides into its place
                field("organizations", "field-text", "person.organizations"),
                field("emails", "field-email", "person.emails"),
                field("phones", "field-phone", "person.phones"),
                field("birthday", "field-date", "person.birthday"),
            ],
            choices: Vec::new(),
        }],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn say(key: &str) -> String {
    match key {
        "person.organizations" => "Organization".to_owned(),
        "person.emails" => "E-mail".to_owned(),
        "person.phones" => "Phone".to_owned(),
        "person.birthday" => "Birthday".to_owned(),
        other => other.to_owned(),
    }
}

fn draw_scene(scene: &str, path: &std::path::Path) -> Result<(), RenderError> {
    let emails: Vec<&str> = if scene == "one-email" {
        vec!["amara@example.org"]
    } else {
        vec!["amara@example.org", "a.okafor@work.example"]
    };
    let (w, h) = SIZE;
    let canvas = Canvas::new(w, h, fs_components_workbenches::styles()).with_window_styles();
    let data = input(&emails);
    draw_to_png(
        &canvas,
        move |w, h| {
            let ctx = DrawCtx::new(
                fs_components_core::form_view::PlaceInfo::default(),
                (StyleId::Admin, false),
                Arc::new(say),
                Locale::english(),
                Arc::new(NoWrite),
                Arc::new(|| input(&[])),
            );
            let mounted = View.mount(data.clone(), ctx);
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

/// The child: draws the scene named by `FS_DP_SCENE` into `FS_DP_FILE`; without them it does nothing.
#[test]
fn child_draws_one_scene() {
    let (Some(scene), Some(file)) = (
        std::env::var_os("FS_DP_SCENE"),
        std::env::var_os("FS_DP_FILE"),
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

fn picture(scene: &str, scheme: &str) -> Option<image::RgbaImage> {
    let file = shots().join(format!("detail-page-{scene}-{scheme}.png"));
    let _ = std::fs::remove_file(&file);
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_draws_one_scene", "--nocapture"])
        .env("FS_RENDER_SCHEME", scheme)
        .env("FS_DP_SCENE", scene)
        .env("FS_DP_FILE", &file)
        .status()
        .unwrap();
    assert!(status.success(), "{scene} {scheme}: the child failed");
    file.exists()
        .then(|| image::open(&file).unwrap().to_rgba8())
}

fn luma(p: [u8; 4]) -> u32 {
    (u32::from(p[0]) * 3 + u32::from(p[1]) * 6 + u32::from(p[2])) / 10
}

fn ground(img: &image::RgbaImage) -> u32 {
    let (w, h) = img.dimensions();
    let (mut sum, mut n) = (0u64, 0u64);
    for y in (0..h).step_by(4) {
        for x in (0..w / 8).step_by(4) {
            sum += u64::from(luma(img.get_pixel(x, y).0));
            n += 1;
        }
    }
    (sum / n) as u32
}

/// The pixels that are not the ground of the picture.
fn ink(img: &image::RgbaImage) -> usize {
    let g = ground(img);
    img.pixels().filter(|p| luma(p.0).abs_diff(g) > 12).count()
}

#[test]
fn two_emails_draw_more_than_one_and_the_dark_page_is_dark() {
    let Some(two) = picture("two-emails", "light") else {
        eprintln!("skipped: no graphics");
        return;
    };
    let one = picture("one-email", "light").unwrap();
    let two_dark = picture("two-emails", "dark").unwrap();
    let _ = picture("one-email", "dark");
    assert!(ink(&two) > 2000, "the page is drawn");
    assert!(
        ink(&two) > ink(&one) + 1500,
        "the second e-mail is a card of its own: {} against {}",
        ink(&two),
        ink(&one)
    );
    assert!(ground(&two_dark) + 80 < ground(&two), "dark is not dark");
}

//! `AppWidget::play`: a described input moves a card of the kanban board inside the shell, then the picture is drawn.
//!
//! Pattern: none (tests). Needs a graphics adapter (software Vulkan on the runner); without one each test says so and returns.
//! The board element has a mouse drag (4 px rule): the input presses the card, moves over the target column and releases there.
#![cfg(feature = "play")]
#![allow(clippy::pedantic)]

#[path = "../../fs-components-kanban/tests/board_host/mod.rs"]
mod host;

use std::path::Path;
use std::sync::Arc;

use fs_components_app::replay::{Code, Plan, parse};
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};
use fs_components_kanban::MemoryBoard;
use fs_components_render::RenderError;
use fs_components_styles::StyleId;
use host::{H, W, board, cards_in, drag, host, last};

const ELEMENT: &str = "element-6066-11e4-a52e-4f735466cecf";
const SIZE: (u32, u32) = (900, 700);

/// The board of the test host as the screen of a shell; the host keeps the cards, as a program keeps its records.
struct Board(Arc<MemoryBoard>);

impl Surface for Board {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(board(&self.0, StyleId::Apple).build(width, height))
    }
    fn styles(&self) -> String {
        fs_components_kanban::styles()
    }
}

fn shell(h: &Arc<MemoryBoard>) -> AppWidget {
    fs_components_widgets::live::set_min_gap_ms(0);
    AppWidget::new(
        "Board",
        vec![Screen::new("b", "Board", Arc::new(Board(h.clone())))],
    )
    .unwrap()
}

fn click(id: &str) -> String {
    format!(
        r#"{{"type":"pointerMove","origin":{{"{ELEMENT}":"{id}"}},"x":0,"y":0}},{{"type":"pointerDown","button":0}},{{"type":"pointerUp","button":0}},{{"type":"pause","duration":100}}"#
    )
}

fn plan(actions: &str) -> Plan {
    parse(&format!(
        r#"{{"actions":[{{"type":"pointer","id":"mouse","parameters":{{"pointerType":"mouse"}},"actions":[{actions}]}}]}}"#
    ))
    .expect("valid")
}

fn out(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("fs-app-play-{}-{name}.png", std::process::id()))
}

/// `None` when there is no graphics adapter.
fn play(app: &AppWidget, plan: &Plan, path: &Path) -> Option<Result<(), AppError>> {
    match app.play(plan, SIZE.0, SIZE.1, path) {
        Err(AppError::Picture(RenderError::NoGraphics(why))) => {
            eprintln!("skipped: {why}");
            None
        }
        other => Some(other),
    }
}

#[test]
fn a_played_input_moves_the_card_and_the_picture_shows_it_in_the_new_column() {
    let _ = (W, H);
    let (before, after) = (out("before"), out("after"));
    let h = host();
    let app = shell(&h);
    let empty = parse(r#"{"actions":[]}"#).unwrap();
    let Some(r) = play(&app, &empty, &before) else {
        return;
    };
    r.unwrap();
    assert_eq!(
        cards_in(&h, "todo"),
        ["a", "b"],
        "an empty plan changes nothing"
    );

    let moves = plan(&drag("B-card-a", "B-col-doing"));
    play(&app, &moves, &after).unwrap().unwrap();
    assert_eq!(last(&h), "moved:a:todo>doing@^");
    assert_eq!(cards_in(&h, "doing"), ["a"]);

    // The risk of the draft: the surface must read the records again before the picture. The picture of the shell that was
    // played equals the picture of a shell whose host held the card in "doing" from the start; and it differs from the first.
    let h2 = host();
    let mut cols = h2.snapshot();
    let a = cols[0].cards.remove(0);
    cols[1].cards.push(a);
    h2.put(cols);
    let fresh = out("fresh");
    play(&shell(&h2), &empty, &fresh).unwrap().unwrap();
    if let Some(d) = std::env::var_os("FS_COMP_SHOTS") {
        for p in [&before, &after, &fresh] {
            std::fs::copy(p, std::path::Path::new(&d).join(p.file_name().unwrap())).unwrap();
        }
    }
    let (b, a, f) = (load(&before), load(&after), load(&fresh));
    assert_ne!(
        b, a,
        "the picture after the input still shows the old state"
    );
    // The played picture is not pixel-equal to the fresh one (the played one has the line "Moved - Undo" under the board, which shifts
    // the heads by a pixel), so the played picture is first compared with the one before the move: the columns "to do" and "doing"
    // change, and the line under the board is there. Then only the columns of the card (0..300, rows <590) are compared with the fresh one.
    let same = |x: &image::RgbaImage, y: &image::RgbaImage, cols: std::ops::Range<u32>| {
        cols.clone()
            .all(|c| (0..590).all(|r| x.get_pixel(c, r) == y.get_pixel(c, r)))
    };
    assert!(
        !same(&a, &b, 320..608),
        "the column doing still shows no card"
    );
    assert!(
        (600..700).any(|r| (0..900).any(|c| a.get_pixel(c, r) != b.get_pixel(c, r))),
        "the line under the board does not say the card was moved"
    );
    assert!(
        !same(&a, &b, 0..300),
        "the column to do still shows the card that moved"
    );
    // The surface read the records again: the columns of the played picture equal those of a shell that held the card in "doing" from the start.
    assert!(
        same(&a, &f, 0..300),
        "the played picture differs from the fresh one in the column to do"
    );
    for p in [before, after, fresh] {
        std::fs::remove_file(p).unwrap();
    }
}

fn load(p: &Path) -> image::RgbaImage {
    image::open(p).unwrap().to_rgba8()
}

#[test]
fn a_missing_element_and_a_key_release_are_errors_with_a_code_and_no_picture() {
    let h = host();
    let app = shell(&h);
    let path = out("never");
    let Some(r) = play(&app, &plan(&click("B-card-nope")), &path) else {
        return;
    };
    let e = r.unwrap_err();
    assert!(
        matches!(&e, AppError::Replay(r) if r.code == Code::ElementMissing),
        "{e}"
    );
    assert!(e.to_string().contains("steps-element-missing"));
    let up = parse(r#"{"actions":[{"type":"key","id":"k","actions":[{"type":"keyDown","value":"a"},{"type":"keyUp","value":"a"}]}]}"#).unwrap();
    let e = app.play(&up, SIZE.0, SIZE.1, &path).unwrap_err();
    assert!(
        matches!(&e, AppError::Replay(r) if r.code == Code::KeyUp),
        "{e}"
    );
    assert!(!path.exists(), "a refused plan draws nothing");
    assert_eq!(last(&h), "");
}

#[test]
fn the_picture_is_drawn_after_the_rebuild_even_when_the_input_ends_on_the_release() {
    let h = host();
    let app = shell(&h);
    let path = out("no-pause");
    let moves =
        plan(&drag("B-card-a", "B-col-doing").replace(r#",{"type":"pause","duration":100}"#, ""));
    let Some(r) = play(&app, &moves, &path) else {
        return;
    };
    r.unwrap();
    assert_eq!(cards_in(&h, "doing"), ["a"]);
    // "To do" has one card left: the card of the host is not drawn there any more.
    let after = load(&path);
    std::fs::remove_file(&path).unwrap();
    let before = out("no-pause-before");
    let h2 = host();
    play(&shell(&h2), &parse(r#"{"actions":[]}"#).unwrap(), &before)
        .unwrap()
        .unwrap();
    let b = load(&before);
    std::fs::remove_file(&before).unwrap();
    assert!(
        (0..300).any(|c| (0..after.height()).any(|r| after.get_pixel(c, r) != b.get_pixel(c, r))),
        "the picture shows the state before the move"
    );
}

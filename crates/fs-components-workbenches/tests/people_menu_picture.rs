//! The context menu of the people workbench as pictures: the real shell (`fs-components-app`) opens it from the lines of the workbench,
//! on a person, on empty space and as the menu of the list; light and dark.
//!
//! Tests: pixel tests, skipped with a message without a graphics adapter. The program's part is played by a shell and a hook; the
//! workbench only says which lines and on what. With `FS_COMP_SHOTS=<dir>` the pictures are kept
//! (`people-menu-<scene>-<light|dark>.png`); `FS_RENDER_SCHEME=dark` draws the dark scheme.
//!
//! Mutants (each must turn a test red):
//! - the menu of a person has no lines (hook never handed the items) -> `the_menus_are_drawn_with_their_lines`
//! - the lines of the list menu are those of a person               -> `the_menus_are_drawn_with_their_lines`

#[path = "people_common/mod.rs"]
mod people_common;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use blinc_core::reactive::{global_graph, set_stateful_deps_notifier};
use blinc_core::{BlincContextState, HookState};
use blinc_layout::click_outside;
use blinc_layout::div::{Div, div};
use blinc_layout::overlay_state::overlay_stack;
use fs_components_app::commands::{KeyNames, Platform, Point, Resolved, Target, Trigger};
use fs_components_app::{AppWidget, Element, Menu, MenuEntry, ModelTexts, Screen, Surface};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_workbenches::people::{PeopleCapabilities, PeopleLabels, PeopleWidget};
use people_common::Book;

const SIZE: (u32, u32) = (900, 600);

struct Face(PeopleWidget);

impl Surface for Face {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(self.0.build(width, height))
    }
    fn styles(&self) -> String {
        self.0.styles()
    }
}

fn scheme() -> &'static str {
    if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        "dark"
    } else {
        "light"
    }
}

/// What the window composes: the program's element and the overlay stack above it.
fn composed(app: &AppWidget, w: f32, h: f32) -> Div {
    let layer = {
        let stack = overlay_stack();
        let mut stack = stack.lock().unwrap();
        stack.set_viewport_with_scale(w, h, 1.0);
        stack.build_overlay_layer()
    };
    div().w(w).h(h).relative().child(app.build(w, h)).child(layer)
}

fn init(styles: &str) {
    let bundle = fs_components_render::bundle(styles);
    blinc_theme::ThemeState::init(bundle, blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(AtomicBool::new(false)),
        );
    }
    set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    blinc_layout::init_text_measurer();
    click_outside::clear_click_outside_handlers();
    overlay_stack().lock().unwrap().close_all();
    overlay_stack().lock().unwrap().update(u64::MAX / 2);
}

fn shoot(app: &AppWidget, scene: &str) -> Option<image::RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-people-menu-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("people-menu-{scene}-{}.png", scheme()));
    let canvas = Canvas::new(SIZE.0, SIZE.1, app.styles());
    match draw_to_png(&canvas, |w, h| composed(app, w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
            Some(picture)
        }
    }
}

fn open(widget: &PeopleWidget, app: &AppWidget, items: &[Resolved], at: (f32, f32), ids: &[&str]) {
    let names = KeyNames::english();
    let say = |key: &fs_components_app::commands::TextKey| widget.say(key);
    let texts = ModelTexts {
        say: &say,
        platform: Platform::Linux,
        names: &names,
    };
    let target = Target {
        kind: "person".into(),
        ids: ids.iter().map(|i| (*i).to_owned()).collect(),
    };
    let trigger = Trigger::Pointer {
        at: Point { x: at.0, y: at.1 },
        shift: false,
    };
    app.open_context_menu(&trigger, target, items, &texts);
}

#[test]
fn the_menus_are_drawn_with_their_lines() {
    let _one = people_common::harness::serial();
    let mut book = Book::with(&["ada", "bob", "clara"]);
    book.capabilities = PeopleCapabilities::all().with_export().with_merge();
    *book.groups.lock().unwrap() = vec![("team".into(), "Team".into(), vec!["bob".into()])];
    let widget = PeopleWidget::new(Arc::new(book), PeopleLabels::default());
    let app = AppWidget::new("Contacts", vec![Screen::new("a", "Contacts", Arc::new(Face(widget.clone())))]).unwrap();
    app.set_menus(vec![Menu::new("Menu", vec![MenuEntry::new("noop", "Nothing")])], |_| {})
        .unwrap();
    init(&app.styles());
    let person = widget.with_state(|s| s.person_menu());
    let list = widget.with_state(|s| s.list_menu());
    let empty = widget.with_state(|s| s.empty_menu());
    assert!(!person.is_empty() && !list.is_empty() && !empty.is_empty());
    assert_ne!(person, list, "the lines of a person are not the lines of the list");

    open(&widget, &app, &person, (200.0, 190.0), &["ada"]);
    let Some(on_person) = shoot(&app, "person") else {
        return;
    };
    open(&widget, &app, &list, (140.0, 60.0), &[]);
    let on_list = shoot(&app, "list").unwrap();
    open(&widget, &app, &empty, (200.0, 480.0), &[]);
    let on_empty = shoot(&app, "empty").unwrap();
    app.close_context_menu();
    let none = shoot(&app, "closed").unwrap();
    for (a, b, what) in [
        (&on_person, &none, "a menu on a person"),
        (&on_list, &none, "the menu of the list"),
        (&on_empty, &none, "a menu on empty space"),
        (&on_person, &on_list, "person and list"),
    ] {
        assert_ne!(a, b, "{what} differs from the picture without it");
    }
}

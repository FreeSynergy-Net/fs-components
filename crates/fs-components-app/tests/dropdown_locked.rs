//! A locked line of a dropdown (a command that is off for now, or not finished yet) keeps the height of a free line, is
//! `aria-disabled` (never the HTML attribute), names its reason as a tooltip, and a press on it runs nothing and keeps the panel
//! open (ENTWURF-unfertig-tooltip G1/G2, the view test of the dropdown that U3 left open).
//!
//! Tests: layout and click path of Blinc's overlay stack, as the window drives it (see `menu_common`).

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use blinc_layout::div::div;
use fs_components_app::{AppWidget, Element, Screen, Surface};
use fs_components_commands::{Arrangement, Command, Commands, MenuItem, resolve};
use fs_components_widgets::dropdown::{Align, dropdown};
use fs_components_words::Locale;
use menu_common::Harness;

/// A dropdown of three command lines: free, off for now, not finished yet (German reasons' language: `de`).
struct Locked(Arc<AtomicUsize>);

impl Surface for Locked {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let ran = Arc::clone(&self.0);
        let list = Commands::new()
            .with(Command::new("open", "t-open"))
            .with(Command::new("save", "t-save").disabled("t-why"))
            .with(Command::new("zip", "t-zip").unsupported("t-soon"));
        let tree = Arrangement::new(
            "m",
            vec![
                MenuItem::command("open"),
                MenuItem::command("save"),
                MenuItem::command("zip"),
            ],
        );
        let items = resolve(&tree, &list).unwrap();
        let menu = dropdown(|_| div().id("anchor").w(60.0).h(28.0))
            .align(Align::Start)
            .width(220.0)
            .locale(&Locale::parse("de").unwrap())
            .commands(&items, &|key| key.as_str().to_owned(), move |_| {
                ran.fetch_add(1, Ordering::SeqCst);
            });
        Box::new(
            div().w(width).h(height).relative().child(
                div()
                    .absolute()
                    .left(20.0)
                    .top(20.0)
                    .child(menu.into_element()),
            ),
        )
    }
}

fn open() -> (Harness, Arc<AtomicUsize>) {
    let ran = Arc::new(AtomicUsize::new(0));
    let app = AppWidget::new(
        "Title",
        vec![Screen::new("a", "A", Arc::new(Locked(Arc::clone(&ran))))],
    )
    .unwrap();
    let mut h = Harness::new(app, 800.0, 500.0);
    h.click("anchor");
    (h, ran)
}

/// The id of the open panel (`fs-dropdown-panel-<n>`).
fn panel_id(h: &Harness) -> String {
    let registry = h.tree.element_registry();
    h.tree
        .iter_nodes()
        .find_map(|(node, _)| {
            registry
                .get_id(node)
                .filter(|id| id.starts_with("fs-dropdown-panel-"))
        })
        .expect("the panel is open")
}

#[test]
fn g1_a_locked_line_has_the_height_of_a_free_one_and_the_panel_is_not_taller() {
    let _g = menu_common::serial();
    let (h, _) = open();
    let panel = panel_id(&h);
    for n in [1, 2] {
        let (_, _, _, line) = h.bounds(&format!("{panel}-line-{n}"));
        assert!((line - 32.0).abs() < 0.5, "line {n} is {line} px high");
    }
    let (_, _, _, whole) = h.bounds(&panel);
    assert!(
        (whole - (3.0 * 32.0 + 2.0)).abs() < 0.5,
        "no extra line: {whole}"
    );
}

#[test]
fn g2_a_locked_line_is_aria_disabled_names_its_reason_and_a_press_runs_nothing() {
    let _g = menu_common::serial();
    let (mut h, ran) = open();
    let panel = panel_id(&h);
    for n in [1, 2] {
        let id = format!("{panel}-line-{n}");
        assert_eq!(h.attr(&id, "aria-disabled").as_deref(), Some("true"));
        assert_eq!(h.attr(&id, "disabled"), None, "never the HTML attribute");
        assert_eq!(
            h.attr(&id, "aria-describedby").as_deref(),
            Some(format!("{id}-why-bubble").as_str())
        );
        assert_eq!(
            h.attr(&id, "tabindex").as_deref(),
            Some("-1"),
            "roving, no tab stop"
        );
        h.press_at(h.centre(&id));
        h.frame();
        assert_eq!(ran.load(Ordering::SeqCst), 0, "a locked line runs nothing");
        assert!(h.has(&panel), "the panel stays open");
        assert_eq!(
            h.attr(&format!("{id}-why-bubble"), "role").as_deref(),
            Some("tooltip"),
            "the press opens the reason"
        );
    }
}

//! The Blinc parts build: the button in its states and the path field view, with the words in two languages.
//!
//! Pattern: Smoke tests over `build()` (the element tree is made and laid out; Blinc's singletons are set up as a window does).
//! The look is NOT judged here (pictures come with the frontends). Mutants: `build` panics; the provisional marker not shown
//! for the path field (the tree has no `-provisional` part); the problem not shown after a refused path.
#![cfg(feature = "ui")]

use std::sync::{Arc, Mutex};

use blinc_core::BlincContextState;
use blinc_core::HookState;
use blinc_core::reactive::global_graph;
use blinc_layout::div::div;
use blinc_layout::prelude::RenderTree;
use fs_components_filepicker::ui::{FilePickButton, PathPromptView, STYLES};
use fs_components_filepicker::{
    Answer, FakeChooser, Labels, Mode, OpenRequest, PathFieldChooser, PathPrompt,
};

static ONE: Mutex<()> = Mutex::new(());

fn init() -> std::sync::MutexGuard<'static, ()> {
    let guard = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    fs_components_widgets::live::set_min_gap_ms(0);
    blinc_theme::ThemeState::init(blinc_cn::cn_bundle(), blinc_theme::ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        );
    }
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    guard
}

fn laid_out(ui: impl blinc_layout::prelude::ElementBuilder + 'static) -> RenderTree {
    blinc_layout::init_text_measurer();
    let page = div().w(480.0).h(320.0).child(ui);
    let mut tree = RenderTree::from_element(&page);
    tree.compute_layout(480.0, 320.0);
    tree
}

fn request() -> OpenRequest {
    OpenRequest::new(Mode::OpenFile, &Labels::default())
}

#[test]
fn the_styles_are_only_theme_variables() {
    assert!(STYLES.contains(".fs-fp"));
    assert!(
        !STYLES.contains('#'),
        "no colour literal: the style supplies the variables"
    );
}

#[test]
fn the_button_builds_in_every_phase() {
    let _g = init();
    let fake = FakeChooser::new().answer(Answer::Cancel);
    let button = FilePickButton::new(&Labels::default(), Arc::new(fake), request()).id("pick");
    let _tree = laid_out(button.build());
    assert!(button.activate());
    let _tree = laid_out(button.build());
}

#[test]
fn the_button_of_the_path_field_is_marked_provisional() {
    let _g = init();
    let prompt = PathPrompt::new();
    let button = FilePickButton::new(
        &Labels::default(),
        Arc::new(PathFieldChooser::new(&prompt)),
        request(),
    )
    .id("pick");
    let tree = laid_out(button.build());
    assert!(
        tree.query_by_id("pick-provisional").is_some(),
        "the marker is in the tree of a button over the path field"
    );
}

#[test]
fn the_path_view_builds_with_and_without_a_question() {
    let _g = init();
    let prompt = PathPrompt::new();
    let view = PathPromptView::new(&Labels::default(), &prompt).id("pp");
    let _empty = laid_out(view.build());
    assert!(!view.confirm(), "no question is open: nothing is taken");
    let chooser = PathFieldChooser::new(&prompt);
    let asker = std::thread::spawn(move || {
        fs_components_filepicker::FileChooser::open(&chooser, request())
    });
    while prompt.pending().is_none() {
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let open_view = laid_out(view.build());
    assert!(
        open_view.query_by_id("pp-provisional").is_some(),
        "the path field is marked provisional"
    );
    assert!(
        open_view.query_by_id("pp-problem").is_none(),
        "no problem before a path was refused"
    );
    assert!(!view.confirm(), "the empty field is refused");
    assert!(prompt.problem().is_some());
    let refused = laid_out(view.build());
    assert!(
        refused.query_by_id("pp-problem").is_some(),
        "the refused path shows its problem"
    );
    view.cancel();
    assert!(asker.join().unwrap().unwrap_err().is_cancelled());
}

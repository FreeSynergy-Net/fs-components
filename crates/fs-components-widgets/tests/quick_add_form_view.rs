//! The form view of `quick-add` (`checklist@^1.1.0`): one line makes one `create`, three lines ask "3 entries?" and then make
//! exactly three, a refusal gives the text back, and without `create` in `may` the row is locked and says why.
//!
//! Pattern: Test double (a recording host) over a mounted row.

#![allow(clippy::pedantic)]

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, background, draw, ink, layout_with, serial, set_dark};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::{
    FormWrite, NoWrite, WriteArg, WriteDone, WriteRefused, WriteValue,
};
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::plan::STYLES;
use fs_components_widgets::quick_add::form_view::{QuickForm, View, WIDGET};
use fs_components_words::Locale;

type Log = Arc<Mutex<Vec<String>>>;

/// Records every `create`; refuses the title `bad` with the code `full`.
struct Host(Log);

impl FormWrite for Host {
    fn can(&self, channel: &str, record: Option<&str>) -> bool {
        channel == "create" && record.is_none()
    }

    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        let Some(WriteValue::Text(t)) = args.first().map(|a| a.value.clone()) else {
            return Err(WriteRefused::new("no-label"));
        };
        assert!(record.is_none() && args[0].name == "label");
        if t == "bad" {
            return Err(WriteRefused::new("full"));
        }
        self.0.lock().unwrap().push(format!("{channel} {t}"));
        Ok(WriteDone::default())
    }
}

fn input(may: &[&str]) -> FormInput {
    FormInput {
        form: "checklist@1.1.0".to_owned(),
        rows: Vec::new(),
        channels: Vec::new(),
        may: may.iter().map(|m| (*m).to_owned()).collect(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(log: &Log, write: Arc<dyn FormWrite>) -> DrawCtx {
    let _ = log;
    DrawCtx {
        place: fs_components_core::form_view::PlaceInfo::default(),
        look: (StyleId::Admin, false),
        say: Arc::new(|k| k.to_owned()),
        locale: Locale::english(),
        write,
        reread: Arc::new(|| input(&["create"])),
    }
}

fn form(log: &Log) -> QuickForm {
    QuickForm::new(
        &input(&["create"]),
        ctx(log, Arc::new(Host(Arc::clone(log)))),
    )
}

#[test]
fn the_view_is_the_widget_of_the_block() {
    assert_eq!(View.widget(), WIDGET);
    assert_eq!(WIDGET, "QuickAdd");
}

#[test]
fn one_line_makes_one_create_and_blank_nothing() {
    let log = Log::default();
    let f = form(&log);
    f.enter("  Write letter ", false);
    f.enter("   ", false);
    assert_eq!(*log.lock().unwrap(), ["create Write letter"]);
}

#[test]
fn three_lines_ask_first_and_then_make_exactly_three() {
    let _g = serial();
    let log = Log::default();
    let f = form(&log);
    f.enter("a\nb\nc", false);
    assert!(log.lock().unwrap().is_empty(), "nothing before the answer");
    let tree = layout_with(f.build(600.0, 80.0), 600.0, 80.0, STYLES);
    assert!(
        tree.query_by_id("quick-add-ask").is_some(),
        "the question stands"
    );
    f.answer(true);
    assert_eq!(*log.lock().unwrap(), ["create a", "create b", "create c"]);
    f.enter("x\ny", false);
    f.answer(false);
    assert_eq!(log.lock().unwrap().len(), 3, "a no makes nothing");
}

#[test]
fn a_refusal_stops_and_keeps_the_rest() {
    let log = Log::default();
    let f = form(&log);
    f.enter("a\nbad\nc", false);
    f.answer(true);
    assert_eq!(*log.lock().unwrap(), ["create a"]);
}

#[test]
fn without_create_in_may_the_row_is_locked_and_says_why() {
    let _g = serial();
    let log = Log::default();
    let f = QuickForm::new(&input(&[]), ctx(&log, Arc::new(Host(Arc::clone(&log)))));
    f.enter("a", false);
    assert!(log.lock().unwrap().is_empty());
    let tree = layout_with(f.build(600.0, 80.0), 600.0, 80.0, STYLES);
    assert_eq!(
        attr(&tree, "quick-add-locked", "aria-readonly").as_deref(),
        Some("true")
    );
    assert!(tree.query_by_id("quick-add-why").is_some());
    // the host's own no counts as well
    let g = QuickForm::new(&input(&["create"]), ctx(&log, Arc::new(NoWrite)));
    assert!(!g.allowed());
    g.refresh(input(&["create"]));
}

#[test]
fn the_row_draws_in_both_schemes() {
    let _g = serial();
    let log = Log::default();
    for dark in [false, true] {
        set_dark(dark);
        let mounted = View.mount(
            input(&["create"]),
            ctx(&log, Arc::new(Host(Arc::clone(&log)))),
        );
        let name = if dark {
            "quick_add_dark.png"
        } else {
            "quick_add_light.png"
        };
        let img = draw(mounted.build(640.0, 60.0), 640, 60, name);
        assert!(
            ink(&img, (0, 0, 640, 60), background(&img)) > 100,
            "dark={dark}"
        );
    }
    set_dark(false);
}

#[test]
fn the_question_is_answered_from_the_keyboard() {
    let _g = serial();
    let ask = |log: &Log| {
        let f = form(log);
        f.enter("a\nb\nc", false);
        let tree = layout_with(f.build(600.0, 80.0), 600.0, 80.0, STYLES);
        (f, tree)
    };
    // Tab to "Yes", Enter: all three are made
    let log = Log::default();
    let (_f, mut tree) = ask(&log);
    let mut input = Input::new();
    assert!(
        input.tab_to(&mut tree, "quick-add-yes"),
        "Yes is a tab stop"
    );
    input.key(&mut tree, 13);
    assert_eq!(*log.lock().unwrap(), ["create a", "create b", "create c"]);
    // Tab to "No", Space: nothing is made
    let log = Log::default();
    let (_f, mut tree) = ask(&log);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "quick-add-no"), "No is a tab stop");
    input.key(&mut tree, 32);
    assert!(log.lock().unwrap().is_empty(), "No makes nothing");
    // Escape is No, from wherever the keyboard is
    let log = Log::default();
    let (f, mut tree) = ask(&log);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "quick-add-yes"));
    input.key(&mut tree, 27);
    assert!(log.lock().unwrap().is_empty(), "Escape made nothing");
    f.answer(true); // the question is gone: a late answer finds nothing to send
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn the_keyboard_goes_to_yes_and_back_to_the_line() {
    let log = Log::default();
    let asked: Log = Log::default();
    let seen = Arc::clone(&asked);
    let f = form(&log).focus_to(move |id| seen.lock().unwrap().push(id.to_owned()));
    f.enter("a\nb", false);
    f.answer(false);
    assert_eq!(*asked.lock().unwrap(), ["quick-add-yes", "quick-add-entry"]);
}

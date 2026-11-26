//! The form view of `task-row` (`checklist@^1.1.0`): fixed rows (no module) through the registry entry; the commands go out as
//! `toggle` and `open`, a row the host does not allow is locked, the groups become sections and the facts stand in columns.
//!
//! Pattern: Test double (a recording host, a `say` that returns the key) over fixed rows.

#![allow(clippy::pedantic)]

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, layout_with, serial};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused, WriteValue};
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::plan::form_view::{View, WIDGET};
use fs_components_widgets::plan::{STYLES, Tick, read_rows};
use fs_components_words::Locale;

const W: f32 = 700.0;
const H: f32 = 400.0;

type Log = Arc<Mutex<Vec<String>>>;

/// A host that records every call and allows everything except `toggle` on the row `fixed`.
struct Host(Log);

impl FormWrite for Host {
    fn can(&self, channel: &str, record: Option<&str>) -> bool {
        !(channel == "toggle" && record == Some("fixed"))
    }

    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        let arg = args.first().map_or(String::new(), |a| match &a.value {
            WriteValue::Flag(f) => format!(" {}={f}", a.name),
            _ => String::new(),
        });
        self.0
            .lock()
            .unwrap()
            .push(format!("{channel} {}{arg}", record.unwrap_or("-")));
        Ok(WriteDone::default())
    }
}

fn row(key: &str, label: &str, done: &str, group: &str, facts: &[&str], order: &str) -> FormRow {
    let r = FormRow::of(
        key,
        &[
            ("label", label),
            ("done", done),
            ("group", group),
            ("order", order),
        ],
    );
    let pairs: Vec<(&str, &str)> = facts.iter().map(|f| ("facts", *f)).collect();
    r.with_values("facts", &pairs)
}

fn input() -> FormInput {
    FormInput {
        form: "checklist@1.1.0".to_owned(),
        rows: vec![
            row("b", "Pay the bill", "false", "ready", &["High", ""], "2"),
            row("a", "Call the printer", "true", "today", &["", "Tue"], "1"),
            row("fixed", "Fixed by the module", "false", "today", &[], "3"),
            row("n", "No group", "false", "", &["x"], "4"),
        ],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(log: &Log) -> DrawCtx {
    DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k| k.to_owned()),
        Locale::english(),
        Arc::new(Host(Arc::clone(log))),
        Arc::new(input),
    )
}

#[test]
fn the_rows_are_sorted_grouped_and_their_facts_stand_in_columns() {
    let rows = read_rows(&input(), &|k| k.to_owned(), &|k| k != "fixed");
    let titles: Vec<&str> = rows.sections.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(
        titles,
        ["", "today", "ready"],
        "no group first, then by first appearance"
    );
    assert_eq!(
        rows.keys(),
        ["n", "a", "fixed", "b"],
        "by `order`, a group's rows together"
    );
    let lines: Vec<_> = rows.sections.iter().flat_map(|s| &s.lines).collect();
    assert!(
        lines
            .iter()
            .all(|l| l.slots.len() == 2 && l.facts.len() == 2)
    );
    assert!(
        lines.iter().all(|l| l.slots == lines[0].slots),
        "the same columns in every line"
    );
    assert!(lines[0].slots[0] > 0.0 && lines[0].slots[1] > 0.0);
    assert_eq!(lines[2].tick, Tick::Locked);
    assert_eq!(rows.is_done("a"), Some(true));
}

#[test]
fn the_view_is_the_widget_of_the_block() {
    assert_eq!(View.widget(), WIDGET);
    assert_eq!(WIDGET, "TaskRow");
}

#[test]
fn a_click_on_the_box_toggles_a_click_on_the_line_opens_and_a_locked_box_does_nothing() {
    let _g = serial();
    let log = Log::default();
    let mounted = View.mount(input(), ctx(&log));
    let mut tree = layout_with(mounted.build(W, H), W, H, STYLES);
    let mut input = Input::new();
    input.click(&mut tree, "tasks-a-check");
    input.click(&mut tree, "tasks-b");
    input.click(&mut tree, "tasks-fixed-check");
    assert_eq!(
        attr(&tree, "tasks-fixed-head", "aria-readonly").as_deref(),
        Some("true")
    );
    assert_eq!(*log.lock().unwrap(), ["toggle a done=false", "open b"]);
}

#[test]
fn the_cursor_walks_and_space_and_enter_send_the_two_commands() {
    let log = Log::default();
    let mounted = View.mount(input(), ctx(&log));
    assert!(!mounted.key(Key::Space), "no cursor, nothing to do");
    assert!(mounted.key(Key::Down));
    assert!(mounted.key(Key::Down));
    assert!(mounted.key(Key::Space));
    assert!(mounted.key(Key::Enter));
    assert!(mounted.key(Key::End));
    assert!(mounted.key(Key::Space));
    assert!(!mounted.key(Key::Escape));
    assert_eq!(
        *log.lock().unwrap(),
        ["toggle a done=false", "open a", "toggle b done=true"]
    );
}

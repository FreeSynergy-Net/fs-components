//! `tablist` as a view of the form `rows`, over fixed rows (no module): the chosen row, the command `open`, the locked tab.
//!
//! Pattern: Test double (a recording write port that lets `open` through for the rows it is told).
//!
//! Mutants (each must turn a test red):
//! - the row marked `current` is not the chosen tab   -> `the_row_marked_current_is_the_chosen_tab`
//! - a choice sends nothing or twice                  -> `a_key_choice_sends_exactly_one_open_with_the_key_of_the_row`
//! - a locked row can be chosen                       -> `a_row_the_host_does_not_let_open_is_locked_and_sends_nothing`

use std::sync::{Arc, Mutex};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_widgets::tabs::form_view::View;
use fs_components_words::Locale;

/// Records every call; `open` is allowed for the keys in `allowed`.
struct Host {
    allowed: Vec<&'static str>,
    sent: Mutex<Vec<(String, Option<String>)>>,
}

impl FormWrite for Host {
    fn can(&self, channel: &str, record: Option<&str>) -> bool {
        channel == "open" && record.is_some_and(|r| self.allowed.contains(&r))
    }

    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        _args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.sent
            .lock()
            .unwrap()
            .push((channel.to_owned(), record.map(str::to_owned)));
        Ok(WriteDone::default())
    }
}

fn input() -> FormInput {
    FormInput {
        form: "rows@1.0.0".to_owned(),
        rows: vec![
            FormRow::of("list", &[("label", "List")]),
            FormRow::of("kanban", &[("label", "Kanban"), ("state", "current")]),
            FormRow::of("gantt", &[("label", "Gantt")]),
        ],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn mount(allowed: Vec<&'static str>) -> (Box<dyn Mounted>, Arc<Host>) {
    let host = Arc::new(Host {
        allowed,
        sent: Mutex::new(Vec::new()),
    });
    let ctx = DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(str::to_owned),
        Locale::english(),
        host.clone(),
        Arc::new(input),
    );
    (View.mount(input(), ctx), host)
}

#[test]
fn the_view_is_the_widget_tabs() {
    assert_eq!(View.widget(), "Tabs");
}

#[test]
fn the_row_marked_current_is_the_chosen_tab() {
    let (tabs, host) = mount(vec!["list", "kanban", "gantt"]);
    // Right from the chosen "kanban" lands on "gantt": the cursor started at the marked row, not at the first
    assert!(tabs.key(Key::Right));
    let sent = host.sent.lock().unwrap();
    assert_eq!(
        sent.as_slice(),
        [("open".to_owned(), Some("gantt".to_owned()))]
    );
}

#[test]
fn a_key_choice_sends_exactly_one_open_with_the_key_of_the_row() {
    let (tabs, host) = mount(vec!["list", "kanban", "gantt"]);
    assert!(tabs.key(Key::Left));
    assert_eq!(host.sent.lock().unwrap().len(), 1);
    assert_eq!(
        host.sent.lock().unwrap()[0],
        ("open".to_owned(), Some("list".to_owned()))
    );
}

#[test]
fn a_row_the_host_does_not_let_open_is_locked_and_sends_nothing() {
    let (tabs, host) = mount(vec!["kanban"]);
    // both neighbours are locked: the keys skip them, nothing is sent
    let _ = tabs.key(Key::Right);
    let _ = tabs.key(Key::Left);
    assert!(host.sent.lock().unwrap().is_empty());
}

#[test]
fn a_refresh_with_new_rows_keeps_the_view_usable() {
    let (tabs, _host) = mount(vec!["list", "kanban", "gantt"]);
    let mut next = input();
    next.rows.push(FormRow::of("table", &[("label", "Table")]));
    tabs.refresh(next);
    assert!(tabs.key(Key::End));
}

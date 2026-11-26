//! The board as `views()` gives it to the host: through `View::mount`, the real layout and router, a fake host that keeps its
//! rows. A drag shows the card in the new column after the form is read again, and the "..." button of a column reaches the
//! column dialogs (the host has no menu hook).
//!
//! Mutants: the "..." button does nothing without a menu -> `the_column_button_reaches_the_dialogs` and `a_choice_opens_the_dialog_of_that_gesture`;
//! the choice does not open the dialog -> `a_choice_opens_the_dialog_of_that_gesture`; the drag is not sent -> `a_drag_shows_the_card_in_its_new_column`;
//! a button that stays alive without any right -> `without_any_right_the_button_asks_nothing`. (A move shows itself without the host reading
//! the form again, the board answers it: the reread after a COLUMN dialog is held by the unit test in `form/view.rs`.)
//!
//! Pattern: Fake (a host that keeps the rows and the calls) over the real adapter, element and router.
#![allow(clippy::pedantic)]

mod board_common;
#[macro_use]
mod common;

use std::sync::{Arc, Mutex};

use board_common::Rig;
use fs_components_core::form_read::{ChannelInfo, FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused, WriteValue};
use fs_components_core::states::Load;
use fs_components_kanban::View;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

/// A module that keeps the rows: a move changes the column of the card, a rename the title of the column.
struct Host {
    allow: bool,
    now: Mutex<FormInput>,
    sent: Mutex<Vec<String>>,
}

fn text(args: &[WriteArg], name: &str) -> Option<String> {
    args.iter().find_map(|a| match (&a.value, a.name == name) {
        (WriteValue::Text(t), true) => Some(t.clone()),
        _ => None,
    })
}

impl FormWrite for Host {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        self.allow
    }

    gestures_by_channel!();

    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.sent
            .lock()
            .unwrap()
            .push(format!("{channel} {record:?}"));
        let mut now = self.now.lock().unwrap();
        match (channel, record) {
            ("move", Some(card)) => {
                let column = text(args, "column").unwrap_or_default();
                for row in &mut now.rows {
                    if row.get("title") == Some(card) {
                        *row = FormRow::of(card, &[("title", card), ("column", &column)]);
                    }
                }
            }
            ("column.rename", Some(column)) => {
                let name = text(args, "name").unwrap_or_default();
                for choice in &mut now.channels[0].choices {
                    if choice.0 == column {
                        choice.1.clone_from(&name);
                    }
                }
            }
            _ => {}
        }
        Ok(WriteDone::default())
    }
}

fn input() -> FormInput {
    let row = |k: &str| FormRow::of(k, &[("title", k), ("column", "todo")]);
    FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows: vec![row("a"), row("b")],
        channels: vec![ChannelInfo {
            name: "column".into(),
            fields: Vec::new(),
            choices: vec![
                ("todo".into(), "To do".into()),
                ("done".into(), "Done".into()),
            ],
        }],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn rig(allow: bool) -> (Rig, Arc<Host>) {
    let host = Arc::new(Host {
        allow,
        now: Mutex::new(input()),
        sent: Mutex::default(),
    });
    let reread = host.clone();
    let ctx = DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(str::to_owned),
        Locale::english(),
        host.clone(),
        Arc::new(move || reread.now.lock().unwrap().clone()),
    );
    let mounted = View.mount(input(), ctx);
    (Rig::mounted(Arc::from(mounted), 900.0, 640.0), host)
}

#[test]
fn a_drag_shows_the_card_in_its_new_column() {
    let (mut rig, host) = rig(true);
    let (x, y, w, _) = rig.bounds("fs-board-head-done");
    let (before, ..) = rig.bounds("fs-board-card-a");
    assert!(before < x, "the card starts in the first column");
    rig.drag_to("fs-board-card-a", (x + w / 2.0, y + 120.0));
    assert_eq!(host.sent.lock().unwrap().len(), 1, "exactly one write");
    rig.frame();
    let (after, ..) = rig.bounds("fs-board-card-a");
    assert!(
        after >= x && after < x + w,
        "the card is in the column `done`: {after} vs {x}"
    );
}

#[test]
fn the_column_button_reaches_the_dialogs() {
    let (mut rig, _host) = rig(true);
    assert!(!rig.has("column-menu"));
    rig.click("fs-board-menu-todo");
    assert!(
        rig.has("column-menu-opt-rename"),
        "the choice stands over the board"
    );
    assert!(rig.has("column-menu-opt-limit") && rig.has("column-menu-opt-add"));
}

#[test]
fn a_choice_opens_the_dialog_of_that_gesture() {
    let (mut rig, _host) = rig(true);
    rig.click("fs-board-menu-todo");
    rig.click("column-menu-opt-rename");
    rig.click("column-menu-ok");
    assert!(!rig.has("column-menu-opt-rename"), "the choice is gone");
    assert!(
        rig.has("rename-column"),
        "the line for the new name is open"
    );
}

#[test]
fn without_any_right_the_button_asks_nothing() {
    let (mut rig, host) = rig(false);
    rig.click("fs-board-menu-todo");
    assert!(
        !rig.has("column-menu"),
        "without any right the button asks nothing"
    );
    assert!(host.sent.lock().unwrap().is_empty());
}

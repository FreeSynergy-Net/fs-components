//! The adapters under the one board element: the real layout and router, the pointer carries a card, and the host's write port
//! hears exactly one `move`. This is what `views()` draws for the block `board`.
//!
//! Mutants: the adapter is not wired to the element (nothing is sent) -> `a_mouse_drag_sends_one_move_to_the_host`; a wiggle of 3 px
//! sends a move -> `a_wiggle_is_a_click_and_sends_nothing`; `can` is not asked -> `without_the_right_the_drag_sends_nothing`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::{Arc, Mutex};

use board_common::Rig;
use fs_components_core::form_read::{ChannelInfo, FormInput, FormRow};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_core::states::Load;
use fs_components_fields::KeysAsText;
use fs_components_kanban::{BoardBench, Labels};
use fs_components_kanban::{FormEvents, FormSource};

struct Host {
    allow: bool,
    sent: Mutex<Vec<String>>,
}

impl FormWrite for Host {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        self.allow
    }
    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.sent
            .lock()
            .unwrap()
            .push(format!("{channel} {record:?} {args:?}"));
        Ok(WriteDone::default())
    }
}

fn input() -> FormInput {
    let row = |k: &str, c: &str| FormRow::of(k, &[("title", k), ("column", c)]);
    FormInput {
        form: "cards-in-columns@1.2.0".into(),
        rows: vec![row("a", "todo"), row("b", "todo"), row("c", "todo")],
        channels: vec![ChannelInfo {
            name: "column".into(),
            fields: Vec::new(),
            choices: vec![
                ("todo".into(), "To do".into()),
                ("doing".into(), "Doing".into()),
                ("done".into(), "Done".into()),
            ],
        }],
        may: Vec::new(),
        load: Load::Ready,
    }
}

fn rig(allow: bool) -> (Rig, Arc<Host>) {
    let host = Arc::new(Host {
        allow,
        sent: Mutex::default(),
    });
    let say = Arc::new(KeysAsText);
    let source = Arc::new(FormSource::new(input(), host.clone(), say.clone()));
    let events = Arc::new(FormEvents::new(host.clone(), source.clone(), say));
    let bench = BoardBench::new(source, Labels::default())
        .events(events)
        .id("B");
    (Rig::new(bench, 1100.0, 640.0), host)
}

#[test]
fn a_mouse_drag_sends_one_move_to_the_host() {
    let (mut rig, host) = rig(true);
    let (x, y, w, _) = rig.bounds("B-head-done");
    rig.drag_to("B-card-a", (x + w / 2.0, y + 120.0));
    let sent = host.sent.lock().unwrap();
    assert_eq!(sent.len(), 1, "exactly one write: {sent:?}");
    assert!(sent[0].starts_with("move Some(\"a\")"), "{}", sent[0]);
    assert!(sent[0].contains("Text(\"done\")") && !sent[0].contains("after"));
}

#[test]
fn a_wiggle_is_a_click_and_sends_nothing() {
    let (mut rig, host) = rig(true);
    let (x, y) = rig.centre("B-card-c");
    rig.down(x, y);
    rig.move_to(x + 2.0, y + 1.0);
    rig.up(x + 2.0, y + 1.0);
    rig.frame();
    assert!(host.sent.lock().unwrap().is_empty());
}

#[test]
fn without_the_right_the_drag_sends_nothing() {
    let (mut rig, host) = rig(false);
    let (x, y, w, _) = rig.bounds("B-head-done");
    rig.drag_to("B-card-a", (x + w / 2.0, y + 120.0));
    assert!(host.sent.lock().unwrap().is_empty());
}

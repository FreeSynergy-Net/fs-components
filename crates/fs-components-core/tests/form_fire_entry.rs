//! The entry of the slot `menu` at the gesture port: several actions hang at `menu`, each is one entry of the context menu, and the
//! port tells which one was meant. A port that does not know entries refuses every one as unbound.
//!
//! Pattern: none (tests).
use std::sync::Mutex;

use fs_components_core::form_write::{
    FormWrite, NoWrite, WriteArg, WriteDone, WriteRefused, trigger,
};

/// A host with two entries at `menu` ("claim", "release"; "note" carries two actions) and one action at `drop`.
#[derive(Default)]
struct Menus {
    ran: Mutex<Vec<String>>,
}

impl Menus {
    fn actions(trigger: &str, entry: Option<&str>) -> Result<Vec<&'static str>, WriteRefused> {
        match (trigger, entry) {
            (trigger::MENU, Some("claim")) => Ok(vec!["task.claim"]),
            (trigger::MENU, Some("release")) => Ok(vec!["task.release"]),
            (trigger::MENU, Some("note")) => Ok(vec!["task.note", "task.touch"]),
            (trigger::MENU, None) => Err(WriteRefused::new("gesture-ambiguous")),
            (trigger::DROP, None) => Ok(vec!["card.move"]),
            _ => Err(WriteRefused::unbound(trigger)),
        }
    }
}

impl FormWrite for Menus {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        false
    }
    fn send(
        &self,
        _channel: &str,
        _record: Option<&str>,
        _args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        Err(WriteRefused::new("unsupported"))
    }
    fn may_fire(
        &self,
        t: &str,
        entry: Option<&str>,
        _record: Option<&str>,
    ) -> Result<(), WriteRefused> {
        Self::actions(t, entry).map(|_| ())
    }
    fn fire(
        &self,
        t: &str,
        entry: Option<&str>,
        _record: Option<&str>,
        _args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        let all = Self::actions(t, entry)?;
        self.ran
            .lock()
            .unwrap()
            .extend(all.iter().map(|a| (*a).to_owned()));
        Ok(WriteDone::default())
    }
}

#[test]
fn two_entries_at_menu_stay_apart_by_their_name() {
    let w = Menus::default();
    w.fire(trigger::MENU, Some("claim"), Some("c-1"), &[])
        .unwrap();
    w.fire(trigger::MENU, Some("release"), Some("c-1"), &[])
        .unwrap();
    assert_eq!(*w.ran.lock().unwrap(), ["task.claim", "task.release"]);
}

#[test]
fn one_entry_may_carry_several_actions() {
    let w = Menus::default();
    w.fire(trigger::MENU, Some("note"), Some("c-1"), &[])
        .unwrap();
    assert_eq!(*w.ran.lock().unwrap(), ["task.note", "task.touch"]);
}

#[test]
fn without_an_entry_the_menu_is_ambiguous_and_a_gesture_without_entries_needs_none() {
    let w = Menus::default();
    let e = w.fire(trigger::MENU, None, Some("c-1"), &[]).unwrap_err();
    assert_eq!(e.code, "gesture-ambiguous");
    assert!(w.may_fire(trigger::MENU, None, Some("c-1")).is_err());
    assert!(w.fire(trigger::DROP, None, Some("c-1"), &[]).is_ok());
    assert!(w.may_fire(trigger::MENU, Some("nobody"), None).is_err());
}

#[test]
fn a_port_without_entries_refuses_every_entry_as_unbound() {
    let e = NoWrite
        .fire(trigger::MENU, Some("claim"), Some("c-1"), &[])
        .unwrap_err();
    assert_eq!(e, WriteRefused::unbound(trigger::MENU));
    assert_eq!(
        NoWrite.may_fire(trigger::MENU, Some("claim"), None),
        Err(WriteRefused::unbound(trigger::MENU))
    );
}

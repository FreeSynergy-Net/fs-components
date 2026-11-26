//! G1: the gesture side of the write port. A port that only knows channels refuses every gesture as `gesture-unbound`.
//!
//! Pattern: none (tests).
use std::sync::Mutex;

use fs_components_core::form_write::{
    FormWrite, GESTURE_UNBOUND, NoWrite, WriteArg, WriteDone, WriteRefused, trigger,
};

/// An adapter of the old kind: only `can` and `send`.
struct ChannelsOnly;

impl FormWrite for ChannelsOnly {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        true
    }
    fn send(
        &self,
        _channel: &str,
        _record: Option<&str>,
        _args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        Ok(WriteDone::default())
    }
}

/// One call seen: trigger, record, arguments.
type Call = (String, Option<String>, Vec<WriteArg>);

/// An adapter that binds the gesture `drop` only and records what it was given.
#[derive(Default)]
struct DropOnly {
    seen: Mutex<Vec<Call>>,
}

impl FormWrite for DropOnly {
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
        _entry: Option<&str>,
        _record: Option<&str>,
    ) -> Result<(), WriteRefused> {
        if t == trigger::DROP {
            Ok(())
        } else {
            Err(WriteRefused::unbound(t))
        }
    }
    fn fire(
        &self,
        t: &str,
        _entry: Option<&str>,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.may_fire(t, None, record)?;
        self.seen
            .lock()
            .unwrap()
            .push((t.to_owned(), record.map(str::to_owned), args.to_vec()));
        Ok(WriteDone {
            record: record.map(str::to_owned),
            text: None,
        })
    }
}

#[test]
fn the_default_refuses_every_gesture_as_unbound_with_its_trigger() {
    for w in [&NoWrite as &dyn FormWrite, &ChannelsOnly] {
        for t in trigger::ALL {
            let unbound = WriteRefused {
                code: GESTURE_UNBOUND.into(),
                params: vec![("trigger".into(), t.into())],
            };
            assert_eq!(w.may_fire(t, None, Some("c-1")), Err(unbound.clone()));
            assert_eq!(w.fire(t, None, Some("c-1"), &[]), Err(unbound));
        }
    }
}

#[test]
fn an_unknown_trigger_is_refused_the_same_way_not_guessed() {
    let e = ChannelsOnly.fire("spin", None, None, &[]).unwrap_err();
    assert_eq!(e, WriteRefused::unbound("spin"));
    assert_eq!(e.params, vec![("trigger".to_owned(), "spin".to_owned())]);
    // A channel that `send` would do is no gesture: no detour.
    assert!(ChannelsOnly.can("move", None));
    assert!(ChannelsOnly.fire("move", None, None, &[]).is_err());
}

#[test]
fn may_fire_and_fire_agree_and_fire_gets_its_input_unchanged() {
    let w = DropOnly::default();
    let args = [
        WriteArg::text("column", "doing"),
        WriteArg::number("after", 2.0),
    ];
    for t in trigger::ALL {
        let may = w.may_fire(t, None, Some("c-1"));
        let done = w.fire(t, None, Some("c-1"), &args);
        assert_eq!(may.is_ok(), done.is_ok(), "{t}");
        assert_eq!(may.err(), done.err(), "{t}");
    }
    let seen = w.seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0, "drop");
    assert_eq!(seen[0].1.as_deref(), Some("c-1"));
    assert_eq!(seen[0].2, args.to_vec());
}

#[test]
fn the_trigger_list_is_each_word_once_in_kebab_case() {
    assert_eq!(
        trigger::ALL,
        [
            trigger::ACTIVATE,
            trigger::MENU,
            trigger::DROP,
            trigger::QUICK_ADD
        ]
    );
    let mut sorted = trigger::ALL.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), trigger::ALL.len());
    for t in trigger::ALL {
        assert!(
            !t.is_empty() && !t.starts_with('-') && !t.ends_with('-'),
            "{t}"
        );
        assert!(t.chars().all(|c| c.is_ascii_lowercase() || c == '-'), "{t}");
    }
}

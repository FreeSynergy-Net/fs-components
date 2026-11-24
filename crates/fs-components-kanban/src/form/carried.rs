//! The port for the channels the board carries in its gestures: the host asks the element, it does not rebuild the element.
//!
//! Pattern: Ports and Adapters (a published port of the adapter; the answer comes from the real [`FormEvents`] run against a
//! recording [`FormWrite`], so it cannot drift from what the gestures send). Deviation: no trait, a function, as the answer is a fixed list.

use std::sync::{Arc, Mutex, PoisonError};

use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused};
use fs_components_fields::KeysAsText;

use super::events::FormEvents;
use crate::{BoardEvents, Card, Column, MemoryBoard, MoveRequest, NewCard};

/// A write port that allows everything and remembers the names of the channels and arguments it saw.
struct Probe(Mutex<Vec<String>>);

impl FormWrite for Probe {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        true
    }
    fn send(
        &self,
        channel: &str,
        _record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        let mut names = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        names.push(channel.to_owned());
        names.extend(args.iter().map(|a| a.name.clone()));
        Ok(WriteDone::default())
    }
}

/// The names (sorted, without doubles) of the channels and arguments the gestures of the board carry: a move with a place between
/// two cards (`column`, `after`, `before`) and a new card (`title`, `column`, `lane`), with the gesture channels `move` and `create`.
pub fn carried_channels() -> Vec<String> {
    let probe = Arc::new(Probe(Mutex::default()));
    // The lane is only sent when the column has one; the first column has none, the second has.
    let mut plain = Column::new("a", "");
    plain.cards = vec![Card::new("x", ""), Card::new("y", "")];
    let mut laned = Column::new("b", "");
    laned.lane = Some("l".to_owned());
    let events = FormEvents::new(
        probe.clone(),
        Arc::new(MemoryBoard::new(vec![plain, laned])),
        Arc::new(KeysAsText),
    );
    // after card `x`: both neighbours (`after` = x, `before` = y)
    let _ = events.moved(&MoveRequest::new("", "", "a", 1, Some("x".to_owned())));
    let _ = events.new_card(&NewCard::new("b", "", false));
    let mut names = probe
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    names.sort();
    names.dedup();
    names
}

//! The frame at the side of a card from the rows of the form `cards-in-columns`: the path is the name of the column
//! (`column-label`), the title the `title`, the properties the channels the card has: `facts` (one line per field of the module),
//! `priority`, `state`, `assignee`, `description`. A channel that no module field fills and the row does not carry takes no place;
//! a channel the module fills but the row leaves empty is a line with an empty value (the frame draws it as the grey "set").
//! The names are the host's sentences for `channel.<name>` (a field of `facts`: `fact.<name>`, as on the card), else the bare name.
//!
//! Pattern: Strategy (`DetailContent`, the program's content of the frame) as an Object Adapter: it holds the source of the rows
//! and translates one row into the frame of the core. The core (`DetailFrame`) knows nothing of channels. The main button is none:
//! the moves are the buttons the frame brings itself.

use std::sync::Arc;

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::said_or;
use fs_components_fields::Say;

use super::read::fact_label;
use super::source::FormSource;
use crate::detail::{DetailContent, DetailFrame};

/// The channels shown as plain properties, in order (after `facts`).
const OWN: [&str; 4] = ["priority", "state", "assignee", "description"];

/// The content of the frame of a card, read from the rows of the form.
pub struct FormDetail {
    source: Arc<FormSource>,
    say: Arc<dyn Say>,
}

impl FormDetail {
    /// The content over `source` (the rows as they are now); `say` words the names.
    pub fn new(source: Arc<FormSource>, say: Arc<dyn Say>) -> Self {
        Self { source, say }
    }
}

impl DetailContent for FormDetail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        self.source
            .read(|input| frame_of(input, self.say.as_ref(), card))
    }
}

/// The sentence of `key`, or `name` when the host has none (`say` answers the key itself then).
fn named(say: &dyn Say, key: &str, name: &str) -> String {
    said_or(say.say(key), key, name)
}

/// The values of a channel of a row as one text (the channel may repeat): non-empty values joined by a comma.
fn joined(row: &FormRow, channel: &str) -> String {
    row.all(channel)
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

/// The lines of the channel `facts`: for each field the values that carry ITS path (one line per value under the same name; none = one
/// line with an empty value), never by position. A field the module maps also to `assignee` is the assignee line.
fn fact_lines(input: &FormInput, row: &FormRow, say: &dyn Say) -> Vec<(String, String)> {
    let fields = input.channel("facts").map_or(&[][..], |c| &c.fields[..]);
    let in_assignee: Vec<&str> = input.channel("assignee").map_or_else(Vec::new, |c| {
        c.fields.iter().map(|f| f.name.as_str()).collect()
    });
    fields
        .iter()
        .filter(|f| !in_assignee.contains(&f.name.as_str()))
        .flat_map(|f| {
            let name = named(say, &fact_label(f), &f.name);
            let values = row.field("facts", &f.name);
            let lines: Vec<String> = if values.is_empty() {
                vec![String::new()]
            } else {
                values.into_iter().map(|v| v.trim().to_owned()).collect()
            };
            lines.into_iter().map(move |v| (name.clone(), v))
        })
        .collect()
}

/// The frame of the card `card`, or `None` when no row has this key.
fn frame_of(input: &FormInput, say: &dyn Say, card: &str) -> Option<DetailFrame> {
    let row = input.rows.iter().find(|r| r.key == card)?;
    let column = row.get("column").unwrap_or_default();
    let mut frame = DetailFrame::new(row.get("title").unwrap_or(card))
        .path(row.get("column-label").unwrap_or(column));
    let lines = fact_lines(input, row, say);
    let mapped = |name: &str| {
        input
            .channel("facts")
            .is_some_and(|c| c.fields.iter().any(|f| f.name == name))
    };
    for (name, value) in lines {
        frame = frame.property(name, value);
    }
    for channel in OWN {
        let value = joined(row, channel);
        // a field of `facts` with the same name wins (no double); no module field and no value: no line
        if mapped(channel) || (value.is_empty() && input.channel(channel).is_none()) {
            continue;
        }
        let name = named(say, &format!("channel.{channel}"), channel);
        frame = frame.property(name, value);
    }
    Some(frame)
}

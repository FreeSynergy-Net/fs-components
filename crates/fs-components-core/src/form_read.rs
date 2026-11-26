//! The read side of a form (the counterpart of [`crate::form_write`]): what a component is given to draw a form. The input is the
//! CHANNELS of the form (`label`, `parent`, `column` ...), never the fields of a module; the host fills it once from what a module
//! offers, so no component parses a module's data itself and none knows a field name.
//!
//! Every value keeps the path of its field (the mirror of `Value { path, text }` on the wire, `formsource.capnp`), so a reader asks
//! "the values of field F in channel K", never "value number i": a field with two values or none cannot shift the others.
//!
//! Pattern: Value objects (also the lane heads: `Heads`, `HeadRow`, and `FieldValue`) (rows by channel, the channels' own information,
//! the state of the load); `FormRow::from_values` is the one Adapter from the wire values to a row.

use crate::states::Load;

/// One value of a record with the path of the field it belongs to (RFC 9553 / RFC 6350: a field may repeat, `emails` has one entry
/// per address).
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldValue {
    /// The path of the field of the module (`emails`).
    pub path: String,
    /// The text of the value (may be empty).
    pub text: String,
}

impl FieldValue {
    /// A value of the field `path`.
    #[must_use]
    pub fn new(path: &str, text: &str) -> Self {
        Self {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }
}

/// One record: a key and its values by channel (a channel may have several values; a record may leave a channel out).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormRow {
    /// The key of the record.
    pub key: String,
    /// The values by channel name, each with the path of its field.
    pub values: Vec<(String, Vec<FieldValue>)>,
    /// What the module lets the asking person do with this record: the keys of its commands (`card.move`). No statement = nothing.
    pub may: Vec<String>,
}

impl FormRow {
    /// A row with these `(channel, value)` pairs (one value each; the path of the value is the channel name).
    #[must_use]
    pub fn of(key: &str, pairs: &[(&str, &str)]) -> Self {
        Self {
            key: key.to_owned(),
            values: pairs
                .iter()
                .map(|(c, v)| ((*c).to_owned(), vec![FieldValue::new(c, v)]))
                .collect(),
            may: Vec::new(),
        }
    }

    /// The same row with the values of `channel` as `(path, text)` pairs, in order (an earlier entry of the channel is replaced).
    #[must_use]
    pub fn with_values(mut self, channel: &str, values: &[(&str, &str)]) -> Self {
        self.values.retain(|(c, _)| c != channel);
        self.values.push((
            channel.to_owned(),
            values.iter().map(|(p, t)| FieldValue::new(p, t)).collect(),
        ));
        self
    }

    /// The row of a record from the values the host got on the wire: `map` is the map of the form (channel, the fields that fill
    /// it), `values` the `(path, text)` pairs of the record in the order of the module. Each value goes to the channel(s) whose
    /// field it belongs to; a value of a field the map does not name is never seen; a field without a value leaves nothing behind.
    #[must_use]
    pub fn from_values<P, T>(
        key: &str,
        may: Vec<String>,
        map: &[(String, Vec<String>)],
        values: impl IntoIterator<Item = (P, T)>,
    ) -> Self
    where
        P: AsRef<str>,
        T: AsRef<str>,
    {
        let values: Vec<(P, T)> = values.into_iter().collect();
        let by_channel = map
            .iter()
            .map(|(channel, fields)| {
                let taken = fields
                    .iter()
                    .flat_map(|f| values.iter().filter(move |(p, _)| p.as_ref() == f))
                    .map(|(p, t)| FieldValue::new(p.as_ref(), t.as_ref()))
                    .collect::<Vec<_>>();
                (channel.clone(), taken)
            })
            .filter(|(_, v)| !v.is_empty())
            .collect();
        Self {
            key: key.to_owned(),
            values: by_channel,
            may,
        }
    }

    /// The same row with the commands the person may use on it.
    #[must_use]
    pub fn allowing(mut self, keys: &[&str]) -> Self {
        self.may = keys.iter().map(|k| (*k).to_owned()).collect();
        self
    }

    /// The texts of the values of `channel`, in order (none when the record leaves it out).
    #[must_use]
    pub fn all(&self, channel: &str) -> Vec<&str> {
        self.values
            .iter()
            .find(|(c, _)| c == channel)
            .map_or_else(Vec::new, |(_, v)| {
                v.iter().map(|x| x.text.as_str()).collect()
            })
    }

    /// The texts of the values of the field `path` in `channel`, in the order of the module (none when the record has none: the
    /// cell of the field is then empty; never the value of another field).
    #[must_use]
    pub fn field(&self, channel: &str, path: &str) -> Vec<&str> {
        self.values
            .iter()
            .filter(|(c, _)| c == channel)
            .flat_map(|(_, v)| v.iter())
            .filter(|x| x.path == path)
            .map(|x| x.text.as_str())
            .collect()
    }

    /// The first value of `channel`; empty values count as none.
    #[must_use]
    pub fn get(&self, channel: &str) -> Option<&str> {
        self.all(channel).first().copied().filter(|v| !v.is_empty())
    }
}

/// A percentage from the text of a channel (`progress`): a number, cut to 0 to 100 and rounded; a text that is no number: none.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is cut to 0..=100 first"
)]
pub fn percent(text: &str) -> Option<u8> {
    let p: f64 = text.trim().parse().ok()?;
    p.is_finite().then(|| p.clamp(0.0, 100.0).round() as u8)
}

/// One field of a module that fills a channel, as far as a component needs to know it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInfo {
    /// The name of the field (the host may show it as a label).
    pub name: String,
    /// The building block of the contract that draws a value of this field's type (`field-email`, `field-text` ...): the host chose it
    /// from the type; the component only draws what it is told.
    pub block: String,
    /// The word id of the field from the type catalog of the module (`person.emails`, ADR 0066); empty = the module names none, the
    /// reader then shows `name`. The head of a column or the label of a fact is `say(label)`, never a guess from the field name.
    pub label: String,
}

impl FieldInfo {
    /// The words that name the field for a reader: the sentence of its word id (`label`), else the name of the field. The ONE place
    /// that rule lives (heads of columns, labels of a sheet), so no reader guesses a sentence key from the field name.
    #[must_use]
    pub fn shown(&self, say: &dyn Fn(&str) -> String) -> String {
        if self.label.is_empty() {
            self.name.clone()
        } else {
            crate::form_view::said_or(say(&self.label), &self.label, &self.name)
        }
    }
}

/// What the module says about one channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelInfo {
    /// The name of the channel.
    pub name: String,
    /// The fields of the module that fill it, in the order of the module's map (the values of the channel carry the path of their field).
    pub fields: Vec<FieldInfo>,
    /// The values the channel may take, in order, as `(key, label)`, also those no record uses yet (the named columns of a board).
    pub choices: Vec<(String, String)>,
}

/// The heads of the lanes of a board, as the host sends them. The grouping is always the channel `assignee` (the only grouping the
/// board knows today), so it is not part of the input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heads {
    /// Lanes were asked for, but the list did not arrive.
    Missing,
    /// The heads in their order.
    Rows(Vec<HeadRow>),
}

/// One head: `key` is the value cards carry in the grouping channel (already matched by the host), `label` what the reader sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadRow {
    /// The value cards carry.
    pub key: String,
    /// The name shown (empty = show the key).
    pub label: String,
}

/// What a component is given to draw one form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormInput {
    /// The form and its version that the module keeps (`cards-in-columns@1.1.0`).
    pub form: String,
    /// The records.
    pub rows: Vec<FormRow>,
    /// What the module says about each channel it fills.
    pub channels: Vec<ChannelInfo>,
    /// What the person may do without a record (`create`): the keys of the commands.
    pub may: Vec<String>,
    /// How far the reading is.
    pub load: Load,
    /// The lanes (swimlanes) the host asks for: `None` = no lanes asked; see [`Heads`].
    pub heads: Option<Heads>,
}

impl FormInput {
    /// The information about `channel`, if the module fills it.
    #[must_use]
    pub fn channel(&self, channel: &str) -> Option<&ChannelInfo> {
        self.channels.iter().find(|c| c.name == channel)
    }
}

#[cfg(test)]
mod tests {
    use super::FormRow;

    #[test]
    fn an_empty_value_counts_as_none_and_a_missing_channel_has_no_values() {
        let row = FormRow::of("a", &[("title", "T"), ("note", "")]);
        assert_eq!(row.get("title"), Some("T"));
        assert_eq!(row.get("note"), None);
        assert_eq!(row.all("nope"), Vec::<&str>::new());
    }
}

//! The read side of a form (the counterpart of [`crate::form_write`]): what a component is given to draw a form. The input is the
//! CHANNELS of the form (`label`, `parent`, `column` ...), never the fields of a module; the host fills it once from what a module
//! offers, so no component parses a module's data itself and none knows a field name.
//!
//! Pattern: Value objects (rows by channel, the channels' own information, the state of the load).

use crate::states::Load;

/// One record: a key and its values by channel (a channel may have several values; a record may leave a channel out).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormRow {
    /// The key of the record.
    pub key: String,
    /// The values by channel name.
    pub values: Vec<(String, Vec<String>)>,
    /// What the module lets the asking person do with this record: the keys of its commands (`card.move`). No statement = nothing.
    pub may: Vec<String>,
}

impl FormRow {
    /// A row with these `(channel, value)` pairs (one value each).
    #[must_use]
    pub fn of(key: &str, pairs: &[(&str, &str)]) -> Self {
        Self {
            key: key.to_owned(),
            values: pairs
                .iter()
                .map(|(c, v)| ((*c).to_owned(), vec![(*v).to_owned()]))
                .collect(),
            may: Vec::new(),
        }
    }

    /// The same row with the commands the person may use on it.
    #[must_use]
    pub fn allowing(mut self, keys: &[&str]) -> Self {
        self.may = keys.iter().map(|k| (*k).to_owned()).collect();
        self
    }

    /// The values of `channel` (none when the record leaves it out).
    #[must_use]
    pub fn all(&self, channel: &str) -> &[String] {
        self.values
            .iter()
            .find(|(c, _)| c == channel)
            .map_or(&[], |(_, v)| v.as_slice())
    }

    /// The first value of `channel`; empty values count as none.
    #[must_use]
    pub fn get(&self, channel: &str) -> Option<&str> {
        self.all(channel)
            .first()
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }
}

/// One field of a module that fills a channel, as far as a component needs to know it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInfo {
    /// The name of the field (the host may show it as a label).
    pub name: String,
    /// The building block of the contract that draws a value of this field's type (`field-email`, `field-text` ...): the host chose it
    /// from the type; the component only draws what it is told.
    pub block: String,
}

/// What the module says about one channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelInfo {
    /// The name of the channel.
    pub name: String,
    /// The fields of the module that fill it, in the order of the module's map (one value of the channel per field).
    pub fields: Vec<FieldInfo>,
    /// The values the channel may take, in order, as `(key, label)`, also those no record uses yet (the named columns of a board).
    pub choices: Vec<(String, String)>,
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
        assert_eq!(row.all("nope"), [] as [String; 0]);
    }
}

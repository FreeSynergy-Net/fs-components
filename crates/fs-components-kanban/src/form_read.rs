//! The board read from the channels of the form `cards-in-columns`: `column` is the KEY of a column (the value as the module gives
//! it, never changed: a move names it), `column-label` its name, `order` the place (the cards of a column stand in the sequence of
//! `order`, cards without a number at the end), `progress` the bar, `facts` the facts of the card (one value per field the module maps
//! to the channel, in the order of the map; no value = the slot is kept, not drawn). A column the module names but that holds no card
//! (`choices` of the channel `column`) is drawn too.
//!
//! Pattern: Adapter (rows by channel in, the columns of the board out); pure functions.

use std::cmp::Ordering;
use std::collections::HashMap;

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_fields::FieldSlot;

use crate::model::{Card, Column};

/// The bar of a card from the text of the channel `progress`: a number, cut to 0 to 100 (a text that is no number: no bar).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is cut to 0..=100 first"
)]
fn percent(text: &str) -> Option<u8> {
    let p: f64 = text.trim().parse().ok()?;
    p.is_finite().then(|| p.clamp(0.0, 100.0).round() as u8)
}

/// The card of a row: title, bar and the facts in the order of the fields of the channel `facts`.
fn card_of(input: &FormInput, row: &FormRow, title: &str) -> Card {
    let mut card = Card::new(&row.key, title);
    card.progress = row.get("progress").and_then(percent);
    let fields = input.channel("facts").map_or(&[][..], |c| &c.fields[..]);
    for (i, field) in fields.iter().enumerate() {
        let label = format!("fact.{}", field.name);
        let value = row.all("facts").get(i).map_or("", String::as_str);
        card = card.fact_or_more(if value.is_empty() {
            FieldSlot::unset(&field.block, &label)
        } else {
            FieldSlot::new(&field.block, &label, value)
        });
    }
    card
}

/// The number in the channel `order` of a row (none: the channel is missing, empty or no finite number).
fn order_of(row: &FormRow) -> Option<f64> {
    row.get("order")
        .and_then(|t| t.trim().parse::<f64>().ok())
        .filter(|n| n.is_finite())
}

/// Sorts the cards of every column STABLY by the `order` of their row; cards without a number go to the end in the order they came.
fn sort_by_order(columns: &mut [Column], rows: &[FormRow]) {
    let orders: HashMap<&str, Option<f64>> =
        rows.iter().map(|r| (r.key.as_str(), order_of(r))).collect();
    let place = |key: &str| orders.get(key).copied().flatten();
    for column in columns {
        column
            .cards
            .sort_by(|a, b| match (place(&a.key), place(&b.key)) {
                (Some(x), Some(y)) => x.total_cmp(&y),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            });
    }
}

/// The columns of the board with their cards: the columns the module names (also empty ones), then the others in order of
/// appearance. The key of a column is the value of the channel `column` unchanged; its name is `column-label` or the key.
#[must_use]
pub fn columns_of(input: &FormInput) -> Vec<Column> {
    let named = input.channel("column").map_or(&[][..], |c| &c.choices[..]);
    let mut out: Vec<Column> = named
        .iter()
        .map(|(key, label)| Column::new(key, label))
        .collect();
    for row in &input.rows {
        let (Some(title), Some(column)) = (row.get("title"), row.get("column")) else {
            continue; // required channels: a row without them cannot be placed
        };
        let at = out.iter().position(|c| c.key == column).unwrap_or_else(|| {
            out.push(Column::new(
                column,
                row.get("column-label").unwrap_or(column),
            ));
            out.len() - 1
        });
        if out[at].lane.is_none() {
            out[at].lane = row.get("lane").filter(|l| !l.is_empty()).map(str::to_owned);
        }
        out[at].cards.push(card_of(input, row, title));
    }
    sort_by_order(&mut out, &input.rows);
    out
}

#[cfg(test)]
mod tests {
    use fs_components_core::form_read::{ChannelInfo, FieldInfo};
    use fs_components_core::states::Load;

    use super::*;

    fn input(rows: Vec<FormRow>) -> FormInput {
        FormInput {
            form: "cards-in-columns@1.0.0".into(),
            rows,
            channels: Vec::new(),
            may: Vec::new(),
            load: Load::Ready,
        }
    }

    fn card(key: &str, column: &str, order: Option<&str>) -> FormRow {
        let mut pairs = vec![("title", key), ("column", column)];
        if let Some(o) = order {
            pairs.push(("order", o));
        }
        FormRow::of(key, &pairs)
    }

    fn keys(columns: &[Column], at: usize) -> Vec<String> {
        columns[at].cards.iter().map(|c| c.key.clone()).collect()
    }

    #[test]
    fn the_board_is_made_from_the_channels_column_and_title_in_order_of_appearance() {
        let cols = columns_of(&input(vec![
            FormRow::of("a", &[("title", "One"), ("column", "To do")]),
            FormRow::of("b", &[("title", "Two"), ("column", "Doing")]),
            FormRow::of("c", &[("title", "Three"), ("column", "To do")]),
            FormRow::of("d", &[("title", "No column")]),
        ]));
        let names: Vec<_> = cols.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(names, ["To do", "Doing"]);
        assert_eq!(keys(&cols, 0), ["a", "c"]);
        assert_eq!(keys(&cols, 1), ["b"]);
        let none = columns_of(&input(Vec::new()));
        assert_eq!(none.len(), usize::from(false));
    }

    #[test]
    fn the_key_of_a_column_is_its_value_and_its_name_is_the_label() {
        let cols = columns_of(&input(vec![FormRow::of(
            "a",
            &[
                ("title", "A"),
                ("column", "To-Do_1"),
                ("column-label", "To do"),
            ],
        )]));
        assert_eq!(cols[0].key, "To-Do_1", "no slug: a move names this key");
        assert_eq!(cols[0].title, "To do");
    }

    #[test]
    fn a_column_the_module_names_is_drawn_without_a_card_and_in_its_place() {
        let mut i = input(vec![card("a", "doing", None)]);
        i.channels.push(ChannelInfo {
            name: "column".into(),
            fields: Vec::new(),
            choices: vec![
                ("todo".into(), "To do".into()),
                ("doing".into(), "In progress".into()),
                ("closed".into(), "Closed".into()),
            ],
        });
        let cols = columns_of(&i);
        let ks: Vec<_> = cols.iter().map(|c| c.key.as_str()).collect();
        assert_eq!(ks, ["todo", "doing", "closed"]);
        assert!(cols[2].cards.is_empty() && cols[2].title == "Closed");
        assert_eq!(cols[1].cards.len(), 1);
    }

    #[test]
    fn a_card_has_its_bar_and_its_facts_in_the_order_of_the_map() {
        let mut a = card("a", "todo", None);
        a.values.push(("progress".into(), vec!["40".into()]));
        a.values.push((
            "facts".into(),
            vec!["2026-11-26T12:00:00".into(), "high".into()],
        ));
        let mut b = card("b", "doing", None);
        b.values.push(("progress".into(), vec!["250".into()]));
        b.values
            .push(("facts".into(), vec![String::new(), "low".into()]));
        let mut i = input(vec![a, b]);
        let field = |n: &str, b: &str| FieldInfo {
            name: n.into(),
            block: b.into(),
        };
        i.channels.push(ChannelInfo {
            name: "facts".into(),
            fields: vec![field("due", "field-date-time"), field("prio", "field-text")],
            choices: Vec::new(),
        });
        let cols = columns_of(&i);
        let a = &cols[0].cards[0];
        assert_eq!(a.progress, Some(40));
        let facts: Vec<_> = a
            .facts()
            .iter()
            .map(|f| (f.block.as_str(), f.value.as_str()))
            .collect();
        assert_eq!(
            facts,
            [
                ("field-date-time", "2026-11-26T12:00:00"),
                ("field-text", "high")
            ]
        );
        let b = &cols[1].cards[0];
        assert_eq!(b.progress, Some(100), "more than 100 is cut to 100");
        assert_eq!(b.facts().len(), 2, "a fact without a value keeps its place");
        assert_eq!(b.shown_facts().count(), 1, "but it is not drawn");
    }

    #[test]
    fn a_module_that_delivers_unsorted_is_drawn_in_the_sequence_of_order() {
        let cols = columns_of(&input(vec![
            card("c", "todo", Some("2")),
            card("a", "todo", Some("0")),
            card("b", "todo", Some("1")),
            card("x", "doing", Some("10")),
            card("y", "doing", Some("9")),
        ]));
        assert_eq!(keys(&cols, 0), ["a", "b", "c"]);
        assert_eq!(keys(&cols, 1), ["y", "x"]);
    }

    #[test]
    fn cards_without_a_number_stay_in_the_order_they_came_and_stand_behind_the_numbered_ones() {
        let cols = columns_of(&input(vec![
            card("n1", "todo", None),
            card("b", "todo", Some("5")),
            card("n2", "todo", Some("soon")),
            card("a", "todo", Some("5")),
            card("n3", "todo", Some("")),
        ]));
        // equal numbers keep the delivery (stable): b before a
        assert_eq!(keys(&cols, 0), ["b", "a", "n1", "n2", "n3"]);
    }

    #[test]
    fn many_cards_with_the_same_number_keep_the_delivery_the_sort_is_stable() {
        let num = |i: usize| (i * 7 + i / 3) % 5;
        let rows: Vec<_> = (0..200)
            .map(|i| card(&format!("k{i:03}"), "todo", Some(&num(i).to_string())))
            .collect();
        let mut want: Vec<usize> = (0..200).collect();
        want.sort_by_key(|i| (num(*i), *i));
        let want: Vec<String> = want.iter().map(|i| format!("k{i:03}")).collect();
        assert_eq!(keys(&columns_of(&input(rows)), 0), want);
    }
}

#[cfg(test)]
mod lane_tests {
    use fs_components_core::form_read::{FormInput, FormRow};
    use fs_components_core::states::Load;

    use super::columns_of;

    #[test]
    fn a_column_takes_the_lane_of_its_cards_and_none_when_they_name_none() {
        let input = FormInput {
            form: "cards-in-columns@1.2.0".into(),
            rows: vec![
                FormRow::of("a", &[("title", "A"), ("column", "x"), ("lane", "b1")]),
                FormRow::of("b", &[("title", "B"), ("column", "y"), ("lane", "")]),
            ],
            channels: Vec::new(),
            may: Vec::new(),
            load: Load::Ready,
        };
        let cols = columns_of(&input);
        assert_eq!(cols[0].lane.as_deref(), Some("b1"));
        assert_eq!(cols[1].lane, None);
    }
}

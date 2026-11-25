//! The board read from the channels of the form `cards-in-columns`: `column` is the KEY of a column (the value as the module gives
//! it, never changed: a move names it), `column-label` its name, `order` the place (the cards of a column stand in the sequence of
//! `order`, cards without a number at the end), `progress` the bar, `priority` and `estimated` (channels of their own, one fact each, labelled `fact.priority` / `fact.estimated`: a whole number and an ISO duration; drawn BEFORE the fields of `facts` so a full `facts` channel cannot push them into "+ N more"; a field of `facts` with the same name wins, no double), `facts` the facts of the card (one value per field the module maps
//! to the channel, in the order of the map; no value = the slot is kept, not drawn), `assignee` the assignees of the card (a repeatable channel, one name per value, drawn as initials, more than `MAX_ASSIGNEES` as "+N"; a field of `facts` the module also maps to `assignee` is not drawn twice). `column-limit` is the work-in-progress limit of the column: the first row of the column that carries a whole number sets it (no number, none: no limit). A column the module names but that holds no card
//! (`choices` of the channel `column`) is drawn too.
//!
//! Pattern: Adapter (rows by channel in, the columns of the board out); pure functions.

use std::cmp::Ordering;
use std::collections::HashMap;

use fs_components_core::form_read::{FieldInfo, FormInput, FormRow, percent};
use fs_components_fields::{FieldSlot, Kind};

use crate::model::{Card, Column};

/// A channel of its own (`priority`, `estimated`) as one more fact of the card, labelled `fact.<channel>` and drawn by the block of `kind`;
/// no value in the row = no fact (nothing empty is added). A field of the channel `facts` with the same name wins: the module already
/// maps it there, so the fact is not drawn twice under the same label.
fn own_fact(card: Card, row: &FormRow, mapped: &[FieldInfo], channel: &str, kind: Kind) -> Card {
    if mapped.iter().any(|f| f.name == channel) {
        return card;
    }
    match row.get(channel).map(str::trim).filter(|v| !v.is_empty()) {
        Some(value) => card.fact_or_more(FieldSlot::new(
            kind.block(),
            &format!("fact.{channel}"),
            value,
        )),
        None => card,
    }
}

/// The facts of the channel `facts` as slots. The channel carries ONE value per field the module maps to it, in the order of the map
/// and without a field name (`FormRow` keeps values by channel only), so a value belongs to a field only while the counts agree.
/// A field the module maps ALSO to the channel `assignee` is not drawn here (it shows as the initials of the card, once); a field only in
/// `facts` stays a fact, whatever its name.
/// When a field gave several values (counts differ) the position says nothing about the field any more: no value is laid on a
/// wrong label; every value is drawn under the label `fact.facts` (nothing is lost, nothing is mislabelled).
fn facts_of(card: Card, row: &FormRow, fields: &[FieldInfo], in_assignee: &[&str]) -> Card {
    let values = row.all("facts");
    if values.len() != fields.len() && !values.is_empty() {
        return values.iter().filter(|v| !v.is_empty()).fold(card, |c, v| {
            c.fact_or_more(FieldSlot::new("field-text", "fact.facts", v))
        });
    }
    fields
        .iter()
        .enumerate()
        .filter(|(_, f)| !in_assignee.contains(&f.name.as_str()))
        .fold(card, |c, (i, field)| {
            let label = format!("fact.{}", field.name);
            let value = values.get(i).map_or("", String::as_str);
            c.fact_or_more(if value.is_empty() {
                FieldSlot::unset(&field.block, &label)
            } else {
                FieldSlot::new(&field.block, &label, value)
            })
        })
}

/// The assignees of a row: every non-empty value of the channel `assignee` (the channel repeats, one value per person), in order.
fn assignees_of(row: &FormRow) -> Vec<String> {
    row.all("assignee")
        .iter()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .collect()
}

/// The card of a row: title, bar, the assignees and the facts in the order of the fields of the channel `facts`.
fn card_of(input: &FormInput, row: &FormRow, title: &str) -> Card {
    let mut card = Card::new(&row.key, title);
    card.progress = row.get("progress").and_then(percent);
    card.assignees = assignees_of(row);
    let fields = input.channel("facts").map_or(&[][..], |c| &c.fields[..]);
    // The channels of their own come FIRST: behind a full `facts` channel (limit `DEFAULT_FACTS`) they would land only in "+ N more".
    card = own_fact(card, row, fields, "priority", Kind::Integer);
    card = own_fact(card, row, fields, "estimated", Kind::Duration);
    let in_assignee: Vec<&str> = input.channel("assignee").map_or_else(Vec::new, |c| {
        c.fields.iter().map(|f| f.name.as_str()).collect()
    });
    facts_of(card, row, fields, &in_assignee)
}

/// The WIP limit in the channel `column-limit` of a row: a whole number (anything else: none).
fn limit_of(row: &FormRow) -> Option<usize> {
    row.get("column-limit")
        .and_then(|t| t.trim().parse::<usize>().ok())
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
    let mut mixed: Vec<usize> = Vec::new(); // columns whose cards name more than one lane
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
        if let Some(lane) = row.get("lane").filter(|l| !l.is_empty()) {
            if mixed.contains(&at) {
                // already mixed: stays without a lane
            } else if out[at].lane.as_deref().is_none_or(|l| l == lane) {
                out[at].lane = Some(lane.to_owned());
            } else {
                // cards of two lanes in one column: no lane is guessed
                out[at].lane = None;
                mixed.push(at);
            }
        }
        if out[at].limit.is_none() {
            out[at].limit = limit_of(row);
        }
        out[at].cards.push(card_of(input, row, title));
    }
    sort_by_order(&mut out, &input.rows);
    out
}

#[cfg(test)]
mod tests;

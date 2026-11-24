//! The gestures of the board in the form view: the cards through [`FormEvents`], the columns through the dialogs of `ask.rs`.
//! The "..." button of a column (the host has no menu) opens a choice: rename, limit, a new column.
//!
//! Pattern: Adapter (`Events` is the `BoardEvents` of the element over the write port) + Strategy (one dialog per gesture).

use std::sync::Arc;

use fs_components_core::form_write::WriteArg;
use fs_components_widgets::dialogs::{ChoiceDialog, ChoiceOption};

use super::ask::{Text, Words};
use super::events::{COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, LIMIT, NAME};
use super::view::Wiring;
use crate::{Answer, BoardEvents, ColumnAction, MoveRequest, NewCard};

/// The gestures of the board: the cards through [`FormEvents`], the columns through dialogs.
pub(crate) struct Events(pub(crate) Arc<Wiring>);

impl Events {
    /// Ask the reader one line (keys: dialog id, title, placeholder, hint) and give it to `apply`.
    fn ask_text(
        &self,
        (id, title, placeholder, hint): (&'static str, &str, &str, Option<&str>),
        initial: &str,
        apply: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    ) {
        let w = &self.0;
        let words = Words {
            id,
            title: w.say.say(title),
            placeholder: w.say.say(placeholder),
            hint: hint.map(|k| w.say.say(k)),
        };
        Text::new(&w.ask, &w.labels, words, apply).open(initial, None);
    }

    /// The "..." button of `column`: one thing out of rename, limit, new column; what the host does not allow is greyed out.
    fn choose(&self, column: &str) {
        let w = self.0.clone();
        let (rename, limit, add) = ("rename", "limit", "add");
        let option = |key: &str, word: &str, channel: &str| {
            let option = ChoiceOption::new(key, &w.say.say(word));
            if w.can(channel) {
                option
            } else {
                option.disabled()
            }
        };
        let (done, left) = (w.clone(), w.clone());
        let column = column.to_owned();
        let dialog = ChoiceDialog::new(&w.labels, &w.column_menu)
            .id("column-menu")
            .option(option(rename, "kanban.rename", COLUMN_RENAME))
            .option(option(limit, "kanban.limit", COLUMN_LIMIT))
            .option(option(add, "kanban.new_column", COLUMN_CREATE))
            .on_cancel(move || left.ask.set_choice(None))
            .on_choose(move |key| {
                done.ask.set_choice(None);
                let events = Self(done.clone());
                match key {
                    "rename" => events.rename_column(&column),
                    "limit" => events.set_limit(&column),
                    _ => events.add_column(),
                }
            });
        w.ask.set_choice(Some(dialog));
    }

    fn add_column(&self) {
        if !self.0.can(COLUMN_CREATE) {
            return;
        }
        let w = self.0.clone();
        let keys = (
            "new-column",
            "kanban.new_column",
            "kanban.column_name",
            None,
        );
        self.ask_text(keys, "", move |name| {
            w.send(COLUMN_CREATE, None, &[WriteArg::text(NAME, name)])
        });
    }

    fn rename_column(&self, column: &str) {
        if !self.0.can(COLUMN_RENAME) {
            return;
        }
        let (w, column) = (self.0.clone(), column.to_owned());
        let initial = self.0.source.column(&column).map(|c| c.title);
        let keys = ("rename-column", "kanban.rename", "kanban.column_name", None);
        self.ask_text(keys, &initial.unwrap_or_default(), move |name| {
            w.send(COLUMN_RENAME, Some(&column), &[WriteArg::text(NAME, name)])
        });
    }

    fn set_limit(&self, column: &str) {
        if !self.0.can(COLUMN_LIMIT) {
            return;
        }
        let (w, column) = (self.0.clone(), column.to_owned());
        let initial = self
            .0
            .source
            .column(&column)
            .and_then(|c| c.limit)
            .map_or_else(String::new, |n| n.to_string());
        let hint = Some("kanban.limit_hint");
        let keys = (
            "column-limit",
            "kanban.limit",
            "kanban.limit_placeholder",
            hint,
        );
        self.ask_text(keys, &initial, move |text| match parse_limit(text) {
            Ok(limit) => {
                let n = f64::from(limit.unwrap_or(0));
                w.send(COLUMN_LIMIT, Some(&column), &[WriteArg::number(LIMIT, n)])
            }
            // not a number: the dialog stays open and says so
            Err(_) => Err(w.say.say("kanban.limit_invalid")),
        });
    }
}

impl BoardEvents for Events {
    fn moved(&self, request: &MoveRequest) -> Answer {
        self.0.done(self.0.cards.moved(request))
    }

    fn new_card(&self, card: &NewCard) -> Answer {
        self.0.done(self.0.cards.new_card(card))
    }

    fn column_action(&self, column: &str, action: ColumnAction) {
        match action {
            ColumnAction::Rename => self.rename_column(column),
            ColumnAction::SetLimit => self.set_limit(column),
            ColumnAction::EditColumns => self.choose(column),
            _ => {}
        }
    }
}

/// The limit a line says: a whole number of cards, `0` = no limit; `Err` when the line is not a number.
fn parse_limit(text: &str) -> Result<Option<u32>, std::num::ParseIntError> {
    let n: u32 = text.trim().parse()?;
    Ok((n > 0).then_some(n))
}

#[cfg(test)]
mod tests {
    use super::parse_limit;

    #[test]
    fn a_limit_is_a_whole_number_and_zero_is_none() {
        assert_eq!(parse_limit(" 3 ").ok(), Some(Some(3)));
        assert_eq!(parse_limit("0").ok(), Some(None));
        assert!(parse_limit("many").is_err());
    }
}

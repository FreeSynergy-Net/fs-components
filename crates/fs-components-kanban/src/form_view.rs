//! The board as a [`FormView`]: it reads the form `cards-in-columns` by channel ([`columns_of`]) and writes only through the write
//! port of the host (`FormWrite`): `move` and `create` of the form, and the three gestures on a column (`column.create`,
//! `column.rename`, `column.limit`), each asked of the reader by a dialog the board brings itself. A channel the host does not
//! allow leaves its control locked; after an accepted call the form is read again (`DrawCtx::reread`) and the board drawn anew.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (`BoardSource` and `BoardEvents` over `FormInput` and `FormWrite`).

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::{FormWrite, WriteArg};
use fs_components_fields::{Look, Say};
use fs_components_widgets::dialogs::{Labels, modal};
use fs_components_words::Localized;

use crate::KanbanBoard;
use crate::ask::{Ask, Text, Words};
use crate::form_events::{
    COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, CREATE, FormEvents, LIMIT, MOVE, NAME,
};
use crate::form_read::columns_of;
use crate::model::{Column, Operation};
use crate::source::{Answer, BoardEvents, BoardSource};

/// The least height of the board, in pixels.
pub const MIN_H: f32 = 380.0;

/// The widget this view is (`widget` of the block `board`).
pub const WIDGET: &str = "fs-components-kanban";

/// The view of the block `board`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        let say: Arc<dyn Say> = Arc::new({
            let say = ctx.say.clone();
            move |key: &str| say(key)
        });
        let wiring = Arc::new(Wiring {
            input: Mutex::new(input),
            write: ctx.write.clone(),
            reread: ctx.reread.clone(),
            ask: Arc::default(),
            say: say.clone(),
            labels: Labels::for_locale(&ctx.locale),
        });
        let events = Arc::new(Events(wiring.clone()));
        let board =
            KanbanBoard::new(wiring.clone(), say, Look::new(ctx.look.0, ctx.look.1)).events(events);
        Box::new(MountedBoard { board, wiring })
    }
}

/// The board with its state; kept by the host over a rebuild.
struct MountedBoard {
    board: KanbanBoard,
    wiring: Arc<Wiring>,
}

impl Mounted for MountedBoard {
    fn build(&self, width: f32, height: f32) -> Div {
        let height = height.max(MIN_H);
        let (board, ask) = (self.board.clone(), self.wiring.ask.clone());
        // the question to the reader (a column name) stands over the board while it is open
        let area = self.wiring.ask.live.region("ask", move || {
            let page = board.build(width, height);
            match ask.current() {
                Some(dialog) => {
                    dialog.focus();
                    modal(page, dialog.build())
                }
                None => div().w(width).h(height).child(page),
            }
        });
        div().w(width).h(height).child(area)
    }

    fn refresh(&self, input: FormInput) {
        *self
            .wiring
            .input
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = input;
    }
}

/// What the board needs of its host and of the form.
struct Wiring {
    input: Mutex<FormInput>,
    write: Arc<dyn FormWrite>,
    reread: Arc<dyn Fn() -> FormInput + Send + Sync>,
    ask: Arc<Ask>,
    say: Arc<dyn Say>,
    labels: Labels,
}

impl Wiring {
    /// After an accepted call: the form is read again and the board drawn anew.
    fn done(&self, answer: Answer) -> Answer {
        if answer == Answer::Accepted {
            *self.input.lock().unwrap_or_else(PoisonError::into_inner) = (self.reread)();
            self.ask.live.refresh_now();
        }
        answer
    }

    fn send(&self, channel: &str, record: Option<&str>, args: &[WriteArg]) -> Answer {
        match self.write.send(channel, record, args) {
            Ok(_) => self.done(Answer::Accepted),
            Err(refused) => Answer::Refused(refused.code),
        }
    }

    /// A refusal as the sentence the dialog shows (`kanban.refused.<code>`).
    fn said(&self, answer: &Answer) -> Result<(), String> {
        match answer {
            Answer::Accepted => Ok(()),
            Answer::Refused(code) => Err(self.say.say(&format!("kanban.refused.{code}"))),
        }
    }

    fn column(&self, key: &str) -> Option<Column> {
        columns_of(&self.input.lock().unwrap_or_else(PoisonError::into_inner))
            .into_iter()
            .find(|c| c.key == key)
    }
}

impl BoardSource for Wiring {
    fn columns(&self) -> Vec<Column> {
        columns_of(&self.input.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn supports(&self, op: Operation) -> bool {
        let channel = match op {
            Operation::Move => MOVE,
            Operation::AddCard => CREATE,
            Operation::AddColumn => COLUMN_CREATE,
            Operation::Rename => COLUMN_RENAME,
            Operation::Limit => COLUMN_LIMIT,
        };
        self.write.can(channel, None)
    }
}

/// The gestures of the board: the cards through [`FormEvents`], the columns through dialogs.
struct Events(Arc<Wiring>);

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
}

impl BoardEvents for Events {
    fn move_card_at(&self, key: &str, from: &str, to: &str, index: Option<usize>) -> Answer {
        if from == to && index.is_none() {
            return Answer::Refused("unsupported".into());
        }
        let answer = FormEvents::new(self.0.write.clone(), self.0.clone()).move_card_at(key, from, to, index);
        self.0.done(answer)
    }

    fn move_card(&self, key: &str, from: &str, to: &str) -> Answer {
        self.move_card_at(key, from, to, None)
    }

    fn add_card(&self, column: &str, title: &str) -> Answer {
        let answer = FormEvents::new(self.0.write.clone(), self.0.clone()).add_card(column, title);
        self.0.done(answer)
    }

    fn add_column(&self) {
        if !self.0.supports(Operation::AddColumn) {
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
            w.said(&w.send(COLUMN_CREATE, None, &[WriteArg::text(NAME, name)]))
        });
    }

    fn rename_column(&self, column: &str) {
        if !self.0.supports(Operation::Rename) {
            return;
        }
        let (w, column) = (self.0.clone(), column.to_owned());
        let initial = self.0.column(&column).map(|c| c.title).unwrap_or_default();
        let keys = ("rename-column", "kanban.rename", "kanban.column_name", None);
        self.ask_text(keys, &initial, move |name| {
            w.said(&w.send(COLUMN_RENAME, Some(&column), &[WriteArg::text(NAME, name)]))
        });
    }

    fn set_limit(&self, column: &str) {
        if !self.0.supports(Operation::Limit) {
            return;
        }
        let (w, column) = (self.0.clone(), column.to_owned());
        let initial = self
            .0
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
                w.said(&w.send(COLUMN_LIMIT, Some(&column), &[WriteArg::number(LIMIT, n)]))
            }
            // not a number: the dialog stays open and says so
            Err(_) => Err(w.say.say("kanban.limit_invalid")),
        });
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

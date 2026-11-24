//! The board as a [`FormView`]: it reads the form `cards-in-columns` by channel ([`columns_of`]) and writes only through the write
//! port of the host (`FormWrite`): `move` and `create` of the form, and the gestures on a column (`column.create`,
//! `column.rename`, `column.limit`), each asked of the reader by a dialog the board brings itself. A channel the host does not
//! allow leaves its control locked; after an accepted call the form is read again (`DrawCtx::reread`) and the board drawn anew.
//! What is drawn is the one board element, [`BoardBench`]: with the mouse a card is carried to another column.
//!
//! Pattern: Abstract Factory (`View`) + Ports and Adapters (the port is `board::BoardSource` / `board::BoardEvents`, the adapters
//! are [`FormSource`] and [`FormEvents`]); the dialogs are Strategy objects of `ask.rs`. The host must put `board::styles()` in the theme.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::{FormWrite, WriteArg};
use fs_components_fields::Say;
use fs_components_widgets::dialogs::{Labels, modal};
use fs_components_words::Localized;

use crate::ask::{Ask, Text, Words};
use crate::board::{
    Answer, BoardBench, BoardEvents, ColumnAction, Labels as BoardLabels, MoveRequest, NewCard,
};
use crate::form_events::{COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, FormEvents, LIMIT, NAME};
use crate::form_source::{FormSource, reason};

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
        let source = Arc::new(FormSource::new(input, ctx.write.clone(), say.clone()));
        let wiring = Arc::new(Wiring {
            source: source.clone(),
            write: ctx.write.clone(),
            reread: ctx.reread.clone(),
            ask: Arc::default(),
            say: say.clone(),
            labels: Labels::for_locale(&ctx.locale),
            cards: Arc::new(FormEvents::new(
                ctx.write.clone(),
                source.clone(),
                say.clone(),
            )),
        });
        let events = Arc::new(Events(wiring.clone()));
        let board = BoardBench::new(source, BoardLabels::for_locale(&ctx.locale)).events(events);
        Box::new(MountedBoard { board, wiring })
    }
}

/// The board with its state; kept by the host over a rebuild.
struct MountedBoard {
    board: BoardBench,
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
        self.wiring.source.set(input);
        self.board.sync();
    }
}

/// What the board needs of its host and of the form.
struct Wiring {
    source: Arc<FormSource>,
    write: Arc<dyn FormWrite>,
    reread: Arc<dyn Fn() -> FormInput + Send + Sync>,
    ask: Arc<Ask>,
    say: Arc<dyn Say>,
    labels: Labels,
    /// The adapter that sends moves and new cards.
    cards: Arc<FormEvents>,
}

impl Wiring {
    /// After an accepted call: the form is read again and the board drawn anew.
    fn done(&self, answer: Answer) -> Answer {
        if answer == Answer::Accepted {
            self.source.set((self.reread)());
            self.ask.live.refresh_now();
        }
        answer
    }

    /// A column gesture as the sentence the dialog shows when it is refused.
    fn send(&self, channel: &str, record: Option<&str>, args: &[WriteArg]) -> Result<(), String> {
        match self.write.send(channel, record, args) {
            Ok(_) => {
                self.done(Answer::Accepted);
                Ok(())
            }
            Err(refused) => Err(reason(self.say.as_ref(), &refused.code).text),
        }
    }

    fn can(&self, channel: &str) -> bool {
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
            // the table of columns is not here yet: the one thing the form offers is a new column
            ColumnAction::EditColumns => self.add_column(),
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

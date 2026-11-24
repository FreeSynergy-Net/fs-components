//! The board as a [`FormView`]: it reads the form `cards-in-columns` by channel ([`columns_of`]) and writes only through the write
//! port of the host (`FormWrite`): `move` and `create` of the form, and the gestures on a column (`column.create`,
//! `column.rename`, `column.limit`), each asked of the reader by a dialog the board brings itself. A channel the host does not
//! allow leaves its control locked; after an accepted call the form is read again (`DrawCtx::reread`) and the board drawn anew.
//! What is drawn is the one board element, [`BoardBench`]: with the mouse a card is carried to another column.
//!
//! Pattern: Abstract Factory (`View`) + Ports and Adapters (the port is `board::BoardSource` / `board::BoardEvents`, the adapters
//! are [`FormSource`] and [`FormEvents`]); the dialogs are Strategy objects of `ask.rs`. The host must put `board::styles()` in the theme.

use std::sync::{Arc, OnceLock};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::form_write::{FormWrite, WriteArg};
use fs_components_fields::{Look, Say};
use fs_components_widgets::dialogs::{Labels, modal};
use fs_components_words::Localized;

use crate::ask::Ask;
use crate::board::{Answer, BoardBench, Labels as BoardLabels};
use crate::form_dialogs::Events;
use crate::form_events::FormEvents;
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
        Box::new(assemble(input, &ctx))
    }
}

/// The board over `input` with its wiring (apart from `mount` so the tests reach the inside).
fn assemble(input: FormInput, ctx: &DrawCtx) -> MountedBoard {
    let say: Arc<dyn Say> = Arc::new({
        let say = ctx.say.clone();
        move |key: &str| say(key)
    });
    let source = Arc::new(FormSource::new(input, ctx.write.clone(), say.clone()));
    let words = BoardLabels::for_locale(&ctx.locale);
    let wiring = Arc::new(Wiring {
        source: source.clone(),
        write: ctx.write.clone(),
        reread: ctx.reread.clone(),
        ask: Arc::default(),
        say: say.clone(),
        labels: Labels::for_locale(&ctx.locale),
        column_menu: words.column_menu.clone(),
        cards: Arc::new(FormEvents::new(ctx.write.clone(), source.clone(), say)),
        board: OnceLock::new(),
    });
    let events = Arc::new(Events(wiring.clone()));
    let board = BoardBench::new(source, words)
        .events(events)
        .look(Look::new(ctx.look.0, ctx.look.1));
    // the wiring draws the board anew after an accepted call, so it holds the board (a clone shares the state)
    let _ = wiring.board.set(board.clone());
    MountedBoard { board, wiring }
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
        // the question to the reader (a column name, what to do with a column) stands over the board while it is open
        let area = self.wiring.ask.live.region("ask", move || {
            let page = board.build(width, height);
            if let Some(choice) = ask.current_choice() {
                return modal(page, choice.build());
            }
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
pub(crate) struct Wiring {
    pub(crate) source: Arc<FormSource>,
    pub(crate) write: Arc<dyn FormWrite>,
    reread: Arc<dyn Fn() -> FormInput + Send + Sync>,
    pub(crate) ask: Arc<Ask>,
    pub(crate) say: Arc<dyn Say>,
    pub(crate) labels: Labels,
    /// The title of the choice behind the "..." button of a column.
    pub(crate) column_menu: String,
    /// The adapter that sends moves and new cards.
    pub(crate) cards: Arc<FormEvents>,
    /// The board this wiring belongs to (set once, right after it is built).
    board: OnceLock<BoardBench>,
}

impl Wiring {
    /// After an accepted call: the form is read again and the board drawn anew (a moved card finds its place itself, a changed
    /// column does not: the board reads its source again, which is what `sync` does when the generation moved).
    pub(crate) fn done(&self, answer: Answer) -> Answer {
        if answer == Answer::Accepted {
            self.source.set((self.reread)());
            if let Some(board) = self.board.get() {
                board.sync();
            }
            self.ask.live.refresh_now();
        }
        answer
    }

    /// A column gesture as the sentence the dialog shows when it is refused.
    pub(crate) fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<(), String> {
        match self.write.send(channel, record, args) {
            Ok(_) => {
                self.done(Answer::Accepted);
                Ok(())
            }
            Err(refused) => Err(reason(self.say.as_ref(), &refused.code).text),
        }
    }

    pub(crate) fn can(&self, channel: &str) -> bool {
        self.write.can(channel, None)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use fs_components_core::form_read::{ChannelInfo, FormRow};
    use fs_components_core::form_write::{WriteDone, WriteRefused, WriteValue};
    use fs_components_core::states::Load;
    use fs_components_styles::StyleId;

    use super::*;
    use crate::board::{BoardEvents, ColumnAction};

    /// A module that keeps the column names: a rename takes the new name and a `!`.
    struct Module(Mutex<FormInput>);

    impl FormWrite for Module {
        fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
            true
        }

        fn send(
            &self,
            _channel: &str,
            _record: Option<&str>,
            args: &[WriteArg],
        ) -> Result<WriteDone, WriteRefused> {
            if let Some(WriteValue::Text(name)) = args.first().map(|a| &a.value) {
                self.0.lock().unwrap().channels[0].choices[0].1 = format!("{name}!");
            }
            Ok(WriteDone::default())
        }
    }

    fn input() -> FormInput {
        FormInput {
            form: "cards-in-columns@1.2.0".into(),
            rows: vec![FormRow::of("a", &[("title", "a"), ("column", "todo")])],
            channels: vec![ChannelInfo {
                name: "column".into(),
                fields: Vec::new(),
                choices: vec![("todo".into(), "To do".into())],
            }],
            may: Vec::new(),
            load: Load::Ready,
        }
    }

    #[test]
    fn a_column_dialog_that_was_taken_shows_the_new_name_at_once() {
        let module = Arc::new(Module(Mutex::new(input())));
        let again = module.clone();
        let ctx = DrawCtx {
            look: (StyleId::Admin, false),
            say: Arc::new(str::to_owned),
            locale: fs_components_words::Locale::english(),
            write: module,
            reread: Arc::new(move || again.0.lock().unwrap().clone()),
        };
        let mounted = assemble(input(), &ctx);
        let title = |m: &MountedBoard| m.board.read(|s| s.heads()[0].title.clone());
        assert_eq!(title(&mounted), "To do");
        Events(mounted.wiring.clone()).column_action("todo", ColumnAction::Rename);
        let dialog = mounted.wiring.ask.current().expect("the line is asked");
        assert!(dialog.confirm());
        assert_eq!(title(&mounted), "To do!", "the board read the form again");
        assert!(mounted.wiring.ask.current().is_none(), "the line is closed");
    }
}

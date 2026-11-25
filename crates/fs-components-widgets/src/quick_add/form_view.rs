//! The block `quick-add` as a [`FormView`] of the form `checklist@^1.1.0`: the always-present row `+ [ add ... ]` that makes the
//! next entry. Enter sends the command `create` (channel `label`) for the typed line; Shift+Enter puts it at the top. More than one
//! line asks "N entries?" first and then sends exactly that many `create`.
//!
//! Pattern: Adapter (typed text in, `create` requests out through the port) over the pure rule [`submit`]. The row is locked
//! with its reason while the host does not allow `create` (not in `may`, or the port says no); a refusal puts the text back.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_core::focus::FocusTo;
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted, said_or};
use fs_components_core::form_write::WriteArg;

use super::{Quick, submit};
use crate::plan::{Entry, redraw};

/// The widget this view is (`widget` of the block `quick-add`).
pub const WIDGET: &str = "QuickAdd";
/// The command of the form that makes an entry.
pub const CREATE: &str = "create";
const YES: &str = "quick-add-yes";
const NO: &str = "quick-add-no";
const ENTRY: &str = "quick-add-entry";

/// The view of the block `quick-add`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        Box::new(QuickForm::new(&input, ctx))
    }
}

struct Held {
    ctx: DrawCtx,
    may_create: bool,
    /// The titles waiting for the answer to "N entries?" (in the order to send).
    pending: Option<Vec<String>>,
    /// Asks the host to give the keyboard to an element (the question's "Yes", then the line again).
    focus_to: Option<FocusTo>,
}

/// The mounted row.
#[derive(Clone)]
pub struct QuickForm {
    held: Arc<Mutex<Held>>,
    entry: Entry,
}

impl QuickForm {
    /// The row over `input` (its `may` says whether `create` is allowed) and `ctx`.
    #[must_use]
    pub fn new(input: &FormInput, ctx: DrawCtx) -> Self {
        let entry = Entry::new(&said_or(
            (ctx.say)("quick-add.placeholder"),
            "quick-add.placeholder",
            "Add",
        ));
        Self {
            held: Arc::new(Mutex::new(Held {
                ctx,
                may_create: input.may.iter().any(|m| m == CREATE),
                pending: None,
                focus_to: None,
            })),
            entry,
        }
    }

    /// The host that moves the keyboard: to "Yes" when the question stands, back to the line after the answer (WAI-ARIA APG, dialog).
    #[must_use]
    pub fn focus_to(self, host: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.held().focus_to = Some(Arc::new(host));
        self
    }

    fn focus(&self, id: &str) {
        let host = self.held().focus_to.clone();
        if let Some(host) = host {
            host(id);
        }
    }

    fn held(&self) -> MutexGuard<'_, Held> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether the row may be used now.
    #[must_use]
    pub fn allowed(&self) -> bool {
        let held = self.held();
        held.may_create && held.ctx.write.can(CREATE, None)
    }

    /// Enter (or Shift+Enter: `top`) with `typed`. One line goes out now; more ask first ([`QuickForm::answer`]).
    pub fn enter(&self, typed: &str, top: bool) {
        if !self.allowed() {
            return;
        }
        match submit(typed, top) {
            Quick::Nothing => {}
            Quick::Create(titles) => self.send(&titles),
            Quick::Confirm(titles) => {
                self.held().pending = Some(titles);
                redraw();
                self.focus(YES);
            }
        }
    }

    /// The answer to "N entries?": yes sends all of them, no lets them go.
    pub fn answer(&self, yes: bool) {
        let Some(titles) = self.held().pending.take() else {
            return;
        };
        if yes {
            self.send(&titles);
        }
        redraw();
        self.focus(ENTRY);
    }

    /// One `create` per title, in order; the first refusal stops, says why and gives the rest back to the line.
    fn send(&self, titles: &[String]) {
        let write = self.held().ctx.write.clone();
        for (i, title) in titles.iter().enumerate() {
            if let Err(refused) = write.send(CREATE, None, &[WriteArg::text("label", title)]) {
                let key = format!("quick-add.refused.{}", refused.code);
                let said = (self.held().ctx.say)(&key);
                self.entry
                    .set_notice(Some(&said_or(said, &key, &refused.code)));
                self.entry.set(&titles[i..].join("\n"));
                return;
            }
        }
    }

    fn word(&self, key: &str, fallback: &str) -> String {
        let said = (self.held().ctx.say)(key);
        said_or(said, key, fallback)
    }

    /// A real button (Tab, Enter and Space press it), like the banner and the board.
    fn button(&self, id: &str, label: String, yes: bool) -> Div {
        let me = self.clone();
        let variant = if yes {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        };
        div().w_fit().child(
            ButtonBuilder::with_key(id.to_owned(), label)
                .variant(variant)
                .on_click(move |_| me.answer(yes))
                .build_component()
                .id(id),
        )
    }
}

impl Mounted for QuickForm {
    fn build(&self, width: f32, _height: f32) -> Div {
        let mut root = div().class("fs-plan-add").flex_row().w(width);
        if !self.allowed() {
            let why = self.word("quick-add.locked", "Adding is not allowed here");
            return root
                .id("quick-add-locked")
                .attr("aria-readonly", "true")
                .child(text("+").class("fs-plan-add-plus"))
                .child(text(why).class("fs-plan-entry-notice").id("quick-add-why"));
        }
        root = root.child(text("+").class("fs-plan-add-plus"));
        let pending = self.held().pending.clone();
        if let Some(titles) = pending {
            let ask = format!(
                "{} {}",
                titles.len(),
                self.word("quick-add.confirm", "entries?")
            );
            let no = self.clone();
            // Escape means No, wherever the keyboard is while the question stands (the frame of the dialogs does the same).
            return root
                .id("quick-add-question")
                .child(text(ask).id("quick-add-ask"))
                .child(self.button(YES, self.word("quick-add.yes", "Yes"), true))
                .child(self.button(NO, self.word("quick-add.no", "No"), false))
                .on_key_down(move |ctx| {
                    if ctx.key_code == 27 {
                        no.answer(false);
                    }
                });
        }
        let me = self.clone();
        root.child(self.entry.build_with(
            ENTRY,
            Arc::new(move |typed: &str, top: bool| me.enter(typed, top)),
        ))
    }

    fn refresh(&self, input: FormInput) {
        self.held().may_create = input.may.iter().any(|m| m == CREATE);
        redraw();
    }
}

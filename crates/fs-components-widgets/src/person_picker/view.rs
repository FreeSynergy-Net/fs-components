//! The picker as elements.
//!
//! Pattern: Adapter. The state in, Blinc layout, `blinc_cn` buttons, our search field and the state surfaces (`states`) out.
//! The search field sits outside the part that is drawn again, so typing never loses focus. Every state of the list has a
//! surface: loading, failed (code and parameter, with "Try again"), nobody, nobody found; nothing is left empty and silent.
//! Reading aloud is not there yet (Blinc has no bridge to screen readers on the desktop); the keyboard works fully.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::handle::PersonPicker;
use super::model::{Candidate, Outcome, PersonRef};
use super::state::{Notice, Phase, Picking};
use crate::Element;
use crate::keys::bind_keys;
use crate::states::{Alert, Skeleton};

impl PersonPicker {
    /// The element to place in a layout (the frontend puts it in a modal).
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        let mut head = div()
            .class("fs-pp-head")
            .child(text(self.labels.title.clone()).class("fs-pp-title"));
        if self.allow_me {
            head = head.child(self.action("me", &self.labels.me, ButtonVariant::Outline));
        }
        let root = div()
            .id(&self.id)
            .class("fs-pp")
            .child(head)
            .child(self.field.build())
            .child(self.core.live.view("picker", move || this.render()));
        if self.bindings.is_none() {
            return root;
        }
        let this = self.clone();
        bind_keys(
            root,
            Arc::new(move |event| this.press(&event) != Outcome::Ignored),
        )
    }

    fn render(&self) -> Div {
        self.core.read(|state| {
            let l = &self.labels;
            let mut list = div().class("fs-pp-list");
            let mut drawn: Vec<&PersonRef> = Vec::new();
            match state.phase() {
                Phase::Loading => {
                    list = list.child(
                        Skeleton::new(&l.states.loading)
                            .id(&format!("{}-loading", self.id))
                            .rows(3)
                            .build(),
                    );
                }
                Phase::Failed(error) => {
                    let this = self.clone();
                    list = list.child(
                        Alert::failed(&l.states, &l.error(error), move || {
                            this.refresh();
                        })
                        .id(&format!("{}-error", self.id))
                        .build(),
                    );
                }
                Phase::Ready(page) => {
                    for c in &page.items {
                        list = list.child(self.row_of(state, c, &l.unnamed));
                        drawn.push(&c.reference);
                    }
                    if page.items.is_empty() {
                        let (word, tag) = if state.query().trim().is_empty() {
                            (&l.nothing_offered, "nothing")
                        } else {
                            (&l.none_found, "none")
                        };
                        list = list.child(
                            text(word.clone())
                                .class("fs-pp-note")
                                .id(format!("{}-{tag}", self.id)),
                        );
                    }
                    if page.more {
                        list = list.child(
                            text(l.more.clone())
                                .class("fs-pp-note")
                                .id(format!("{}-more", self.id)),
                        );
                    }
                }
            }
            // Chosen people the current answer does not show stay visible and can be taken away.
            // Without a name: after a failed answer it could not be loaded; otherwise the source does not release it.
            let missing = if matches!(state.phase(), Phase::Failed(_)) {
                &l.unloadable
            } else {
                &l.unnamed
            };
            for p in state.chosen().iter().filter(|p| !drawn.contains(p)) {
                let c = Candidate {
                    display: state.source.display_of(p),
                    ..Candidate::new(p.clone())
                };
                list = list.child(self.row_of(state, &c, missing));
            }
            if let Some(notice) = state.notice() {
                let word = match notice {
                    Notice::NoMe => l.me_unset.clone(),
                    Notice::Error(e) => l.error(e),
                };
                list = list.child(
                    text(word)
                        .class("fs-pp-notice")
                        .id(format!("{}-notice", self.id)),
                );
            }
            let buttons = div()
                .class("fs-pp-buttons")
                .child(self.action("cancel", &l.cancel, ButtonVariant::Ghost))
                .child(self.action("confirm", &l.confirm, ButtonVariant::Primary));
            div().w_full().flex_col().gap_px(12.0).child(list).child(
                div()
                    .class("fs-pp-foot")
                    .child(text(l.chosen(state.chosen().len())).class("fs-pp-note"))
                    .child(buttons),
            )
        })
    }

    fn row_of(&self, state: &Picking, c: &Candidate, missing: &str) -> Div {
        let on = state.chosen().contains(&c.reference);
        let cursor = state.cursor() == Some(&c.reference);
        let mut row =
            div()
                .class("fs-pp-row")
                .id(format!("{}-row-{}", self.id, c.reference.as_str()));
        if on {
            row = row.class("fs-pp-row-on");
        }
        if cursor {
            row = row.class("fs-pp-row-cursor");
        }
        let mark = if on {
            div().class("fs-pp-mark").class("fs-pp-mark-on")
        } else {
            div().class("fs-pp-mark")
        };
        // What the source releases, and nothing else: no name means "name not shown", never a guess.
        let name = c
            .display
            .clone()
            .unwrap_or_else(|| missing.to_owned());
        let mut who = div()
            .class("fs-pp-who")
            .child(text(name).class("fs-pp-name").id(format!(
                "{}-name-{}",
                self.id,
                c.reference.as_str()
            )));
        if let Some(detail) = c.detail.as_ref().filter(|d| !d.is_empty()) {
            who = who.child(text(detail.clone()).class("fs-pp-meta"));
        }
        let (this, person) = (self.clone(), c.reference.clone());
        let pick = div()
            .class("fs-pp-pick")
            .id(format!("{}-pick-{}", self.id, c.reference.as_str()))
            .child(mark)
            .child(who)
            .on_click(move |_| {
                this.toggle(&person);
            });
        row.child(pick)
    }

    fn action(&self, tag: &str, word: &str, variant: ButtonVariant) -> impl Element + use<> {
        let id = format!("{}-{tag}", self.id);
        let this = self.clone();
        let tag = tag.to_string();
        ButtonBuilder::with_key(id.clone(), word.to_string())
            .variant(variant)
            .on_click(move |_| match tag.as_str() {
                "me" => {
                    this.choose_me();
                }
                "confirm" => {
                    this.confirm();
                }
                _ => {
                    this.cancel();
                }
            })
            .build_component()
            .id(&id)
    }
}

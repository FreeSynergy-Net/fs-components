//! `switch`: on, off, or undetermined ("some of them"); a person flips it with one gesture and the effect is immediate.
//!
//! Pattern: State (`SwitchState`, pure) + Observer (`Live`). Space, Enter or a click flips it: off to on, on to off, undetermined
//! to on. A disabled switch is no tab stop and ignores everything. The state is told by position AND colour (the thumb sits left,
//! centre or right), never by colour alone.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_widgets::Element;
use fs_components_widgets::keys::Key;
use fs_components_widgets::live::Live;

use super::activatable;

/// The three states of a switch.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchState {
    /// Off.
    Off,
    /// On.
    On,
    /// Some of the things it stands for are on and some are off.
    Mixed,
}

impl SwitchState {
    /// The name of the state in the theme contract (`checked`, `unchecked` is the default, `mixed`).
    pub const fn class(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::On => "on",
            Self::Mixed => "mixed",
        }
    }

    /// What a flip gives.
    #[must_use]
    pub const fn flipped(self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off | Self::Mixed => Self::On,
        }
    }
}

type OnChange = Arc<dyn Fn(SwitchState) + Send + Sync>;

/// The switch. Clones share the state and the callback.
#[derive(Clone)]
pub struct Switch {
    id: String,
    label: String,
    state: Arc<Mutex<SwitchState>>,
    disabled: bool,
    live: Live,
    on_change: Option<OnChange>,
}

impl Switch {
    /// An off switch with a `label`.
    pub fn new(label: &str) -> Self {
        Self {
            id: "switch".to_owned(),
            label: label.to_owned(),
            state: Arc::new(Mutex::new(SwitchState::Off)),
            disabled: false,
            live: Live::new(),
            on_change: None,
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The state to start with.
    #[must_use]
    pub fn state_is(self, state: SwitchState) -> Self {
        self.set(state);
        self
    }

    /// Cannot be changed.
    #[must_use]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Call `handler` with the new state after a flip by the person.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(SwitchState) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// The state now.
    pub fn state(&self) -> SwitchState {
        *self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Set the state from the program (no callback).
    pub fn set(&self, state: SwitchState) {
        *self.state.lock().unwrap_or_else(PoisonError::into_inner) = state;
        self.live.refresh_now();
    }

    /// Flip it as the person does; false (nothing happens) when disabled.
    pub fn flip(&self) -> bool {
        if self.disabled {
            return false;
        }
        let next = self.state().flipped();
        self.set(next);
        if let Some(handler) = &self.on_change {
            handler(next);
        }
        true
    }

    /// The element.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        let flip = self.clone();
        let root = self.live.region("switch", move || this.render());
        let outer = div().id(format!("{}-hit", self.id)).class("fs-switch-hit");
        if self.disabled {
            return outer.attr("disabled", "true").child(root);
        }
        let keys = self.clone();
        activatable(
            outer.child(root),
            Arc::new(move |event| {
                if matches!(event.key, Key::Activate | Key::Enter | Key::Space) {
                    keys.flip()
                } else {
                    false
                }
            }),
            Arc::new(move || {
                flip.flip();
            }),
        )
    }

    fn render(&self) -> Div {
        let state = self.state();
        let checked = match state {
            SwitchState::Off => "false",
            SwitchState::On => "true",
            SwitchState::Mixed => "mixed",
        };
        let mut root = div()
            .id(&self.id)
            .class("fs-switch")
            .class(format!("fs-switch-{}", state.class()))
            .attr("role", "switch")
            .attr("aria-checked", checked);
        if self.disabled {
            root = root.class("fs-switch-disabled");
        }
        root.child(
            div()
                .id(format!("{}-track", self.id))
                .class("fs-switch-track")
                .child(
                    div()
                        .id(format!("{}-thumb", self.id))
                        .class("fs-switch-thumb"),
                ),
        )
        .child(
            text(self.label.clone())
                .id(format!("{}-label", self.id))
                .class("fs-switch-label"),
        )
    }
}

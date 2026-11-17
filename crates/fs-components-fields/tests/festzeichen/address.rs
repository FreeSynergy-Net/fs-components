//! P1 probe: ONE field `local @ server . tld` made of three Blinc text inputs (`InputType::Text`) in one frame, the `@` and the last `.`
//! drawn as fixed signs between them. One focus and keyboard model sits on top; what Blinc's text input does not give is done here.
//!
//! Tests: `festzeichen.rs`. A `Mutation` switches one rule off or wrong so that the test of that rule must turn red.

use std::sync::{Arc, Mutex};

use blinc_layout::widgets::text_input::{
    InputType, SharedTextInputData, focus_text_input, refresh_text_input,
};
use fs_components_primitives::FieldData;
use fs_types::ContactValue;
use fs_types::values::EmailAddress;

use super::split::Change;

/// One rule made wrong, for the mutant tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutation {
    Intact,
    /// A part uses Blinc's `InputType::Email` (third source of the rule).
    BlincEmailType,
    /// A typed `@` stays in the text next to the sign that becomes solid (two `@`), no jump.
    AtIsText,
    /// A pasted address is cut at the first dot.
    FirstDot,
    /// Backspace in an EMPTY part does not go back (Blinc's own `on_backspace_empty` is not used: see `key_up`).
    NoBackJump,
    /// Backspace at the start of a filled part does not go back over the fixed sign.
    NoBackspaceJump,
    /// A `.` typed into a filled tld is swallowed instead of moving the tld into the server.
    TldDotSwallowed,
    /// Arrow keys at the edge of a part do not cross the fixed sign.
    NoArrowJump,
    /// A dot in the middle of the server jumps (and takes the rest along).
    MidDotJumps,
    /// Copy gives only the part that has the cursor.
    CopyPartOnly,
    /// The fixed dot is left out when the parts are put together.
    JoinWithoutDot,
    /// Copy includes the grey ghost text (signs not yet taken and the suggested ending).
    GhostInCopy,
    /// Taking the suggestion appends the whole suggestion to what is typed.
    AcceptDouble,
}

/// The state shared by the handlers of the three parts.
#[derive(Default)]
pub struct State {
    pub hint: Option<&'static str>,
    pub shown: Vec<Change>,
    /// Cursor and length of each part as they were BEFORE the key now pressed (refreshed at key release and mouse release): the order in which
    /// Blinc hands one key to the handlers of the parts and of the frame is not fixed (a HashMap), so the edge jump waits for the release.
    pub snap: [(usize, usize); 3],
    /// Length of each part at the last change, to see that a character was inserted.
    pub lens: [usize; 3],
    pub clip: Vec<String>,
    /// The `@` and the last `.` are grey ghosts until taken (typed, or taken by the accept key), then solid.
    pub real: [bool; 2],
    /// Every key event the frame saw: ('d' or 'u', code).
    pub seen: Vec<(char, u32)>,
}

/// The field.
pub struct AddressField {
    pub segs: [SharedTextInputData; 3],
    /// Keeps the data of the inputs alive (the one door for field data, see `fs-components-primitives`).
    _keep: FieldData,
    pub st: Arc<Mutex<State>>,
    pub mutation: Mutation,
    /// All endings that may be suggested (the caller must give the whole list).
    pub tlds: Vec<&'static str>,
}

pub const IDS: [&str; 3] = ["addr-local", "addr-server", "addr-tld"];

pub(super) fn chars(s: &str) -> usize {
    s.chars().count()
}

impl AddressField {
    /// `tlds` is required: an empty list is an error (no default list).
    pub fn new(mutation: Mutation, tlds: &[&'static str]) -> Result<Arc<Self>, String> {
        if tlds.is_empty() {
            return Err("the list of endings is required".into());
        }
        let ty = if mutation == Mutation::BlincEmailType {
            InputType::Email
        } else {
            InputType::Text
        };
        let keep = FieldData::new();
        let segs = [keep.text_input(), keep.text_input(), keep.text_input()];
        for s in &segs {
            s.lock().unwrap().input_type = ty;
        }
        Ok(Arc::new(Self {
            segs,
            _keep: keep,
            st: Arc::new(Mutex::new(State::default())),
            mutation,
            tlds: tlds.to_vec(),
        }))
    }

    pub fn value(&self, i: usize) -> String {
        self.segs[i].lock().unwrap().value.clone()
    }

    pub fn cursor(&self, i: usize) -> usize {
        self.segs[i].lock().unwrap().cursor
    }

    /// Which part has the keyboard.
    pub fn focus(&self) -> Option<usize> {
        self.segs
            .iter()
            .position(|s| s.lock().unwrap().visual.is_focused())
    }

    pub fn go(&self, i: usize, cursor: Option<usize>) {
        focus_text_input(&self.segs[i]);
        let value = self.value(i);
        self.set(i, &value, cursor.unwrap_or_else(|| chars(&value)));
    }

    pub(super) fn set(&self, i: usize, value: &str, cursor: usize) {
        {
            let mut g = self.segs[i].lock().unwrap();
            g.value = value.to_string();
            g.cursor = cursor;
            g.selection_start = None;
        }
        refresh_text_input(&self.segs[i]);
        self.remember(i);
    }

    /// Like Blinc inserting a character: the value changes, the snapshot does not (yet).
    pub(super) fn set_raw(&self, i: usize, value: &str, cursor: usize) {
        {
            let mut g = self.segs[i].lock().unwrap();
            g.value = value.to_string();
            g.cursor = cursor;
        }
        refresh_text_input(&self.segs[i]);
    }

    pub(super) fn remember(&self, i: usize) {
        let g = self.segs[i].lock().unwrap();
        let mut st = self.st.lock().unwrap();
        st.snap[i] = (g.cursor, chars(&g.value));
        st.lens[i] = chars(&g.value);
    }

    pub(super) fn remember_len(&self, i: usize) {
        let n = chars(&self.value(i));
        self.st.lock().unwrap().lens[i] = n;
    }

    /// The REAL text with the signs that are solid; the grey ghosts are never in it ("" while everything is empty).
    pub fn address(&self) -> String {
        let (l, sv, t) = (self.value(0), self.value(1), self.value(2));
        if l.is_empty() && sv.is_empty() && t.is_empty() {
            return String::new();
        }
        let ghosts = self.mutation == Mutation::GhostInCopy;
        let real = self.st.lock().unwrap().real;
        let at = if real[0] || ghosts { "@" } else { "" };
        let dot = if (real[1] || ghosts) && self.mutation != Mutation::JoinWithoutDot {
            "."
        } else {
            ""
        };
        let tail = if ghosts {
            self.ghost_tld()
        } else {
            String::new()
        };
        format!("{l}{at}{sv}{dot}{t}{tail}")
    }

    /// The text as `fs-types` takes it: the only rule.
    pub fn is_valid(&self) -> bool {
        EmailAddress::check(&self.address()).is_ok()
    }

    /// A character just typed into part `i` (Blinc inserted it; `d.cursor` is behind it): the `@` and the `.` are the fixed signs.
    pub(super) fn typed(&self, i: usize) {
        let prev_len = self.st.lock().unwrap().lens[i];
        let (value, cursor) = {
            let g = self.segs[i].lock().unwrap();
            (g.value.clone(), g.cursor)
        };
        let all: Vec<char> = value.chars().collect();
        if all.len() != prev_len + 1 || cursor == 0 {
            self.remember_len(i);
            return;
        }
        let c = all[cursor - 1];
        let before: String = all[..cursor - 1].iter().collect();
        let after: String = all[cursor..].iter().collect();
        let without = format!("{before}{after}");
        let hint = |h: &'static str| self.st.lock().unwrap().hint = Some(h);
        match (i, c) {
            (0, '@') if self.mutation == Mutation::AtIsText => {
                self.st.lock().unwrap().real[0] = true; // the typed `@` stays in the text AND the sign turns solid: two `@`
                self.remember_len(i);
            }
            (0, '@') => {
                self.st.lock().unwrap().real[0] = true;
                self.set(0, &before, chars(&before));
                let server = format!("{after}{}", self.value(1));
                self.set(1, &server, 0);
                self.go(1, Some(0));
            }
            (1 | 2, '@') => {
                self.set(i, &without, cursor - 1);
                hint("problem.email.many-at");
            }
            (1, '.') if before.is_empty() || before.ends_with('.') || after.starts_with('.') => {
                self.set(1, &without, cursor - 1);
                hint("problem.email.empty-label");
            }
            (1, '.') if cursor == all.len() || self.mutation == Mutation::MidDotJumps => {
                self.st.lock().unwrap().real[1] = true;
                self.set(1, &before, chars(&before));
                if !after.is_empty() {
                    let tld = format!("{after}{}", self.value(2));
                    self.set(2, &tld, 0);
                }
                self.go(2, Some(0));
            }
            (2, '.') if self.mutation == Mutation::TldDotSwallowed => {
                self.set(2, &without, cursor - 1)
            }
            (2, '.') if !without.is_empty() => {
                let server = self.value(1);
                let joined = if server.is_empty() {
                    without
                } else {
                    format!("{server}.{without}")
                };
                self.set(1, &joined, chars(&joined));
                self.set(2, "", 0);
            }
            (2, '.') => {
                self.set(2, &without, 0);
                hint("problem.email.empty-label");
            }
            _ => self.remember(i),
        }
    }
}

//! P1 probe: the keyboard side of the three-part field: ghost suggestions, pasting, key release rules, copy.
//!
//! Tests: `festzeichen.rs`.

use super::address::{AddressField, Mutation, chars};
use super::split::{DotAt, split_address};

impl AddressField {
    /// The grey rest of the suggested ending, behind what is typed ("" when none fits).
    pub fn ghost_tld(&self) -> String {
        let typed = self.value(2).to_lowercase();
        self.tlds
            .iter()
            .find(|t| t.starts_with(&typed) && t.len() > typed.len())
            .map(|t| t[typed.len()..].to_string())
            .unwrap_or_default()
    }

    /// Whether the sign after part `k` (0: `@`, 1: `.`) is solid.
    pub fn real(&self, k: usize) -> bool {
        self.st.lock().unwrap().real[k]
    }

    /// Take the grey suggestion as it is: the sign after the part becomes solid and the cursor goes on; in the ending the suggested rest is added once.
    /// Is something grey to take in part `i`: the sign after it, or the rest of the ending.
    pub fn ghost_visible(&self, i: usize) -> bool {
        if i < 2 {
            !self.value(i).is_empty() && !self.real(i)
        } else {
            !self.ghost_tld().is_empty()
        }
    }

    pub fn accept(&self, i: usize) {
        if i < 2 && !self.value(i).is_empty() {
            self.st.lock().unwrap().real[i] = true;
            self.go(i + 1, Some(0));
        } else if i == 2 {
            let g = self.ghost_tld();
            if g.is_empty() {
                return;
            }
            let typed = self.value(2);
            let v = if self.mutation == Mutation::AcceptDouble {
                format!("{typed}{typed}{g}")
            } else {
                format!("{typed}{g}")
            };
            self.set(2, &v, chars(&v));
        }
    }

    /// Type one character into part `i` as a key does (used for pasted plain text).
    pub fn type_into(&self, i: usize, c: char) {
        let (cur, value) = {
            let g = self.segs[i].lock().unwrap();
            (g.cursor, g.value.clone())
        };
        let mut all: Vec<char> = value.chars().collect();
        all.insert(cur, c);
        self.remember_len(i);
        self.set_raw(i, &all.into_iter().collect::<String>(), cur + 1);
        self.typed(i);
    }

    /// Paste into part `i`: a whole address is spread over the parts, other text goes in as typed.
    pub fn paste(&self, i: usize, text: &str) {
        let dot = if self.mutation == Mutation::FirstDot {
            DotAt::First
        } else {
            DotAt::Last
        };
        if let Some(p) = split_address(text, dot) {
            self.set(0, &p.local, chars(&p.local));
            self.set(1, &p.server, chars(&p.server));
            self.set(2, &p.tld, chars(&p.tld));
            {
                let mut st = self.st.lock().unwrap();
                st.shown = p.shown;
                st.real = [true, !p.tld.is_empty()];
            }
            self.go(if p.tld.is_empty() { 1 } else { 2 }, None);
        } else {
            text.chars()
                .filter(|c| !c.is_control())
                .for_each(|c| self.type_into(i, c));
        }
    }

    /// Ctrl+A at key press: every part is marked (the parts do the same for themselves, in any order).
    pub fn key_down(&self, code: u32, ctrl: bool) {
        self.st.lock().unwrap().seen.push(('d', code));
        if code == 65 && ctrl {
            for s in &self.segs {
                let mut g = s.lock().unwrap();
                g.selection_start = Some(0);
                g.cursor = chars(&g.value);
            }
        }
    }

    /// The release of a key: edge jumps and copy. Done here because by now every handler of the press has run, in whatever order; the snapshot
    /// holds the cursor from before the press, so "was at the edge" is known. Copy at release also overwrites what the part itself copied.
    pub fn key_up(&self, code: u32, ctrl: bool) {
        self.st.lock().unwrap().seen.push(('u', code));
        let Some(i) = self.focus() else { return };
        let (cur, len) = self.st.lock().unwrap().snap[i];
        let m = self.mutation;
        match code {
            // Enter takes the grey suggestion, but only while grey is visible (else it is left alone); Tab never takes it
            13 if self.ghost_visible(i) => self.accept(i),
            35 => {
                self.go(2, None);
                self.accept(2);
            }
            39 if i == 2 && cur == len => self.accept(2),
            37 if i > 0 && cur == 0 && len > 0 && m != Mutation::NoArrowJump => {
                self.go(i - 1, None)
            }
            8 if i > 0
                && cur == 0
                && m != Mutation::NoBackspaceJump
                && (len > 0 || m != Mutation::NoBackJump) =>
            {
                if len == 0 {
                    self.st.lock().unwrap().real[i - 1] = false; // an empty part gives its sign back to the ghost
                }
                self.go(i - 1, None);
            }
            39 | 127
                if i < 2
                    && cur == len
                    && len > 0
                    && (code == 127 || m != Mutation::NoArrowJump) =>
            {
                self.st.lock().unwrap().real[i] = true;
                self.go(i + 1, Some(0));
            }
            67 if ctrl => {
                let text = if m == Mutation::CopyPartOnly {
                    self.value(i)
                } else {
                    self.address()
                };
                self.st.lock().unwrap().clip.push(text);
            }
            _ => {}
        }
        if let Some(j) = self.focus() {
            self.remember(j);
        }
    }
}

//! What a program says to a request of a component: done, refused with a reason, or a question first.
//!
//! Pattern: Value objects (plain data, compared by value). One model for every component that asks a program to do something (a
//! card put down, a board move, a slot that cannot be shown): the same `Reason`, no conversion between them. Every sentence is
//! already in the reader's language (the program's data); the component only shows it.

/// Why something is not possible, said by the program: a code for logs and tests, a sentence for the reader. The same type as the
/// reason of a hidden or missing slot (`SlotReason` of the fields crate): one model.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reason {
    /// The code, for example `bind-error-field-hidden`.
    pub code: String,
    /// The sentence, already in the reader's language.
    pub text: String,
}

impl Reason {
    /// A reason.
    pub fn new(code: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            text: text.into(),
        }
    }

    /// The sentence the reader sees. A reason without a sentence is shown by its code: a reason never vanishes into an empty line.
    pub fn said(&self) -> &str {
        if self.text.is_empty() {
            &self.code
        } else {
            &self.text
        }
    }
}

/// A question the program asks before it does something (an agent holds the card: move it anyway?).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    /// The question.
    pub text: String,
}

impl Question {
    /// A question.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// What the program says to a request of a component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Done; the next read shows it.
    Accepted,
    /// Not done; the thing goes back and the reason is said.
    Refused(Reason),
    /// Ask first; on yes the same request comes again with `confirmed` set.
    Ask(Question),
}

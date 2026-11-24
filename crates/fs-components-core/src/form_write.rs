//! The write port of a form (SW-2, ADR 0077 section 3.3): what a component asks of the host when the reader changes something,
//! in the words of the form channels (named by channel, not by the fields of a module) (`move`, `create`, `remove`), never in the words of a program.
//!
//! The component knows two things only: whether a channel may be used now for a record (`can`: the host's answer, never a guess
//! of ours; `false` means the control is drawn locked and nothing is sent), and the call itself (`send`) with its answer. The
//! answer carries what the host made or changed (`WriteDone`, the field `result` of `FormSource.invoke`: the record and a short
//! code-like text), or a refusal as a code with parameters (`WriteRefused`), never a sentence of a foreign program.
//!
//! Pattern: Port (trait) with a Null object (`NoWrite`: nothing may be written).

/// One value of an argument. The kinds are those of the form channels (`text`, `number`, `flag`, `time`, `reference`).
#[derive(Clone, Debug, PartialEq)]
pub enum WriteValue {
    /// A text (also the id of a column or a reference).
    Text(String),
    /// A number.
    Number(f64),
    /// A flag.
    Flag(bool),
    /// A point in time, as the form writes it (RFC 3339).
    Time(String),
}

/// One argument of a call: the name of the CHANNEL of the form (e.g. `order`), never the field of a module; the host maps it (fs-forms-run, `map`), and the value.
#[derive(Clone, Debug, PartialEq)]
pub struct WriteArg {
    /// The name of the parameter.
    pub name: String,
    /// The value.
    pub value: WriteValue,
}

impl WriteArg {
    /// A text argument.
    pub fn text(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: WriteValue::Text(value.to_owned()),
        }
    }

    /// A number argument.
    pub fn number(name: &str, value: f64) -> Self {
        Self {
            name: name.to_owned(),
            value: WriteValue::Number(value),
        }
    }
}

/// What a call made or changed (`result` of `FormSource.invoke`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WriteDone {
    /// The record the command made or changed; `None` for a delete.
    pub record: Option<String>,
    /// A short text for people the host gives back (a code, never a foreign sentence); `None` = nothing to say.
    pub text: Option<String>,
}

/// The code the host answers with when the data source wants a confirmation first and the window cannot ask it. The adapter
/// (not the host) turns it into the sentence `kanban.refused.ask-unsupported`.
pub const ASK_UNSUPPORTED: &str = "ask-unsupported";

/// A refusal: a code (the tail of a translation key) and its parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteRefused {
    /// The code.
    pub code: String,
    /// The parameters, in order.
    pub params: Vec<(String, String)>,
}

impl WriteRefused {
    /// A refusal with a code and no parameters.
    pub fn new(code: &str) -> Self {
        Self {
            code: code.to_owned(),
            params: Vec::new(),
        }
    }
}

/// The write side of a form, behind the host.
pub trait FormWrite: Send + Sync {
    /// May `channel` be used now, for `record` (`None`: the channel is not about one record, like `create`)? The host decides
    /// (rights, state); `false` means: draw the control locked and send nothing.
    fn can(&self, channel: &str, record: Option<&str>) -> bool;

    /// Call `channel` for `record` with `args`.
    ///
    /// # Errors
    /// A [`WriteRefused`] when the host does not do it.
    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused>;
}

/// Writes nothing: no channel may be used, every call is refused (`unsupported`).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoWrite;

impl FormWrite for NoWrite {
    fn can(&self, _channel: &str, _record: Option<&str>) -> bool {
        false
    }

    fn send(
        &self,
        _channel: &str,
        _record: Option<&str>,
        _args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        Err(WriteRefused::new("unsupported"))
    }
}

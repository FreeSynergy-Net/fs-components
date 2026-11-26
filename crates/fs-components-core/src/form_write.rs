//! The write port of a form (SW-2): what a component asks of the host when the reader changes something,
//! in the words of the form channels (named by channel, not by the fields of a module) (`move`, `create`, `remove`), never in the words of a program.
//!
//! The component knows two things only: whether a channel may be used now for a record (`can`: the host's answer, never a guess
//! of ours; `false` means the control is drawn locked and nothing is sent), and the call itself (`send`) with its answer. The
//! answer carries what the host made or changed (`WriteDone`, the field `result` of `FormSource.invoke`: the record and a short
//! code-like text), or a refusal as a code with parameters (`WriteRefused`), never a sentence of a foreign program.
//!
//! Besides the channels there are the GESTURES (`may_fire`, `fire`, words in [`trigger`]): the component names what the reader did (`drop`),
//! the host finds the `[[action]]` that binds it (at the slot `menu` by the name of the entry). A gesture no action binds is refused with `gesture-unbound`.
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

/// The code of a gesture no `[[action]]` binds at this place (parameter `trigger`).
pub const GESTURE_UNBOUND: &str = "gesture-unbound";

/// The trigger words: the names of the command slots (`kind = "command"`) of the blocks, one list for the contract and the host.
pub mod trigger {
    /// A click without a carry, Enter or Space.
    pub const ACTIVATE: &str = "activate";
    /// The context menu: right click, Shift+F10, menu key.
    pub const MENU: &str = "menu";
    /// An entry put at a place, by pointer or by keys.
    pub const DROP: &str = "drop";
    /// A new entry right here (column foot, empty calendar cell).
    pub const QUICK_ADD: &str = "quick-add";
    /// Every trigger word, each once, in kebab case.
    pub const ALL: [&str; 4] = [ACTIVATE, MENU, DROP, QUICK_ADD];
}

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

    /// `gesture-unbound` with the parameter `trigger`.
    pub fn unbound(trigger: &str) -> Self {
        Self {
            code: GESTURE_UNBOUND.to_owned(),
            params: vec![("trigger".to_owned(), trigger.to_owned())],
        }
    }
}

/// Pattern: Port (the write side of a form behind the host) with a Null object ([`NoWrite`]); the gesture methods default to the
/// refusal `gesture-unbound`, so no adapter breaks and nothing is sent by a detour through `send`.
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

    /// May the gesture `trigger` be used now, for `record`? `Err` = draw the control locked and show the reason (unbound, no right,
    /// ...). The host gives every PLACE its own `FormWrite`, so the place is not a parameter. `entry` is the name of the action
    /// at the slot `menu` (see [`FormWrite::fire`]); for every other trigger it is `None`.
    ///
    /// # Errors
    /// The reason as a code. Default: [`WriteRefused::unbound`].
    fn may_fire(
        &self,
        trigger: &str,
        _entry: Option<&str>,
        _record: Option<&str>,
    ) -> Result<(), WriteRefused> {
        Err(WriteRefused::unbound(trigger))
    }

    /// The gesture `trigger` happened for `record` with `args` (named as the `args` of the slot in the contract). The host finds the
    /// `[[action]]` (this place, this trigger) and runs its command or opens its form. No channel name, no detour.
    ///
    /// `entry` names the action when several actions hang at the slot `menu` (one action = one entry of the context menu, the name
    /// of the action is the name of the entry); one entry may carry several actions. Without an entry the host takes the one action
    /// of the place and trigger, and refuses with `gesture-ambiguous` when there are several. Every other trigger passes `None`.
    ///
    /// # Errors
    /// Refused by the host or the module (code and parameters, also `ask-unsupported`). Default: [`WriteRefused::unbound`].
    fn fire(
        &self,
        trigger: &str,
        _entry: Option<&str>,
        _record: Option<&str>,
        _args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        Err(WriteRefused::unbound(trigger))
    }
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

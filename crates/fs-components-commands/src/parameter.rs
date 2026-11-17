//! The typed parameters and the result of a command (plan `PLAN-components-menue` 4b, decision b7 17.11.).
//!
//! Pattern: Value object. A command that takes input says what kind of input, so a menu, the palette and a block in the
//! workflow editor ask for the same thing. The type is closed on purpose: text, number, a reference to a person, a file or an
//! object, a choice, a list. A person is **only** a reference to a people entry ([`ParamType::Person`]), never a typed name; a file is a
//! reference the file picker gives, never a path typed into a text field.
//!
//! A parameter is built in two steps (`Param::new(..)`, then `.of(type)`) so that a parameter **without** a type can be written
//! down and is reported by the validation ([`crate::Problem::ParamWithoutType`]) instead of being impossible to see.

use crate::command::TextKey;

/// What kind of value a parameter or a result holds.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamType {
    /// Free text.
    Text,
    /// A number.
    Number,
    /// A reference to a person of the people directory (never a typed name).
    Person,
    /// A reference to a file (what the file picker returns).
    File,
    /// A reference to an object of a kind (`group`, `event`, ...); the kind is the name the program publishes.
    Object {
        /// The kind of object.
        kind: String,
    },
    /// One of a fixed set; each option is the key of its text.
    Choice {
        /// The options, in order; at least one.
        options: Vec<String>,
    },
    /// Several values of one type.
    List(Box<ParamType>),
}

impl ParamType {
    /// A reference to an object of this kind.
    pub fn object(kind: impl Into<String>) -> Self {
        Self::Object { kind: kind.into() }
    }

    /// A choice of these options.
    pub fn choice<S: Into<String>>(options: impl IntoIterator<Item = S>) -> Self {
        Self::Choice {
            options: options.into_iter().map(Into::into).collect(),
        }
    }

    /// A list of this type.
    #[must_use]
    pub fn list(of: Self) -> Self {
        Self::List(Box::new(of))
    }

    /// The name of the type in a block description (`text`, `person`, `object:group`, `choice(a|b)`, `list<file>`).
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Text => "text".to_owned(),
            Self::Number => "number".to_owned(),
            Self::Person => "person".to_owned(),
            Self::File => "file".to_owned(),
            Self::Object { kind } => format!("object:{kind}"),
            Self::Choice { options } => format!("choice({})", options.join("|")),
            Self::List(of) => format!("list<{}>", of.name()),
        }
    }

    /// Does the type hold something that cannot be chosen (a choice without options, at any depth)?
    pub(crate) fn is_hollow(&self) -> bool {
        match self {
            Self::Choice { options } => options.is_empty() || options.iter().any(String::is_empty),
            Self::Object { kind } => kind.is_empty(),
            Self::List(of) => of.is_hollow(),
            _ => false,
        }
    }
}

/// One parameter of a command.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    /// Its name in the command (`target`, `name`); unique within the command, not empty.
    pub name: String,
    /// The key of the text that asks for it.
    pub label: TextKey,
    /// Its type; `None` is a mistake the validation reports.
    pub ty: Option<ParamType>,
    /// Must it be given?
    pub required: bool,
}

impl Param {
    /// A required parameter, not yet typed.
    pub fn new(name: impl Into<String>, label: impl Into<TextKey>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            ty: None,
            required: true,
        }
    }

    /// Of this type.
    #[must_use]
    pub fn of(mut self, ty: ParamType) -> Self {
        self.ty = Some(ty);
        self
    }

    /// May be left out.
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// What a command gives back.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ResultType {
    /// Nothing (the usual command).
    #[default]
    Nothing,
    /// A value of this type (a block of a workflow passes it on).
    Value(ParamType),
}

impl ResultType {
    /// The name in a block description (`nothing` or the type).
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Nothing => "nothing".to_owned(),
            Self::Value(ty) => ty.name(),
        }
    }
}

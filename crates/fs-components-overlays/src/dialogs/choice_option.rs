//! One option of the choice dialog.
//!
//! Pattern: none (a value type with a builder).

/// One option of a [`ChoiceDialog`].
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceOption {
    /// What the caller gets back.
    pub key: String,
    /// What the person reads.
    pub label: String,
    /// A line below the label (why, or what it does).
    pub hint: Option<String>,
    /// Cannot be picked.
    pub disabled: bool,
}

impl ChoiceOption {
    /// An option.
    pub fn new(key: &str, label: &str) -> Self {
        Self {
            key: key.to_owned(),
            label: label.to_owned(),
            hint: None,
            disabled: false,
        }
    }

    /// With a hint line.
    #[must_use]
    pub fn hint(mut self, hint: &str) -> Self {
        self.hint = Some(hint.to_owned());
        self
    }

    /// Cannot be picked.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

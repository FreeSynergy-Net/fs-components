//! Where a component sits and which of its looks the place asks for: [`PlaceInfo`], a part of `DrawCtx`.
//!
//! Pattern: Strategy / Parameter Object – the host hands the choice (view, variant) to the component as one value; the component
//! picks its drawing from it, the host has no special case for any component.

/// The place of a component, as declared in the form description (`view=`, `variant=`).
///
/// `None` means "nothing asked": the component takes the default of its contract. That is a normal case, not an exception.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaceInfo {
    /// The view the place asks for, e.g. `month`, `week`, `day` or `agenda` (read by the host from `view=` of the place).
    pub view: Option<String>,
    /// The variant the place asks for, e.g. `compact` or `color-N` (read by the host from `variant=` of the place).
    pub variant: Option<String>,
}

impl PlaceInfo {
    /// A place that asks for this view; the variant stays the default of the contract.
    #[must_use]
    pub fn with_view(mut self, view: impl Into<String>) -> Self {
        self.view = Some(view.into());
        self
    }

    /// A place that asks for this variant; the view stays the default of the contract.
    #[must_use]
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }
}

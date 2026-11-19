//! `landmark`: the regions of a page a screen reader (or a person with a key) can jump to.
//!
//! Pattern: Factory method over a pure rule table. WAI-ARIA APG, landmark regions: the roles are `banner`, `navigation`, `main`,
//! `complementary` (a side), `contentinfo` (a footer), `search`, `form` and `region`; a `region` and a `form` are landmarks only
//! with a name (`aria-label`); when a role stands more than once on a page, each needs its own name; `main`, `banner` and
//! `contentinfo` stand once. [`Landmark::element`] gives the element or nothing when the rule is broken; [`problems`] checks a whole
//! page's list. Limit: Blinc's role list has no region roles yet (project-blinc/Blinc#79): the attribute is set and the host may read
//! it; a screen reader hears it only when Blinc passes it on.

use blinc_layout::div::{Div, div};

/// A landmark role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Landmark {
    /// The head of the page: a title, a logo, a search (`banner`); once per page.
    Banner,
    /// A group of links to places (`navigation`).
    Navigation,
    /// The main content (`main`); once per page.
    Main,
    /// What belongs to the main content but stands on its own: a side (`complementary`).
    Aside,
    /// The foot of the page (`contentinfo`); once per page.
    Footer,
    /// A way to search (`search`).
    Search,
    /// A form that is worth a jump (`form`); needs a name.
    Form,
    /// Any other part worth a jump (`region`); needs a name.
    Region,
}

impl Landmark {
    /// The ARIA role.
    pub fn role(self) -> &'static str {
        match self {
            Self::Banner => "banner",
            Self::Navigation => "navigation",
            Self::Main => "main",
            Self::Aside => "complementary",
            Self::Footer => "contentinfo",
            Self::Search => "search",
            Self::Form => "form",
            Self::Region => "region",
        }
    }

    /// Whether the role is a landmark only with a name.
    pub fn needs_name(self) -> bool {
        matches!(self, Self::Form | Self::Region)
    }

    /// Whether the page may have it once only.
    pub fn once(self) -> bool {
        matches!(self, Self::Banner | Self::Main | Self::Footer)
    }

    /// The element with its role and name; `None` when the role needs a name and has none (it would not be a landmark).
    pub fn element(self, name: Option<&str>) -> Option<Div> {
        let name = name.map(str::trim).filter(|n| !n.is_empty());
        if self.needs_name() && name.is_none() {
            return None;
        }
        let mut region = div().class("fs-landmark").attr("role", self.role());
        if let Some(name) = name {
            region = region.attr("aria-label", name);
        }
        Some(region)
    }
}

/// What is wrong with the landmarks of a page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// A role that needs a name has none.
    NameMissing(Landmark),
    /// A role that may stand once stands more than once.
    NotOnce(Landmark),
    /// A role stands more than once and two of them have the same name or none.
    NotTold(Landmark),
}

/// Check the landmarks of one page, `(role, name)` each.
pub fn problems(page: &[(Landmark, Option<&str>)]) -> Vec<Problem> {
    let mut found = Vec::new();
    let mut seen: Vec<Landmark> = Vec::new();
    for &(role, _) in page {
        if seen.contains(&role) {
            continue;
        }
        seen.push(role);
        let same: Vec<Option<&str>> = page
            .iter()
            .filter(|(r, _)| *r == role)
            .map(|(_, n)| n.map(str::trim).filter(|n| !n.is_empty()))
            .collect();
        if role.needs_name() && same.iter().any(Option::is_none) {
            found.push(Problem::NameMissing(role));
        }
        if same.len() > 1 {
            if role.once() {
                found.push(Problem::NotOnce(role));
            } else if (0..same.len()).any(|i| same[i].is_none() || same[..i].contains(&same[i])) {
                found.push(Problem::NotTold(role));
            }
        }
    }
    found
}

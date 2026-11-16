//! Tokens that the theme contract names beyond the set Blinc's theme has: `--control-border` (the edge of a box a person ticks),
//! `--code-bg` (the ground of a code block).
//! Plain CSS text over the theme variables, so a token follows the scheme of the theme (light, dark) without a second table.
//!
//! Pattern: none (two constants). The shares are measured: the tests of `fs-components-workbenches` (`wiki_tokens*.rs`) draw the
//! pictures and demand 3:1 for the control edge (WCAG 2.1, 1.4.11) and a visible step for the code ground.

/// The tokens for every style: the edge of an empty box is half text colour over the ground it stands on (3:1 or more in light
/// and in dark), the code ground is a faint wash of the text colour over the surface (the hover ground, `--accent-subtle`,
/// is the same wash of 6 percent, so a code block must not take that one: it would vanish when its row is hovered).
pub const TOKENS: &str = r"
:root {
    --control-border: color-mix(in srgb, var(--text-primary) 55%, var(--surface));
    --code-bg: color-mix(in srgb, var(--text-primary) 4%, var(--surface));
}
.cn-checkbox { border-color: var(--control-border); }
.cn-checkbox:hover:not(:checked) { border-color: var(--text-primary); }
";

/// Craft is dark with a ground of almost the same grey as the panel: the code ground needs more of the text colour to stand off.
pub const CRAFT_TOKENS: &str = r"
:root {
    --code-bg: color-mix(in srgb, var(--text-primary) 11%, var(--surface));
}
";

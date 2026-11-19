# Bundle G6: the rest (carousel, chip; radio group, switch and badge are the existing pieces)

Branch `unreviewed/components-g6-1` (on main 772b867f). APG pages read online on 2026-11-19. Blinc passes few roles on (project-blinc/Blinc#79); roles and keys are set and tested here.

| Block | Status | Pattern | Where |
|---|---|---|---|
| `carousel` (new) | built | APG [carousel](https://www.w3.org/WAI/ARIA/apg/patterns/carousel/): `region` with `aria-roledescription` and a name, slide = `group` with `aria-roledescription="slide"` and the name "n / m", named previous and next buttons (MF2 de/en), Left/Right/Home/End | `widgets/carousel/` |
| `chip` (new) | built | a `list` of `listitem`s with a named remove button; Left/Right/Home/End, Delete/Backspace remove | `widgets/chip/` (adds `Key::Delete`, `Key::Backspace` to `keys.rs`) |
| radio group | NOT rebuilt | APG radio group is `fs-components-fields` `segmented` (`radiogroup`, `radio`, arrows, Home/End) | `fields/controls/segmented.rs`; `tests/g6_existing.rs` |
| `switch` | NOT rebuilt | `fs-components-fields` `switch` (role `switch`, Space/Enter) | `fields/controls/switch.rs`; `tests/g6_existing.rs` |
| `badge` | NOT rebuilt | count and dot stay in `widgets/badge.rs`; `tag` is the plain label | catalog blocks `badge`, `tag` |

Limits (honest): the carousel never rotates by itself (so the APG's pause control and stop-on-focus do not apply); no tab dots; one slide at a time. The chip row has no chip INPUT (typing a new chip belongs to the `keywords` field of the field bundles); the remove button is for the pointer, the keyboard uses Delete or Backspace. Both are ONE tab stop with the keys on the static root, so the keyboard stays when the slide or the chips are drawn again.

Pictures: `tests/g6_pictures.rs` (Admin, light and dark, `bilder/g6-1/`). Contrast: `tests/g6_contrast.rs` reads the pairs from the sheets in all five styles and pins the intended tokens.

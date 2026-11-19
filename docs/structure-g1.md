# Bundle G1: structure and navigation (tabs, toolbar, landmarks, breadcrumb, disclosure, accordion)

Branch `unreviewed/components-g1-1` (on main 772b867f). APG pages read online on 2026-11-19. Accessibility attributes are set and can be read by the host; Blinc passes only 12 roles on to the platform (project-blinc/Blinc#79), so a screen reader hears them when Blinc does. Screen readers are not the aim of this round (Blinc's work); the aim is that roles, states and keys are right and tested.

| Block | Pattern | What the APG says | Where |
|---|---|---|---|
| `tablist` / `tab` | APG [tabs](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) | `tablist` (+ `aria-orientation`), `tab` with `aria-selected` and `aria-controls`, `tabpanel` named by its tab; Left/Right (Up/Down vertical) move and wrap, Home/End, automatic or manual (Enter/Space) choosing; disabled tabs skipped | `widgets/tabs/`, rule in `widgets/roving.rs` |
| `toolbar` | APG [toolbar](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/) | `toolbar` with name and orientation, arrows between controls (no wrap), Home/End, Enter/Space presses, toggles `aria-pressed`, `separator` | `widgets/toolbar/` |
| `landmark` (new) | APG [landmarks](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/) | banner, navigation, main, complementary, contentinfo, search, form, region; region and form only with a name; banner/main/contentinfo once; several of one role told apart by name | `widgets/landmark.rs` (the rule `problems` checks a page's list) |
| `breadcrumb` | APG [breadcrumb](https://www.w3.org/WAI/ARIA/apg/patterns/breadcrumb/) | `navigation` with a name (MF2 de/en), steps are links, the last `aria-current="page"`, separators not read; link colour is `--text-link` (was `--accent`, 3.0 to 4.4:1 in Apple and Windows, found by the contrast test) | `widgets/breadcrumb/` |
| `disclosure` (new) | APG [disclosure](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/) | button with `aria-expanded` and `aria-controls`, Enter/Space toggles | `widgets/disclosure/single.rs` |
| `accordion` (new) | APG [accordion](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/) | header = button in a heading (`aria-level`, a setting, default 3), panel = `region` named by its header; single way: the open header is `aria-disabled` | `widgets/disclosure/accordion.rs` |

Decisions: rows of tabs and toolbar items are ONE tab stop with a cursor mark (`fs-tab-current`, `fs-tool-current`), like `segmented`, because a Blinc widget cannot move the keyboard itself; every header of an accordion is its own tab stop. Enter and Space reach a Blinc window as `Key::Activate` (one key), and as raw keys in other frontends: every widget handles both and a test sends both.

Limits (honest): the arrow keys between accordion headers (optional in the APG) are not done; no closable or movable tabs; the toolbar of the file window and the head bar keep their own element (one tab stop per button, G5 note); no `aria-activedescendant` (the row does not redraw itself, only its items).

Pictures: `tests/g1_pictures.rs` draws the six scenes in the Admin style, light and dark (`bilder/g1-1/`, kept when `FS_COMP_SHOTS` is set). Contrast: `tests/g1_contrast.rs` reads the colour pairs from the sheets and measures them in all five styles, light and dark (4.5:1 for text, 3:1 for edges and rings), and pins the intended token of every rule.

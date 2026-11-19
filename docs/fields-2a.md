# Field bundle 2a: choice and keywords

Branch `unreviewed/components-fields-2` (on main 77db5466). Blinc 3.2 (`fs-blinc-3.2`) carries `select`, `combobox` and `radio` in `blinc_cn` for ONE value (`Signal<String>`) and names only 12 roles (`blinc_layout/src/accessibility.rs`: no listbox, combobox, option, checkbox-group); nothing for chips. So no Blinc adapter: the pieces of this repository are put together (`segmented`, `listbox` in `popover`, `chip`), the missing Blinc roles are a request to Blinc, not a fork.

| Type | Rule (fs-types) | Input | Words |
|---|---|---|---|
| `choice` (`field-choice`) | `Choice::pick` against `FieldDescription.choices`; keys matched exactly after trimming | up to 5 entries segments (`controls/segmented.rs`, radiogroup), more a list (`listbox`) in a `popover`; the block is the same, the count is a state | label `<label>.<key>` through `Say`; an EMPTY list is said in words (`notice.choice.empty`), never a dead control (ADR 0097) |
| `keywords` (`field-keywords`) | `Keywords`: trimmed, NFC, lower case, no duplicates, sorted; 64 characters a word, 256 words | chips (`chip`) and a line: a comma or Enter makes a chip, Backspace on the empty line removes the last, the chip's button or Delete removes one | `problem.keywords.bad-word`, `problem.keywords.too-many` |

Decisions: the trait `BasicField` stays (a value is its parts as text): `choice` has one part, `keywords` one part (the typed list); the list of a choice is handed to the view because the trait knows none (`kinds::check_choice`). Display, compact and cell use the generic views (the key, or the canonical list); the label of an entry in those roles is a follow-up with `Say` in `compact`. Files: one module per type (`kinds/choice.rs`, `kinds/keywords.rs`, `view/choice.rs`, `view/keywords.rs`); the shared files got lines only.

Tests: `tests/choice.rs`, `tests/keywords.rs` (rule, trait, input: roles, keys, typing with the window's router), `tests/f2a_contrast.rs` (the palette colours the views read, all styles light and dark, counter-test), `tests/f2a_pictures.rs` (Admin light and dark). The chip row's own contrast is in `fs-components-widgets/tests/g6_contrast.rs`.

Contrast, why no sheet: `choice` and `keywords` draw their own parts with the colours of the style (`Look`, the palette: caption, hint, problem sentence, words in the field, edge, ring); `tests/f2a_contrast.rs` measures exactly those palette pairs in all five styles, light and dark. The parts that carry a CSS sheet are measured where they live: the segments in the field blocks' contrast tests (`blocks_contrast.rs`), the chip row in `fs-components-widgets/tests/g6_contrast.rs`, the list in `g3_contrast.rs`, the popover in `g2` (token pins included).

Follow-up card (not in this bundle): the display, compact and cell roles of `choice` show the key, not the label `<label>.<key>`, because `compact` knows no `Say`; a `Say` parameter for the three roles is mechanical and belongs to its own change.

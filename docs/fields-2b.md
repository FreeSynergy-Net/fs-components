# Field bundle 2b: reference and checklist

Stacked on 2a on `unreviewed/components-fields-2`. Blinc 3.2 has no roles listbox, combobox, option, checkbox-group and no search-with-candidates (see `fields-2a.md`), so both are built from this repository's pieces.

| Piece | Rule | Input | Words |
|---|---|---|---|
| `reference` (`field-reference`, `Kind::Reference`) | `Reference` of fs-types: an absolute IRI, something after the scheme, kept exactly as written (AS2 `id`), 2048 bytes | a line, the candidates of the program's source ([`ReferenceSource`]) as a `listbox` under it; a click, or Enter on the only candidate, picks; the label of the picked one is shown | no source: `notice.reference.no-source` (ADR 0097); `problem.reference.not-absolute`, `problem.reference.bad` |
| person, profile, group, organisation as target | the catalog's more exact rule: `person-picker` draws them (the field block names no target) | | |
| `checklist` (block `checklist`, no `Kind`: not a basic type) | APG checkbox: `group` with `checkbox` items and `aria-checked` | one tab stop; Up/Down/Home/End, Space/Enter ticks, Delete/Backspace removes, Alt+Up/Alt+Down put an item elsewhere; a line adds on Enter; each row has a remove button for the pointer | MF2 de/en (`checklist-add`, `checklist-remove`) |

Decisions: `reference` is one part (the IRI) like `choice`; the source reaches the view, not the trait. The checklist lives in `fs-components-widgets` (it owns no type): the field blocks stay one per basic type. A removed item's key is never given again (`State::add`). Limits: moving into the candidates with the arrow keys is not done (the list is its own tab stop); an item is one line; no drag to reorder; the display/compact/cell of a reference show the IRI, not the label of the target (follow-up with the source).

Contrast: the checklist has a sheet and is measured from it (`tests/f2b_contrast.rs`, all styles, light and dark, token pins, counter-test); the reference input uses the palette like 2a (`f2a_contrast.rs` covers its pairs); its candidate list is the `listbox` measured in G3.

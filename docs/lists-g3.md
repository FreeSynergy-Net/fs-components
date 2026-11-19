# Bundle G3: lists and data (listbox, tree, grid, treegrid, table, feed)

Branch `unreviewed/components-g3-2` (on main 1497c246 (after G1)). APG pages read online on 2026-11-19. Blinc does not pass these roles on to the platform yet (project-blinc/Blinc#79); roles, numbers and keys are set and tested here.

| Block | Pattern | What the APG says | Where |
|---|---|---|---|
| `listbox` (new) | APG [listbox](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/) | `listbox` (+ `aria-multiselectable`), `option` with `aria-selected`; arrows, Home, End; single: selection follows focus; several: Space toggles, Ctrl+A picks all | `widgets/listbox/` |
| `tree` | APG [tree view](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/) | `tree`, `treeitem` with `aria-level/-setsize/-posinset`, `aria-expanded` only on parents, children in a `group`; Down/Up, Right (open, then first child), Left (close, then parent), Home/End, Enter/Space | `widgets/tree/` (the older `tree_view` stays: an adapter over Blinc's tree without roles) |
| `grid` | APG [data grid](https://www.w3.org/WAI/ARIA/apg/patterns/grid/) | `grid`, `aria-rowcount/-colcount` (the head counts), `row` + `aria-rowindex`, `gridcell`/`columnheader` + `aria-colindex`; arrows, Home/End, Ctrl+Home/End, Page Up/Down, Enter/Space | `widgets/grid/` |
| `treegrid` (new) | APG [treegrid](https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/) | a grid whose parent rows have `aria-expanded`, `aria-level`, `aria-setsize`, `aria-posinset`; Right/Left on the first cell open and close | `widgets/grid/` (the same module, `Grid::treegrid`) |
| `table` | APG [table](https://www.w3.org/WAI/ARIA/apg/patterns/table/) | `table` named by its caption, `row`, `columnheader`, `cell`, `rowheader`, `rowgroup`; counts only for a partial table; no keys | `widgets/table/` |
| `feed` (new) | APG [feed](https://www.w3.org/WAI/ARIA/apg/patterns/feed/) | `feed` with `aria-busy`, `article` named by its title with `aria-posinset` and `aria-setsize` (`-1` while the total is not known); Page Down/Up walk the articles | `widgets/feed/` |

Decisions: every widget of the bundle is ONE tab stop with a cursor mark (`fs-option-current`, `fs-treeitem-current`, `fs-grid-cell-current`, `fs-article-current`), like `segmented`, because a Blinc widget cannot move the keyboard itself; the widget's own element is static and only its items are drawn again, so the keyboard stays. Enter and Space reach a Blinc window as `Key::Activate` (one key) and as raw keys in other frontends: every widget handles both and a test sends both. Tests also check the attributes after more than one redraw.

The existing big list (`list_view`, virtual rows, 100 000 rows) and the chat list (`message_list`) keep their own code; the new grid and feed are for lists that fit on a page (a follow-up card: give `list_view` the grid roles).

Limits (honest): no type-ahead in listbox and tree, no Shift range, no `*` in the tree; the head cells of the grid are not on the cursor's way; grid cells are text (a cell that holds controls needs a tab stop of its own); no sorting, no editing, no virtual scrolling; the feed leaves Ctrl+Home and Ctrl+End (leave the feed) to the host.

Pictures: `tests/g3_pictures.rs` draws the six scenes in the Admin style, light and dark (`bilder/g3-1/`). Contrast: `tests/g3_contrast.rs` reads the colour pairs from the sheets, measures them in all five styles, light and dark (4.5:1 text, 3:1 edges and rings), and pins the intended token of every rule.

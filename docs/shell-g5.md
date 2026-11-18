# Bundle G5: shell and HIG (header bar, sidebar, context menu, card, avatar, drop zone)

Branch `unreviewed/components-g5-2` (on `unreviewed/components-g2-1` a7032b3c). APG pages read online on 2026-11-18.

| Block | Pattern | What the APG / HIG says | Where |
|---|---|---|---|
| `head-bar` | APG [toolbar](https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/); GNOME HIG: Header bar | role `toolbar` with `aria-label` (the title); the title is a heading; APG wants ONE tab stop with Left/Right between the controls | `widgets/head_bar/bar.rs` (roles done; arrows NOT done: needs the host to move the keyboard, follow-up) |
| `sidebar` | APG [landmarks](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/) (navigation); GNOME HIG: Sidebar | the places are a `navigation` landmark with a name | `workbenches/file_manager/view/sidebar.rs` |
| `context-menu` (new block) | APG [menu](https://www.w3.org/WAI/ARIA/apg/patterns/menu/); GNOME HIG: Context menu | role `menu`, lines `menuitem`, Escape closes, arrows move, opened from a command (Menu key, Shift+F10) | `app/src/context_menu.rs` already did it (shares `menu_view`); block and role test added |
| `card` / `field-card` | none in the APG; GNOME HIG: Card | a group named by its label (else its value); actions are buttons | `widgets/field_card/card.rs` |
| `avatar-group` | GNOME HIG: Avatar | every face has an alt (the name, `person-face` already); the row is a group naming its people and `+N` | `widgets/avatar_group/` |
| `drop-zone` (new) | GNOME HIG: Drag and drop | a drag needs a way that does not drag: the zone is a button, Enter/Space/click calls `on_pick` (file chooser); while files hover it says so in words | `widgets/drop_zone/` |

Limits (honest): the header bar has one tab stop per button, not one for the bar (APG toolbar); Blinc's drop event carries no paths,
so `on_drop` only says that files were dropped; the deprecated `aria-grabbed` of the board card (`workbenches/board/view.rs:248`) is left
(ARIA 1.1 removed it; a replacement needs words: follow-up card); no picture of the new zone.

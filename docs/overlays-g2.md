# Bundle G2: messages and overlays (alertdialog, tooltip, popover, toast, banner, status line)

Branch `unreviewed/components-g2-1`. Each pattern was read online at https://www.w3.org/WAI/ARIA/apg/patterns/ on 2026-11-18.

| Block (contract id) | Pattern | What the APG / HIG says | Where |
|---|---|---|---|
| `alertdialog` | APG [alertdialog](https://www.w3.org/WAI/ARIA/apg/patterns/alertdialog/) | role `alertdialog` on the element with message and buttons; `aria-modal`, `aria-labelledby` (title), `aria-describedby` (message); keys as the modal dialog: Escape closes, Tab cycles; the keyboard starts on the least destructive button | `widgets/dialogs/alert_dialog.rs` |
| `tooltip` | APG [tooltip](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/) (W3C marks it work in progress) | role `tooltip`, trigger has `aria-describedby`, Escape dismisses, the keyboard stays on the trigger, focus tooltip closes on blur, hover tooltip after a delay and stays while the pointer is over trigger or bubble | `widgets/tooltip/` |
| `popover` | no APG pattern; follows APG [dialog (modal)](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/) non-modal; GNOME HIG: Popover | role `dialog` with a name, trigger `aria-haspopup`/`aria-expanded`/`aria-controls`, Escape closes, the keyboard returns to the trigger | `widgets/popover/` |
| `toast` (extended) | APG [alert](https://www.w3.org/WAI/ARIA/apg/patterns/alert/); GNOME HIG: Toast | role `alert` (warning, error) or `status` (info, ok) with `aria-live`/`aria-atomic`; an alert takes no keyboard focus; APG advises against vanishing alerts (WCAG 2.2.3), so the time is a setting (`Times`, `Times::STAY`), an error stays by default; a pointer or the keyboard on a toast pauses it | `widgets/toast/` |
| `banner` | APG alert (role by step); GNOME HIG: Banner | not the landmark `banner`; role `status`/`alert` by step; the action is a button, the dismiss mark a tab stop | `widgets/banner/` |
| `status` (extended, statusbar) | APG alert/status; GNOME HIG: Statusbar | the existing status line of `fs-components-app` gets role `status` (note) or `alert` (problem) and `aria-live`; no second widget | `app/src/view.rs` |

Limits (honest): tooltip and popover sit in the flow of their host, not in an overlay (no flip at the window edge); no picture was made
of any of them (pictures: not covered here); a widget cannot move the keyboard, so `alertdialog` and `popover` ask the host through
`FocusTo` (`widgets/focus.rs`); the gap on Blinc's side is in `BLINC-ANFRAGEN-SAMMLUNG.md`.

Decisions (after the first review round): (1) Tooltip and popover stay in the flow of their host on purpose in this bundle: an overlay with a flip at the window edge needs the window's overlay stack (see `dropdown` and the status panel of `fs-components-app`) and is a follow-up card. (2) A toast is held by the pointer AND by the keyboard as two separate holders (`Toasts::hold_at`, `Toasts::hold_keyboard_at`); its clock runs only when both have let go (test `pointer_and_keyboard_hold_together...`). (3) Pictures: `tests/g2_pictures.rs` draws each block offscreen and keeps the PNGs when `FS_COMP_SHOTS` names a directory.

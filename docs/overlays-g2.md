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

Limits (honest): tooltip and popover sit in the flow of their host, not in an overlay (no flip at the window edge); pictures of every
block in one FreeSynergy style (Admin), light and dark, come from `tests/g2_pictures.rs` (kept when `FS_COMP_SHOTS` names a directory);
a widget cannot move the keyboard, so `alertdialog` and `popover` ask the host through `FocusTo` (`widgets/focus.rs`). The popover does
not move the keyboard into its panel when it opens (a non-modal panel: the person goes on with Tab); it gives it back to the trigger on
closing. Role and `aria-live` attributes are SET and readable by the host; a screen reader hears them only when Blinc passes them on:
Blinc maps `role` through 12 roles and loses dialog, alertdialog, status, alert and `aria-live` (upstream issue
[project-blinc/Blinc#79](https://github.com/project-blinc/Blinc/issues/79)).

Rules decided in the review round: (1) `aria-describedby` names the body only for a short message (`alertdialog`, `confirm-dialog`),
never for the list of `choice-dialog` or the long text of `license-dialog` (APG dialog-modal). (2) One rule for the live attribute:
`Severity::live` (`assertive` for role `alert`, `polite` for role `status`) is set by toast, banner and the status line of the app alike.
(3) Toast times: info and ok go by themselves (5 s, 4 s); warning and error have role `alert` and STAY by default (APG alert: avoid
alerts that disappear by themselves, WCAG 2.2.3); every time is a setting (`Times`, `Times::STAY`). (4) The contract state `open` of
tooltip and popover is the class `fs-tooltip-open` (on the bubble) and `fs-popover-open` (on the host); the banner part `action` is
`fs-banner-action`. (5) The tooltip bubble is the inverse of the surface (`--text-primary` ground, `--text-inverse` text, small type, shadow)
so it does not look like a second button.

Decisions (after the first review round): (1) Tooltip and popover stay in the flow of their host on purpose in this bundle: an overlay with a flip at the window edge needs the window's overlay stack (see `dropdown` and the status panel of `fs-components-app`) and is a follow-up card. (2) A toast is held by the pointer AND by the keyboard as two separate holders (`Toasts::hold_at`, `Toasts::hold_keyboard_at`); its clock runs only when both have let go (test `pointer_and_keyboard_hold_together...`). (3) Pictures: `tests/g2_pictures.rs` draws each block offscreen and keeps the PNGs when `FS_COMP_SHOTS` names a directory.

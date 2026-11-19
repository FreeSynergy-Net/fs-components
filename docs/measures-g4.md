# Bundle G4: measures and display (slider, range, meter, progress, splitter)

Branch `unreviewed/components-g4-1` (on main 772b867f). APG pages read online on 2026-11-19. Blinc does not pass these roles on to the platform yet (project-blinc/Blinc#79); the roles and values are set and tested here.

| Block | Pattern | What the APG / ARIA says | Where |
|---|---|---|---|
| `slider` | APG [slider](https://www.w3.org/WAI/ARIA/apg/patterns/slider/) | role `slider` with `aria-valuenow/-min/-max`, `aria-valuetext` (the caller's words), `aria-orientation`, a name; arrows by a step, Page Up/Down by a page, Home/End; the pointer sets and drags | `widgets/slider/` (`Scale` is the pure rule) |
| `range` (new) | APG [slider (multi-thumb)](https://www.w3.org/WAI/ARIA/apg/patterns/slider-multithumb/) | each thumb a `slider` with its own name; the low thumb's maximum is the high value and the other way round; each thumb a tab stop | `widgets/range/` |
| `meter` (new) | ARIA 1.2 role `meter` (no APG pattern) | `aria-valuenow/-min/-max`, value text with the zone in words (MF2 de/en), zones after HTML's `meter` (`low`, `high`, `optimum`); read, not set | `widgets/meter/` (`zone` is the pure rule) |
| `progressbar` | APG/ARIA `progressbar` | value attributes while the end is known, none while it is not (`aria-busy`); read, not set | `widgets/progress/` |
| `splitter` (new) | APG [window splitter](https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/) | focusable `separator` with `aria-valuenow` (size of the first pane in percent), `-min`, `-max`, `aria-controls`, `aria-orientation` (`vertical` between left and right panes); arrows, Home/End, Enter collapses and restores; the pointer drags | `widgets/splitter/` (`State` is the pure rule) |

How a live area is drawn again in Blinc decided the structure of slider, range and splitter (found by test, not by reading): an area whose structure changed loses the keyboard; an area whose sizes changed keeps the old attributes (`aria-valuenow` goes stale); only an area that changes nothing but attributes and classes keeps both. So the keyboard sits on an invisible layer over the track (`fs-slider-keys`, role `slider`, never moves, the pointer goes through it), what moves is drawn in a second area, and the track around both stays and carries the pointer, so a drag is not cut off by the redraw it causes. Tests press keys WITH a frame between them, as a window does.

Colour: the contrast test found that the border colour as a track left the accent fill at 1.0:1 in Windows dark; the track is the accent tint now (the fill is 3.1:1 or better in all five styles, light and dark). The zone colours of the meter are not 3:1 against any ground in every style (green on the Apple tint is 2.0:1), so the length of the fill is carried by an edge in the text colour and the zone by a word.

Limits (honest): horizontal sliders only (vertical is a follow-up card); the range thumbs may meet; the splitter has two panes; in a Blinc window Enter and Space come as one key, so Space collapses the splitter too; no tick marks; the progress bar has no animation for an unknown end (a moving bar would cost frames for ever).

Pictures: `tests/g4_pictures.rs` draws the five scenes in the Admin style, light and dark (`bilder/g4-1/`). Contrast: `tests/g4_contrast.rs` reads the pairs from the sheets, measures them in all five styles, light and dark, and pins the intended token of every rule.

# fs-components-calendar-grid - Design

Month grid, week grid and day column (time axis) of a calendar, as one component (ADR 0066: the program gives data, a trait and words; this crate draws). Entry: `CalendarGrid::new(state, source, say, look).events(..).build(width, height)`.

## Data boundary (stable)

| Type | Role |
|---|---|
| `GridItem`, `Tone`, `Day`, `Moment` | plain data, already localised (time zone is the caller's job); no JSCalendar type. All-day items cover `start.day..=end.day`; timed items end exclusive. |
| trait `GridSource` | where items of a range come from (`VecSource` for a fixed list); asked on every build. |
| trait `GridEvents` | `select_slot`, `select_item`, `navigated`; all default to nothing (`NoEvents`). |
| `Nav`, `Mode`, `Slot` | navigation message, month/week/day, a chosen day (+ minute on a time axis). |

Words are ids of the catalog `texts/{en,de}.mf2` (`calendar-view-month|week|day|agenda`, `calendar-today|previous|next|week-short|all-day|empty|more`), answered by the chain host (`ctx.say`), then the catalog, then the key (`texts.rs`). Names of weekdays, months and days are never ours: they come from `DrawCtx.dates` (CLDR over ICU4X, `names.rs`); the week starts with `dates.first_weekday()` unless the place says `view.week-start`. Looks come only from `fs-components-styles` through `Look` (Admin/Fiori, Simple, Apple, Windows all drawn in a test, no extra code).

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `calendar_core/day.rs`, `model.rs` | Value object | dates and items compare by value |
| `calendar_core/*` | Adapter + Strategy | `read_bars` (rows by channel in, entries out, the one reader of `bars-in-time` for grid and timeline), `color_slot`, `partial_notice`, `today_utc`; moved here from the dropped `calendar-view` (ADR 0005, 0050) |
| `source.rs` | Strategy + Null object | the host owns data and decides what a choice means |
| `state.rs` | State | one set of transitions for buttons, clicks and keys |
| `layout.rs`, `axis.rs`, `spans.rs` | none (pure functions) | days of a range, side-by-side lanes, bars over several days, minutes to pixels; tested without a window |
| `handle.rs` | Facade + Observer | one clonable handle; a change updates the state, tells the host, redraws the live area only |
| `view/*` | View function / Adapter | state and measures in, Blinc elements out |
| `view/keys.rs` | Adapter | Blinc key events to `Key` (copy of a private helper of `fs-components-widgets`; move to a shared crate when one exists) |

## Behaviour

- Month: six rows, bars across the days an item covers (cut at week ends), "+N" when the cell is full. Week/day: header, all-day row (all-day and multi-day items), scrolling axis of 24 hours, overlapping items side by side, now line, marked slot, today mark (accent disc).
- Selection: one thing is selected (slot or item). Click on cell/column/bar, arrow keys (month: day, week; timed: day, slot), PageUp/PageDown navigate, Home is today, Enter/Space choose the slot.
- Variant `compact` (place `variant=compact`, month only): the small calendar of a side bar. Strategy `CellPainter` (`view/painter.rs`): `Bars` draws the full month, `Marks` a number and a mark (part `mark`, accent dot) under every day with items; no toolbar, no bar titles; today is the accent disc, the selected day a tinted cell. With `view.week-numbers` the column is narrower and has no head text. Week and day ignore the variant.
- Updates: the toolbar and body live in one `Live` region; a change rebuilds that region only, never per frame.

## Tests

`tests/model.rs`: civil dates, ranges, spans, lanes, minutes/pixels, navigation, keys, source. `tests/view.rs`: drawn offscreen in all four styles, driven by clicks and keys through Blinc's event router.

## Unsupported (named, not built)

- Drag to create/move/resize an item (no gesture API used; resize/long-press avoided on purpose).
- Year view. (The agenda is a mode of the grid: the next 30 days as a list by day, days without items left out.) (Week numbers: month only, ISO 8601, switch `view.week-numbers`; not in the week and day views. `compact` has no small form of the week and the day: they draw as the full grid. The small month has no toolbar and no way to pick a month except PageUp/PageDown and `CalendarGrid::navigate`; a slim header is the host's.)
- Initial scroll position of the time axis (starts at 00:00; the Blinc scroll offset is not set).
- Locale formats: hour labels are `HH:00` and the year/"+N" are plain numbers; month and weekday names come from `Say`.
- Days of 23 or 25 hours (daylight saving): the axis is always 24 hours of 60 minutes, and an item's minutes are the caller's wall-clock minutes; the host must cut or place such days itself.
- Recurrence, time zones, working-hours shading: the host expands and localises.
- Per-style icon shapes, glass/backdrop effects (only colours, radii, spacing, fonts of the style).
- Buttons are `blinc_cn` buttons; their height is cn's, not the style's.
- Overlap lanes use 30-minute minimum height for lane choice only; a very long title is clipped, not wrapped.

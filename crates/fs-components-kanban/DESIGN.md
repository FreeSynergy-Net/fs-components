# fs-components-kanban - Design

ONE board element (ADR 0066: the program gives data, ports and words; this crate draws). Entry of the board `views()` draws: `View` (`form/view.rs`, over `BoardBench`; feature `form`).

**Pattern: Ports and Adapters (Hexagonal).** The element ([`BoardBench`]) is the core and owns its ports (`BoardSource`, `BoardEvents`, `DetailContent`). It takes data (columns with cards) and gives requests back (`MoveRequest`, `NewCard`); every source is an adapter on those ports. The forms adapter (`FormSource`, `FormEvents`, `View`; directory `src/form/`, feature `form`, on by default) is the only part with channel names; without the feature the crate is the pure element and does not depend on `fs-components-core` (CI: `cargo check -p fs-components-kanban --no-default-features`). `fs-components` feature `views` turns it on. Inside: ONE state machine (enum based, over the shared `Sorter`) + Command (`MoveRequest`); deviation from GoF State: enums, no state objects.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `model.rs` | Value object | `Card`, `Column`, `Due`, `Operation`: plain data, texts already in the reader's language |
| `requests.rs` | Value object + Command | `MoveRequest`, `NewCard`, `Answer`, `Reason`, `Question`, `ColumnAction`, `ContextTarget`, `FilterState` |
| `source.rs` | Port (Strategy) + Null object | `BoardSource` (what is shown, what is supported), `BoardEvents` (what the person wants), `NoEvents` |
| `state.rs`, `state_*.rs` | State machine (enum) + Command | `BoardState`: cursor, held card, notice, undo; EVERY way (mouse, key, click, menu command, undo) ends in `place` and yields the same `MoveRequest` |
| `drag.rs`, `hit.rs`, `scroll.rs` | State machine (enum) / pure geometry | the pointer gesture with the 4 px rule; pointer to column and place; auto-scroll |
| `commands.rs`, `bench_menu.rs` | Command | menu commands `board.*` |
| `bench.rs`, `bench_*.rs` | Facade + Observer | `BoardBench`: one clonable handle; change state, tell the host, redraw the live area only |
| `view*.rs`, `card_line.rs` | View function | state in, Blinc elements out; the facts are drawn by the field components (`FieldSlot::compact_view`) |
| `wip.rs`, `quick.rs`, `meta.rs` | none (pure functions) | WIP mark (counts hidden cards, never locks), quick entry (several lines ask first), card text |
| `detail.rs` | Template method + Strategy | the frame at the side; what stands in it is the program's (`DetailContent`) |
| `labels.rs`, `styles.rs`, `texts/` | Factory method | the element brings its own words (`MessageFormat` 2) and its style sheet |
| `memory.rs` | Test double | `MemoryBoard` |
| `form/source.rs`, `form/events.rs`, `form/read.rs`, `form/view.rs`, `form/dialogs.rs`, `form/ask.rs` | Adapter (Ports and Adapters) + Abstract Factory | `FormInput`/`FormWrite` to the ports; channel names live only here |

## Data boundary

`Column` (key, title, `limit` = WIP, cards), `Card` (key, title, subtitle, due, progress, assignee, facts), `Operation` (`AddColumn`, `Rename`, `Limit`, `Move`, `AddCard`). Texts are plain data in the reader's language. `BoardSource::supports(op)` is read on every build: a control of an unsupported operation is drawn disabled in its normal size, the reason in a tooltip (`disabled::wrap`, ADR 0096 H-K5), and unlocks by itself. Card facts: `Card::fact` / `fact_or_more` (limit `DEFAULT_FACTS`, "+ N more" says what did not fit); frame elements: `DEFAULT_ELEMENTS`.

## Moves

One state machine for every way: mouse (press, 4 px, carry, release; auto-scroll), keys with modifiers (Shift+arrows reorder, Home/End, Ctrl+Z, N), click-to-move (a held card, then a click on a column), the move buttons in the frame at the side and the menu commands `board.to:<column>`. A finished move is a `MoveRequest`; the program answers `Accepted`, `Refused(reason)` or `Ask(question)`. The right to move is asked before every gesture and checked in `place`; a card carried when the right goes is given back, nothing is sent.

## Known limits (named, not hidden)

- Fixed pixel sizes (`COLUMN_W`, `CARD_H`, `HEAD_H`, `FOOT_H`, `DETAIL_W`): the hit test needs exact geometry; the columns stand in one row and the wrapping belongs INTO the element (flow bundle). New code assumes no number of columns.
- `NewCard.top` is not sent (channel `create` has no `top`); `Collapse` has no effect; no lanes yet (`Column.lane` is the interim way for `create`).
- `state.load` and `place` call `BoardSource::columns` / `can_drop` with the state lock held: a host that reads the board from inside these calls hangs (`locked`, `generation` and `supports(Move)` are asked there too; only the gestures ask `supports` without the lock). Fix wanted: read the source before taking the lock.

## Blinc notes

Clicks bubble to ancestors and there is no way to stop them. So a click target that has buttons next to it is a transparent layer after the content with the buttons as its siblings, and controls inside a column tell the column through a mark that the click is theirs.

## Tests

`tests/board_*.rs` drive the real element through Blinc's tree and router (`board_common`): moves, keys, click, pointer, reasons, withdrawn rights, limits, facts, progress, locked controls, pictures. `form_events.rs`, `board_form.rs`, `board_mount.rs`: the forms adapter. `lanes.rs`, `limits.rs`: reading of lanes and the facts setting. `board_host/` is the host double the replay and app tests use.

# Theme contract of fs-components

**Version 0.2 (draft; 0.1 was derived from fs-components v0.0.12, 0.2 adds the blocks of 0.0.23).** ADR 0074, step 1. German version: [THEME-CONTRACT.de.md](THEME-CONTRACT.de.md).

The contract is the list of **blocks** (the pieces a window is made of), each with fixed **parts** and **states** (and, where useful, **variants**). A style (theme) styles only these names, so it works on every program, also on future ones. Programs and frontends contain no colours, spacings or fonts (ADR 0074, section 2). This is the first draft: it was **derived from what the code does today**, not invented; the reviewers decide what stays.

## Files

| File | What |
|---|---|
| `contract.toml` | Version, where the other files are, the name prefixes that count as class names, the standard states. |
| `blocks/*.toml` | The blocks: name, where the name comes from, scope, parts, states, variants, one-line summary in EN and DE. |
| `mapping/*.toml` | Every class or id of today (fs-components v0.0.12, the Store, and the two in-flight branches) and the contract name it becomes. |
| `exceptions.toml` | Names in the source that are no class (thread names, state keys, handle ids), each with its reason. |
| `BLOCKS.md`, `BLOCKS.de.md`, `MAPPING.md` | Generated tables (`contract/render`). Do not edit by hand. |
| `HARDCODED-VALUES.md` | Generated list of fixed values in the code (`contract/hardcoded`), the basis of step 2. |
| `check`, `selftest`, `render`, `hardcoded` | Python 3 scripts without dependencies and without network (`lib/contractlib/` holds the code). |

One format: TOML. The files are split only to stay short; `contract.toml` lists them.

## Rules

1. **Block names.** A block is named after the WAI-ARIA 1.2 role where one fits (`button`, `menu`, `tab`, `tree`, `grid`, `dialog`, `alert` ...). Where Open UI has a common component name, after Open UI (`card`, `tag`, `badge`, `avatar`, `breadcrumb`, `text`). Otherwise an own name **with a written reason** (`naming = "own"`, `reason`). For `aria` and `open-ui` the block name is equal to the role or component name; `check` verifies both against the lists in `lib/contractlib/vocab.py`.
2. **Names are only `[a-z0-9-]`**, no capitals, no underscores, no duplicates (blocks, parts, states, variants).
3. **Every block has parts and states** and a summary in both languages. A block that has nothing to report in a state says `default` (stated, not silent).
4. **Parts** are the named places inside a block (`program-card`: `icon`, `title`, `summary`, `badge`, `action`). **States** are situations of the block or a part (`installed`, `update`, `busy`, ARIA ones such as `selected`, `expanded`, `disabled`). **Variants** are looks that the program chooses and that do not change (a button is `primary` or `outline`; a tag has a tone). A state changes while the program runs.
5. **Scope** says who owns the block: `general` (every program), `block` (one widget or crate of fs-components), `program` (a rich block of one program: program card, task row, page tree row ...). Rich blocks are wanted: a style can only treat a Store card, a board column or a task row in its own way if each has its own name.
6. **Target syntax** in the mapping: `block[.part][/variant][:state]`, for example `button/primary:disabled`, `program-card:installed`. A target `-` is allowed only with status `remove`.
7. How the contract names appear in CSS or Blinc selectors is **not decided here** (step 3 with the styles). The mapping says which old name becomes which contract name, nothing more.

## Status of a mapped name

| Status | Meaning |
|---|---|
| `mapped` | The name has its place; only the name changes when step 4 renames it. |
| `provisional` | The place exists, but the program or workbench is still provisional (README: all workbenches except the reader), or the target is a best guess. Needs a decision. |
| `remove` | No contract name. The name is arrangement (a spacer, a width) or one look of one style (all `craft-*` classes): tokens or the frontend take it over. |

Names in `mapping/other-places.toml` come from the Store (`external`) and the branches `tasks-visual` (`fs-plan-*`) and `wiki-frontends-1` (`fs-wk-*`, read only) (`coming`). The check does not look for them in this repository.

## State at version 0.1

Run `contract/check --stats` for the current numbers. At the time of writing: 67 blocks (34 general, 21 block, 12 program). Of the 474 literal `.class("..")`/`.id("..")` names of fs-components (production and tests): 261 mapped, 127 provisional, 27 remove, 58 only in tests (outside the contract), 1 exception (`tabindex=0`, which is a bug, not a class: `.class("tabindex=0")` in `file_manager/view/actions.rs`). Besides these, the source uses 82 more names that no `.class(..)` call spells out (CSS selectors, constants, match arms); 13 of those are no classes (thread names, keys, handle ids, a crate name, a cargo feature) and are on the exception list, the rest are mapped. Nothing is left unmapped.

## What the scripts prove

- `contract/check`: every class or id in `crates/*/src` (`.class("..")`, `.id("..")`, every word of a multi-class string, and every name with the prefix `fs-`, `craft-` or `cn-` in a line of code, which catches CSS strings, constants and match arms) is in the mapping or on the exception list; no mapping or exception is stale; every block has parts, states, summary (EN, DE), a valid scope and a justified name; names are `[a-z0-9-]`; mapping targets point at real blocks, parts, states and variants; no duplicates anywhere; the generated tables are current.
- `contract/selftest`: breaks a copy of the repository in 25 ways (a new class, a removed mapping line, a block without parts, a capital letter, a duplicate, a stale line, a hand-edited table ...) and demands that `check` fails with the right message each time. It also proves that `hardcoded` notices a new fixed value.
- `contract/hardcoded --check`: `HARDCODED-VALUES.md` is current. It is not part of `check` on purpose, because every change of a pixel value in code would break it.

Limits, so nobody trusts the check too much: the scan is textual. A class built at run time (`format!`, a variable) is not seen; ids built with `format!` (`fs-app-menu-{index}`, `{prefix}-card-{id}`, `cal-item-{key}`) are element identities, not style hooks, and are outside the contract. A class name without `fs-`, `craft-` or `cn-` is found only inside `.class("..")`/`.id("..")`. A name used only in a test file is not checked (`mapping/test-only.toml` lists the 58 that exist). The check does not look at the Store or the branches. A mapping can be wrong in meaning even if it passes: the review must read it.

## Not adopted from the reports

The numbers marked `[nicht belegt]` in the reports are not used. The ARIA role list in `vocab.py` is the 1.2 list including `code`, 82 names; the report counted 81 and its table lacks `code`, so one of the two is wrong and the review should check it against the W3C text. WAI-ARIA 1.3 was not looked at. The Open UI list is the 33 research components of open-ui.org as the report names them; its component matrix was not readable and is not used.

## Tokens beyond Blinc's set (0.0.21)

Blinc's theme has a fixed set of colour tokens. A look that the set cannot say gets a token of our own, defined once in `fs-components-styles` (`styles/control_tokens.rs`) as a `:root` variable over the theme's tokens, so it follows the light and the dark scheme of every style. A style may override it (Craft does).

| Token | Role | Default | Craft | Measure (test) |
|---|---|---|---|---|
| `--control-border` | the edge of a box that a person ticks (`.cn-checkbox`, empty) | 55 % `--text-primary` over `--surface` | same | 3:1 against the page (WCAG 2.1, 1.4.11; `wiki_tokens.rs`, `wiki_ground_craft.rs`) |
| `--code-bg` | the ground of a code block (`.fs-tv-code` in the wiki widgets, `.fs-re-code` on the writing surface) | 5 % `--text-primary` over `--surface` | 12 % | not the hover ground `--accent-subtle` (a code block must not vanish when its row is hovered); Craft 1.3:1 against the page |

Tone of an icon (`Tone`) is not a token: an icon that stands on the selection takes `Tone::Normal`, and a small symbol is drawn with a line of at least 1.5 device pixels (`icon_stroked`), because the style's own line (0.7 px at 14 px) never reaches 3:1.

## Version 0.2: dialogs, toast, switch, segmented, skeleton, partial notice (0.0.23)

Version 0.2 adds blocks, parts, states and variants; no name of 0.1 is removed. Per the fixed step `alle-stile-erweitern`, every style gives an explicit entry for every new block.

| Block | Where | New in 0.2 |
|---|---|---|
| `confirm-dialog`, `choice-dialog`, `license-dialog` | `fs-components-widgets/src/dialogs/` | new blocks; states `danger`, `busy` (confirm), `selected`, `disabled` (choice option) |
| `dialog` | same | new state `danger` (the frame all three share) |
| `toast` | `src/toast/` | new block; variants `info`, `ok`, `warning`, `error`; parts `stack`, `more`, `root`, `icon`, `text`, `action`, `close` |
| `alert` | `src/states/` | new part `title`, new variant `ok` (the failed surface is `alert/error` with the retry action) |
| `partial-notice`, `skeleton`, `empty-state` | `src/states/` | `partial-notice` and `skeleton` new (variants `line`, `block`, `circle`; state `busy`); `empty-state` now has code |
| `switch`, `segmented` | `fs-components-fields/src/controls/` | new blocks; `switch` states `checked`, `mixed`, `disabled` |

**Tokens of the blocks (one entry per style, written out in `fs-components-styles/src/styles/block_tokens.rs`).** A new style does not compile until it has its entry (`look` is an exhaustive `match` over `StyleId`). The tokens are `:root` variables over the theme's own tokens, so colours follow the light and the dark scheme; the numbers are the style's: `--scrim`, `--dialog-width`, `--dialog-shadow` (from the style's overlay shadow, light), `--dialog-radius`, `--dialog-pad`, `--message-radius`, `--message-pad`, `--message-tint` (share of the colour of the step in the ground of a toast, alert or partial notice), `--toast-width`, `--toast-edge` (0 = no coloured edge: glyph and tint tell the step), `--skeleton-base`, `--skeleton-radius`, `--skeleton-height`, `--switch-width`, `--switch-height`, `--segment-height`. `BLOCK_TOKENS` says which block reads which token.

**What proves it.** `crates/fs-components-styles/tests/block_tokens.rs` (contract version, blocks of 0.2 against `BLOCK_TOKENS`, every token for every style and appearance, no two styles alike); `crates/fs-components-fields/tests/blocks_styles.rs` (every block drawn in every style, light and dark, with the theme guard; pairs of pictures must differ, an unchanged picture is red; no variable of the new sheets without a definition); `blocks_contrast.rs` (WCAG ratios measured on the pictures). The workflow scripts `stil-vollstaendigkeit` and `zwei-stile` of `frontend-umbau` do not exist yet; these tests are their check for the blocks of 0.2.

## Open points (for the review)

- Is `provisional` the right word for a workbench role that maps cleanly to a general block (for example `fs-tasks-failed` to `alert/error`)? It is provisional only because the workbench is.
- Several blocks of the Store (`program-row`, `program-detail`) are reconstructed from CSS names of the Store, not from a design. Same for `page-tree-row`, `page-block`, `quick-add`, `task-row`, `board-column`, which come from branches that are not released.
- Blocks not in the code yet but expected by ADR 0074 (tooltip, switch, select, accordion, toast, slider ...) are not listed: the contract was derived from what exists. Open UI lists them; they come when a program needs them.
- Part and state lists contain names no old class reaches yet (349 of 433 parts are reached). They are the gaps a style author needs filled; some may be dropped.
- The mapping of `craft-*` to `remove` assumes that the Craft style becomes a theme per ADR 0074, step 3 and that its classes disappear.
- Colours, sizes and fonts: `HARDCODED-VALUES.md`, part B, is a heuristic (the block of a value is the nearest class name). Step 2 must read it, not trust it.

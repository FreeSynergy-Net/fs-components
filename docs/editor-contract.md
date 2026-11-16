# Editor contract (fs-components-editor), version 2

Several editor building blocks stand behind one trait, `TextEditor` (pattern: Strategy).

- **Input:** a `Doc`, the document in the text model (blocks of Markdown source, one blank line between, one line break at the end).
- **Output:** only change operations of the same model: `Op::InsertBlock`, `Op::RemoveBlock`, `Op::ReplaceBlock`.
- **Input events (`Edit`):** `Type`, `Enter`, and since version 2 `Backspace` (one character, at the start of a block what a person expects: join, leave a list) and `ToggleTask` (a click on the box of a task line). An editor that has input rules (`TextEditor::input_rules()`, shortcuts such as `## ` or `- ` that turn into blocks at once) says so; for such an editor the collection checks the shortcuts and the Backspace that gives one back.
- **Version:** `CONTRACT_VERSION` is 2. Version 2 is **not additive** in the API: `Edit` got two variants and is deliberately not `non_exhaustive`, so a `match` on it outside the crate no longer compiles until it decides what to do with them. `input_rules()` is a provided method (additive).
- **Rule:** an editor changes data only through these operations and never reformats on its own. A document that is loaded and saved without a change is byte for byte the same, and every change is visible as an operation. `apply(loaded, take_ops())` is exactly `document()`.

## Building blocks

| Name | What it is | Status |
|---|---|---|
| `SourceEditor` | plain text (Markdown, code, configuration) in one text area; first implementation: the classic and Confluence-like editor | built |
| `BlockEditor` | Notion-like: every block a field of its own, Enter makes a new block, `/` opens the block menu | built (menu: block types only) |
| `RichTextEditor` | WYSIWYG: the typed block tree `RichEditor`/`Session` in `fs-components-editor::rich` and the writing surface `fs-components-rich` (stage 1: paragraphs, headings, lists, tasks, code, quotes, rules, shortcuts, a cursor over the blocks) | stage 1 built; tables, copy and paste, selection over blocks, undo, IME later |

A building block is not an editor program. Opening, saving and the choice of block and frontend belong to the program (later a facet of fs-text). The style is a layer of its own (ADR 0074); each block has a fixed name for its style parts.

The rich editor keeps the live form of the document (what was typed is what the source holds, so a shortcut given back is `## ` and not `\## `). A program that **stores** the page stores `RichEditor::save_form()` (or `Session::save_form()`): there a paragraph that would be read as something else (`## x`, `- x`, `1. x`, `[x] x`, `> x`) gets a backslash in front, so loading it gives the same blocks. A test reads, writes and reads again for every kind of node. Blocks nobody changed are given back byte for byte in both forms.

## Why a crate of its own

The text model of the wiki (`fs-text`, repository fs-wiki) depends on fs-components, so fs-components cannot depend on it. The contract has small block-addressed operation types of its own; the wiki maps them onto its model.

## Conformance tests

One collection, `fs_components_editor::conformance::check`, is run by every implementation (`crates/fs-components-workbenches/tests/conformance.rs`): the text of the user of 2026-11-14 and the GFM cases (table alignment, task lists, `~strike~`, `~~strike~~`, `_italic_`, `*italic*`), round trip byte for byte, every edit visible as operations. A new building block has to pass it. Version 2 adds cases for Backspace, the task box and, for editors with input rules, the shortcuts, each with a counter-case; the fixtures include a list item with a lazy line and a table in a quote (typing there must show as an operation). The numbers of numbered items are not content (`3. ` is compared as `#. `), but that there is a list is.

## Extension points (declared, not built)

Modules like hot plugins (ADR 0045), one trait per block: `SourceModule` (languages, language server, preview), `BlockModule` (block types, `/` menu entries), `RichModule` (marks, bar buttons), kept in a `Registry`. The traits are empty for now.

## Limits

Text that is not in the canonical form (several blank lines, no line break at the end) is normalised by the block editor; the fixtures are canonical.

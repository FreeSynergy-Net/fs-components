# Editor-Vertrag (fs-components-editor), Version 2

Mehrere Editor-Bausteine stehen hinter einem Trait, `TextEditor` (Muster: Strategy).

- **Eingabe:** ein `Doc`, das Dokument im Text-Modell (Blöcke aus Markdown-Quelltext, eine Leerzeile dazwischen, ein Zeilenumbruch am Ende).
- **Ausgabe:** nur Änderungs-Operationen desselben Modells: `Op::InsertBlock`, `Op::RemoveBlock`, `Op::ReplaceBlock`.
- **Eingabe-Ereignisse (`Edit`):** `Type`, `Enter` und seit Version 2 `Backspace` (ein Zeichen, am Blockanfang das, was man erwartet: anfügen, Liste verlassen) und `ToggleTask` (Klick aufs Kästchen einer Aufgabenzeile). Ein Editor mit Eingaberegeln (`TextEditor::input_rules()`, Kürzel wie `## ` oder `- `, die sofort zum Block werden) sagt das; für ihn prüft die Sammlung die Kürzel und die Rücktaste, die eines zurückgibt.
- **Version:** `CONTRACT_VERSION` ist 2. Version 2 ist in der API **nicht additiv**: `Edit` bekam zwei Varianten und ist absichtlich nicht `non_exhaustive`, ein `match` darauf außerhalb des Crates übersetzt nicht mehr, bis es entscheidet, was es damit tut. `input_rules()` ist eine Methode mit Vorgabe (additiv).
- **Regel:** Ein Editor ändert Daten nur über diese Operationen und formatiert nie eigenmächtig um. Ein Dokument, das ohne Änderung geladen und gespeichert wird, bleibt Byte für Byte gleich, und jede Änderung ist als Operation sichtbar. `apply(geladen, take_ops())` ergibt genau `document()`.

## Bausteine

| Name | Was es ist | Stand |
|---|---|---|
| `SourceEditor` | reiner Text (Markdown, Code, Konfiguration) in einer Textfläche; erste Umsetzung: der Classic- und Confluence-Editor | gebaut |
| `BlockEditor` | Notion-Art: jeder Block ein eigenes Feld, Enter macht einen neuen Block, `/` öffnet das Blockmenü | gebaut (Menü: nur Blocktypen) |
| `RichTextEditor` | WYSIWYG: der getippte Blockbaum `RichEditor`/`Session` in `fs-components-editor::rich` und die Schreibfläche `fs-components-rich` (Stufe 1: Absätze, Überschriften, Listen, Aufgaben, Code, Zitate, Linien, Kürzel, ein Cursor über die Blöcke) | Stufe 1 gebaut; Tabellen, Kopieren und Einfügen, Auswahl über Blöcke, Rückgängig, IME später |

Ein Baustein ist kein Editor-Programm. Öffnen, Speichern und die Wahl von Baustein und Frontend gehören dem Programm (später eine Facette von fs-text). Der Stil ist eine eigene Schicht (ADR 0074); jeder Baustein hat feste Namen für seine Stilteile.

Der Rich-Editor hält die lebende Form des Dokuments (was getippt wurde, steht in der Quelle; ein zurückgegebenes Kürzel ist also `## ` und nicht `\## `). Ein Programm, das die Seite **speichert**, speichert `RichEditor::save_form()` (oder `Session::save_form()`): dort bekommt ein Absatz, der sonst als etwas anderes gelesen würde (`## x`, `- x`, `1. x`, `[x] x`, `> x`), einen Rückstrich davor, damit das Laden dieselben Blöcke ergibt. Ein Test liest, schreibt und liest wieder für jede Knotenart. Blöcke, die niemand änderte, kommen in beiden Formen Byte für Byte zurück.

## Warum ein eigenes Crate

Das Text-Modell des Wikis (`fs-texts`, Repo fs-texts) hängt von fs-components ab, also kann fs-components nicht davon abhängen. Der Vertrag hat kleine blockadressierte Operationstypen; das Wiki bildet sie auf sein Modell ab.

## Konformitätstests

Eine Sammlung, `fs_components_editor::conformance::check`, läuft für jede Umsetzung (`crates/fs-components-workbenches/tests/conformance.rs`): der Text des Nutzers vom 2026-11-14 und die GFM-Fälle (Tabellenausrichtung, Aufgabenlisten, `~durch~`, `~~durch~~`, `_kursiv_`, `*kursiv*`), Round-trip Byte für Byte, jede Änderung als Operation sichtbar. Ein neuer Baustein muss sie bestehen. Version 2 fügt Fälle für Rücktaste, Aufgabenkästchen und, für Editoren mit Eingaberegeln, die Kürzel hinzu, je mit Gegenfall; die Fixtures enthalten einen Listeneintrag mit träger Zeile und eine Tabelle im Zitat (Tippen dort muss als Operation sichtbar werden). Die Nummern nummerierter Einträge zählen nicht als Inhalt (`3. ` wird als `#. ` verglichen), dass es eine Liste ist, aber schon.

## Erweiterungspunkte (nur deklariert, nicht gebaut)

Module wie Hot-Plugins (ADR 0045), ein Trait je Baustein: `SourceModule` (Sprachen, Language Server, Vorschau), `BlockModule` (Blocktypen, Einträge des `/`-Menüs), `RichModule` (Marken, Leistenknöpfe), gehalten in einer `Registry`. Die Traits sind vorerst leer.

## Grenzen

Text, der nicht in der kanonischen Form steht (mehrere Leerzeilen, kein Zeilenumbruch am Ende), wird vom Block-Editor vereinheitlicht; die Fixtures sind kanonisch.

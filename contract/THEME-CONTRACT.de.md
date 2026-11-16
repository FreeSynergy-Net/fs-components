# Theme-Vertrag von fs-components

**Version 0.2 (Entwurf; 0.1 wurde aus fs-components v0.0.12 abgeleitet, 0.2 fügt die Bausteine von 0.0.23 hinzu).** ADR 0074, Schritt 1. Englische Fassung: [THEME-CONTRACT.md](THEME-CONTRACT.md).

Der Vertrag ist die Liste der **Bausteine** (der Stücke, aus denen ein Fenster besteht), jeder mit festen **Teilen** und **Zuständen** (und, wo sinnvoll, **Varianten**). Ein Stil (Theme) gestaltet nur diese Namen. So funktioniert er auf jedem Programm, auch auf künftigen. Programme und Frontends enthalten keine Farben, Abstände oder Schriften (ADR 0074, Abschnitt 2). Das ist der erste Entwurf: Er wurde **aus dem abgeleitet, was der Code heute tut**, nicht erfunden. Was bleibt, entscheidet die Prüfung.

## Dateien

| Datei | Inhalt |
|---|---|
| `contract.toml` | Version, wo die anderen Dateien liegen, welche Namenspräfixe als Klassennamen zählen, die Standard-Zustände. |
| `blocks/*.toml` | Die Bausteine: Name, Herkunft des Namens, Geltungsbereich, Teile, Zustände, Varianten, Einzeiler in EN und DE. |
| `mapping/*.toml` | Jede Klasse oder Id von heute (fs-components v0.0.12, der Store und die zwei laufenden Zweige) und der Vertragsname, der daraus wird. |
| `exceptions.toml` | Namen im Quelltext, die keine Klasse sind (Thread-Namen, Zustands-Schlüssel, Handle-Ids), je mit Grund. |
| `BLOCKS.md`, `BLOCKS.de.md`, `MAPPING.md` | Erzeugte Tabellen (`contract/render`). Nicht von Hand ändern. |
| `HARDCODED-VALUES.md` | Erzeugte Liste der festen Werte im Code (`contract/hardcoded`), Grundlage für Schritt 2. |
| `check`, `selftest`, `render`, `hardcoded` | Python-3-Skripte ohne Abhängigkeiten und ohne Netz (der Code liegt in `lib/contractlib/`). |

Ein Format: TOML. Die Dateien sind nur aufgeteilt, damit sie kurz bleiben; `contract.toml` listet sie.

## Regeln

1. **Baustein-Namen.** Ein Baustein heißt nach der WAI-ARIA-1.2-Rolle, wo eine passt (`button`, `menu`, `tab`, `tree`, `grid`, `dialog`, `alert` ...). Wo Open UI einen gebräuchlichen Komponentennamen hat, nach Open UI (`card`, `tag`, `badge`, `avatar`, `breadcrumb`, `text`). Sonst ein eigener Name **mit schriftlichem Grund** (`naming = "own"`, `reason`). Bei `aria` und `open-ui` ist der Bausteinname gleich dem Rollen- oder Komponentennamen; `check` prüft beides gegen die Listen in `lib/contractlib/vocab.py`.
2. **Namen nur `[a-z0-9-]`**, keine Großbuchstaben, keine Unterstriche, keine Doppelungen (Bausteine, Teile, Zustände, Varianten).
3. **Jeder Baustein hat Teile und Zustände** und eine Kurzbeschreibung in beiden Sprachen. Hat ein Baustein nichts zu melden, steht `default` (ausdrücklich, nicht stillschweigend).
4. **Teile** sind die benannten Stellen im Baustein (`program-card`: `icon`, `title`, `summary`, `badge`, `action`). **Zustände** sind Lagen des Bausteins oder eines Teils (`installed`, `update`, `busy`, dazu ARIA-Zustände wie `selected`, `expanded`, `disabled`). **Varianten** sind Aussehen, die das Programm wählt und die sich nicht ändern (ein Knopf ist `primary` oder `outline`; ein Tag hat einen Ton). Ein Zustand ändert sich, während das Programm läuft.
5. **Geltungsbereich** sagt, wem der Baustein gehört: `general` (jedes Programm), `block` (ein Widget oder Crate von fs-components), `program` (reicher Baustein eines Programms: Programmkarte, Aufgabenzeile, Seitenbaum-Zeile ...). Reiche Bausteine sind gewollt: Ein Stil kann eine Store-Karte, eine Board-Spalte oder eine Aufgabenzeile nur gezielt gestalten, wenn jede einen eigenen Namen hat.
6. **Ziel-Schreibweise** im Mapping: `baustein[.teil][/variante][:zustand]`, zum Beispiel `button/primary:disabled`, `program-card:installed`. Das Ziel `-` ist nur mit Status `remove` erlaubt.
7. Wie die Vertragsnamen in CSS- oder Blinc-Selektoren aussehen, wird **hier nicht entschieden** (Schritt 3 mit den Stilen). Das Mapping sagt nur, welcher alte Name zu welchem Vertragsnamen wird.

## Status eines abgebildeten Namens

| Status | Bedeutung |
|---|---|
| `mapped` | Der Name hat seinen Platz; beim Umstellen (Schritt 4) ändert sich nur der Name. |
| `provisional` | Der Platz existiert, aber das Programm oder die Workbench ist noch vorläufig (README: alle Workbenches außer dem Reader), oder das Ziel ist eine Vermutung. Braucht eine Entscheidung. |
| `remove` | Kein Vertragsname. Der Name ist Anordnung (Abstandshalter, Breite) oder das Aussehen eines Stils (alle `craft-*`-Klassen): Tokens oder das Frontend übernehmen ihn. |

Namen in `mapping/other-places.toml` stammen aus dem Store (`external`) und den Zweigen `tasks-visual` (`fs-plan-*`) und `wiki-frontends-1` (`fs-wk-*`, nur gelesen) (`coming`). Die Prüfung sucht sie nicht in diesem Repository.

## Stand bei Version 0.1

Aktuelle Zahlen: `contract/check --stats`. Zum Zeitpunkt des Schreibens: 67 Bausteine (34 allgemein, 21 baustein-spezifisch, 12 programm-spezifisch). Von den 474 Namen aus `.class("..")`/`.id("..")` in fs-components (Produktion und Tests): 261 abgebildet, 127 vorläufig, 27 zu entfernen, 58 nur in Tests (außerhalb des Vertrags), 1 Ausnahme (`tabindex=0`, das ist ein Fehler, keine Klasse: `.class("tabindex=0")` in `file_manager/view/actions.rs`). Dazu benutzt der Quelltext 82 weitere Namen, die kein `.class(..)`-Aufruf ausschreibt (CSS-Selektoren, Konstanten, Match-Arme); 13 davon sind keine Klassen (Thread-Namen, Schlüssel, Handle-Ids, ein Crate-Name, ein Cargo-Feature) und stehen auf der Ausnahmeliste, die übrigen sind abgebildet. Nichts bleibt unabgebildet.

## Was die Skripte beweisen

- `contract/check`: Jede Klasse oder Id in `crates/*/src` (`.class("..")`, `.id("..")`, jedes Wort einer Mehrklassen-Zeichenkette und jeder Name mit Präfix `fs-`, `craft-` oder `cn-` in einer Codezeile, das fängt CSS-Texte, Konstanten und Match-Arme) steht im Mapping oder auf der Ausnahmeliste; kein Mapping und keine Ausnahme ist veraltet; jeder Baustein hat Teile, Zustände, Kurzbeschreibung (EN, DE), einen gültigen Geltungsbereich und einen begründeten Namen; Namen sind `[a-z0-9-]`; Mapping-Ziele zeigen auf echte Bausteine, Teile, Zustände und Varianten; nirgends Doppelungen; die erzeugten Tabellen sind aktuell.
- `contract/selftest`: Beschädigt eine Kopie des Repositories auf 25 Arten (neue Klasse, entfernte Mapping-Zeile, Baustein ohne Teile, Großbuchstabe, Doppelung, veraltete Zeile, von Hand geänderte Tabelle ...) und verlangt, dass `check` jedes Mal mit der richtigen Meldung scheitert. Es beweist auch, dass `hardcoded` einen neuen festen Wert bemerkt.
- `contract/hardcoded --check`: `HARDCODED-VALUES.md` ist aktuell. Das gehört bewusst nicht zu `check`, weil jede Änderung eines Pixelwerts im Code es sonst brechen würde.

Grenzen, damit niemand der Prüfung zu viel traut: Die Suche ist textuell. Eine Klasse, die zur Laufzeit gebaut wird (`format!`, Variable), wird nicht gesehen; mit `format!` gebaute Ids (`fs-app-menu-{index}`, `{prefix}-card-{id}`, `cal-item-{key}`) sind Element-Kennungen, keine Stil-Haken, und liegen außerhalb des Vertrags. Ein Klassenname ohne `fs-`, `craft-` oder `cn-` wird nur in `.class("..")`/`.id("..")` gefunden. Ein Name, den nur Testdateien benutzen, wird nicht geprüft (`mapping/test-only.toml` listet die 58, die es gibt). Die Prüfung schaut weder in den Store noch in die Zweige. Ein Mapping kann dem Sinn nach falsch sein und trotzdem bestehen: Die Prüfung muss es lesen.

## Nicht übernommen aus den Berichten

Die mit `[nicht belegt]` markierten Zahlen der Berichte werden nicht benutzt. Die ARIA-Rollenliste in `vocab.py` ist die Liste von 1.2 einschließlich `code`, 82 Namen; der Bericht zählte 81 und seine Tabelle hat kein `code`, also irrt einer von beiden, und die Prüfung sollte es am W3C-Text nachsehen. WAI-ARIA 1.3 wurde nicht angesehen. Die Open-UI-Liste sind die 33 Research-Komponenten von open-ui.org, wie der Bericht sie nennt; die Komponenten-Matrix war nicht lesbar und wird nicht benutzt.

## Tokens über Blincs Satz hinaus (0.0.21)

Blincs Theme hat einen festen Satz Farb-Tokens. Ein Aussehen, das der Satz nicht sagen kann, bekommt ein eigenes Token, einmal in `fs-components-styles` definiert (`styles/control_tokens.rs`) als `:root`-Variable über den Tokens des Themes; es folgt so dem hellen und dem dunklen Schema jedes Stils. Ein Stil darf es überschreiben (Craft tut es).

| Token | Rolle | Standard | Craft | Maß (Test) |
|---|---|---|---|---|
| `--control-border` | Rand eines Kästchens, das man abhakt (`.cn-checkbox`, leer) | 55 % `--text-primary` über `--surface` | gleich | 3:1 zur Seite (WCAG 2.1, 1.4.11; `wiki_tokens.rs`, `wiki_ground_craft.rs`) |
| `--code-bg` | Grund eines Code-Blocks (`.fs-tv-code` in den Wiki-Widgets, `.fs-re-code` auf der Schreibfläche) | 5 % `--text-primary` über `--surface` | 12 % | nicht der Hover-Grund `--accent-subtle` (ein Code-Block darf beim Überfahren seiner Zeile nicht verschwinden); Craft 1,3:1 zur Seite |

Der Ton eines Symbols (`Tone`) ist kein Token: ein Symbol auf der Auswahl nimmt `Tone::Normal`, und ein kleines Symbol wird mit einer Linie von mindestens 1,5 Geräte-Pixeln gezeichnet (`icon_stroked`), weil die eigene Linie des Stils (0,7 px bei 14 px) nie 3:1 erreicht.

## Version 0.2: Dialoge, Toast, Schalter, Segmentschalter, Platzhalter, Teil-Ergebnis-Hinweis (0.0.23)

Version 0.2 fügt Bausteine, Teile, Zustände und Varianten hinzu; kein Name aus 0.1 fällt weg. Nach dem festen Schritt `alle-stile-erweitern` gibt jeder Stil für jeden neuen Baustein einen ausdrücklichen Eintrag.

| Baustein | Wo | Neu in 0.2 |
|---|---|---|
| `confirm-dialog`, `choice-dialog`, `license-dialog` | `fs-components-widgets/src/dialogs/` | neue Bausteine; Zustände `danger`, `busy` (Bestätigen), `selected`, `disabled` (Auswahl-Option) |
| `dialog` | dort | neuer Zustand `danger` (der Rahmen, den alle drei teilen) |
| `toast` | `src/toast/` | neuer Baustein; Varianten `info`, `ok`, `warning`, `error`; Teile `stack`, `more`, `root`, `icon`, `text`, `action`, `close` |
| `alert` | `src/states/` | neuer Teil `title`, neue Variante `ok` (die Fehlerfläche ist `alert/error` mit der Aktion Wiederholen) |
| `partial-notice`, `skeleton`, `empty-state` | `src/states/` | `partial-notice` und `skeleton` neu (Varianten `line`, `block`, `circle`; Zustand `busy`); `empty-state` hat jetzt Code |
| `switch`, `segmented` | `fs-components-fields/src/controls/` | neue Bausteine; `switch` Zustände `checked`, `mixed`, `disabled` |

**Tokens der Bausteine (ein Eintrag je Stil, ausgeschrieben in `fs-components-styles/src/styles/block_tokens.rs`).** Ein neuer Stil übersetzt erst, wenn er seinen Eintrag hat (`look` ist ein vollständiges `match` über `StyleId`). Die Tokens sind `:root`-Variablen über den eigenen Tokens des Themes, die Farben folgen also dem hellen und dem dunklen Schema; die Zahlen gehören dem Stil: `--scrim`, `--dialog-width`, `--dialog-shadow` (aus dem Überlagerungs-Schatten des Stils, hell), `--dialog-radius`, `--dialog-pad`, `--message-radius`, `--message-pad`, `--message-tint` (Anteil der Farbe der Stufe im Grund eines Toasts, Alerts oder Teil-Ergebnis-Hinweises), `--toast-width`, `--toast-edge` (0 = kein farbiger Rand: Symbol und Tönung sagen die Stufe), `--skeleton-base`, `--skeleton-radius`, `--skeleton-height`, `--switch-width`, `--switch-height`, `--segment-height`, `--segment-pad`, `--segment-on-bg`, `--segment-on-line`, `--segment-on-line-width`, `--segment-on-text`, `--switch-on-track`, `--switch-on-thumb`, `--switch-off-track`, `--switch-off-thumb`, `--switch-disabled-opacity`, `--alert-info-bg`, `--alert-ok-bg`, `--alert-warning-bg`, `--alert-error-bg`, `--dialog-danger-edge`, `--dialog-head-line`, `--dialog-head-gap`, `--danger-bg`, `--danger-bg-hover`, `--danger-text`, `--danger-line`, `--danger-line-width` (`block_marks.rs`, ausgedrückt über Variablen des Themes; von den letzten fünf: `--danger-bg` der Grund des Gefahr-Knopfs, `--danger-bg-hover` sein Grund unter dem Zeiger, `--danger-text` seine Schrift, `--danger-line` sein Rand, `--danger-line-width` die Breite dieses Randes, 0 = keiner). `BLOCK_TOKENS` sagt, welcher Baustein welches Token liest.

**Was es beweist.** `crates/fs-components-styles/tests/block_tokens.rs` (Vertragsversion, Bausteine von 0.2 gegen `BLOCK_TOKENS`, jedes Token für jeden Stil und jede Erscheinung, keine zwei Stile gleich); `crates/fs-components-fields/tests/blocks_styles.rs` (jeder Baustein in jedem Stil, hell und dunkel, mit Theme-Wächter; Bildpaare müssen sich unterscheiden, ein unverändertes Bild ist rot; keine Variable der neuen Blätter ohne Definition); `blocks_contrast.rs` (WCAG-Verhältnisse, an den Bildern gemessen). Die Workflow-Skripte `stil-vollstaendigkeit` und `zwei-stile` von `frontend-umbau` gibt es noch nicht; diese Tests sind ihre Prüfung für die Bausteine von 0.2.

## Offene Punkte (für die Prüfung)

- Ist `provisional` das richtige Wort für eine Workbench-Rolle, die sauber auf einen allgemeinen Baustein passt (zum Beispiel `fs-tasks-failed` auf `alert/error`)? Sie ist nur vorläufig, weil die Workbench es ist.
- Mehrere Store-Bausteine (`program-row`, `program-detail`) sind aus CSS-Namen des Stores rekonstruiert, nicht aus einem Entwurf. Ebenso `page-tree-row`, `page-block`, `quick-add`, `task-row`, `board-column`, die aus unveröffentlichten Zweigen stammen.
- Bausteine, die der Code noch nicht hat, die ADR 0074 aber erwartet (tooltip, switch, select, accordion, toast, slider ...), stehen nicht drin: Der Vertrag wurde aus dem Bestand abgeleitet. Open UI listet sie; sie kommen, wenn ein Programm sie braucht.
- Teile- und Zustandslisten enthalten Namen, die noch keine alte Klasse erreicht (349 von 433 Teilen werden erreicht). Das sind die Lücken, die ein Stil-Autor gefüllt braucht; manche können fallen.
- Das Mapping von `craft-*` auf `remove` nimmt an, dass Craft nach ADR 0074, Schritt 3 ein Theme wird und seine Klassen verschwinden.
- Farben, Größen, Schriften: `HARDCODED-VALUES.md`, Teil B, ist eine Heuristik (der Baustein eines Werts ist der nächste Klassenname). Schritt 2 muss es lesen, nicht darauf vertrauen.

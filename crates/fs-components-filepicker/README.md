# fs-components-filepicker

The file chooser of FreeSynergy. Rule: wherever a path is needed, a file chooser, never a path field as a lasting solution.

- **Trait** `FileChooser::open(OpenRequest) -> Result<Picked, ChooserError>`; `OpenRequest { title, mode (OpenFile, OpenFiles, SaveFile, Folder), filters (name, patterns, media types), initial_dir, suggested_name }`. A path only comes out of `Picked`, which only a chooser makes. A cancel is `ChooserError::Cancelled`, not a problem. Problems are a code with parameters: `portal-unavailable`, `portal-timeout` (`limit-ms`), `portal-denied` (`detail`), `io`, `path-missing`, `path-unreadable`, `wrong-kind`; their sentences are in `texts/{en,de}.mf2`.
- **`PortalChooser`** (Linux, feature `portal`, on by default): the xdg-desktop-portal `FileChooser` over the pinned crate `ashpd` 0.14.0 on our tokio (no second async runtime). The start of the portal may take 5 s (then `portal-timeout`); the dialog has no limit.
- **`PathFieldChooser`**: a typed path, checked (exists, right kind, readable), always marked "provisional". For Windows or where no portal answers. `default_chooser(&prompt)` picks the portal when a session bus is in sight and puts the path field behind it (`FallbackChooser`); only "no dialog" falls back, a refusal (`portal-denied`) and a cancel never do.
- **`FakeChooser`**: a scripted chooser for tests (paths, several, cancel, errors, a gate, thread ids).
- **`open` blocks**: call it only through `worker::pick_in_worker` (a worker thread, the answer by callback). Never on the window thread (the freeze of fs-people).
- **Blinc** (feature `ui`, on by default; the core has none): `ui::FilePickButton` (a label from MF2, the click starts the dialog in a worker, `on_picked`, states idle / waiting / chosen / failed, a cancel changes nothing) and `ui::PathPromptView` (the path field of the fallback). `ui::STYLES` is the style sheet (theme variables only).

**Honest limits.** The portal path is not proven in a headless test: the translation of filters (media types to portal filters), the URI-to-path step, the error mapping and the "no bus" case (child process with a bogus bus address) are; the real dialog needs a person and a portal (`tests/portal_live.rs`, `#[ignore = "portal"]`). The parent window is not passed (xdg-foreign): the dialog is not modal to our window yet. The path field takes one path (also for several files). Windows and macOS have no system dialog here yet.

## Auf Deutsch

Der Datei-Waehler von FreeSynergy. Regel: ueberall, wo ein Pfad gebraucht wird, ein Datei-Waehler; ein Pfadfeld nie als Dauerloesung.

- **Trait** `FileChooser::open(OpenRequest) -> Result<Picked, ChooserError>`; ein Pfad kommt nur aus `Picked`, das nur ein Waehler erzeugt. Abbruch ist `ChooserError::Cancelled`, kein Fehler. Probleme sind Code + Parameter (`portal-unavailable`, `portal-timeout`, `portal-denied`, `io`, `path-missing`, `path-unreadable`, `wrong-kind`), die Saetze stehen in `texts/{en,de}.mf2`.
- **`PortalChooser`** (Linux, Feature `portal`, Standard an): xdg-desktop-portal `FileChooser` ueber das gepinnte Crate `ashpd` 0.14.0 auf unserem tokio. Start des Portals hoechstens 5 s, der Dialog selbst hat keine Frist.
- **`PathFieldChooser`**: getippter Pfad, geprueft (existiert, richtige Art, lesbar), immer als "vorlaeufig" markiert. Rueckfall nur bei "kein Dialog"; Verweigerung und Abbruch fallen nie zurueck.
- **`FakeChooser`** fuer Tests. **`open` blockiert**: nur ueber `worker::pick_in_worker` aufrufen, nie im Fensterthread.
- **Blinc** (Feature `ui`): `ui::FilePickButton`, `ui::PathPromptView`.

**Ehrliche Grenzen.** Der Portal-Pfad ist im Headless-Test nicht belegt: belegt sind Filter-Uebersetzung, URI zu Pfad, Fehlerabbildung und der Fall ohne Bus (Kindprozess mit falscher Bus-Adresse); der echte Dialog braucht einen Menschen und ein Portal (`tests/portal_live.rs`, `#[ignore = "portal"]`). Das Elternfenster wird noch nicht uebergeben (xdg-foreign). Das Pfadfeld nimmt einen Pfad (auch bei mehreren Dateien).

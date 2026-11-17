# fs-components-commands

The command model of FreeSynergy programs, without Blinc and without dependencies: commands (state with a reason, check or radio, symbol, shortcuts), the arrangement of them per place as a tree (command, submenu, separator, group, placeholder), validation, `aria-keyshortcuts` shortcuts with `Primary` (Ctrl, or Cmd on macOS), their display per platform and language, and the report of shortcut conflicts. Views (menu bar, context menu, palette) live in `fs-components-app` and `-widgets`. Text is never held here, only the keys of the MF2 texts (ADR 0057).

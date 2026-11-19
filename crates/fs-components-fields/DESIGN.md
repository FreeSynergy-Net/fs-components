# fs-components-fields - Design

The four components of each basic type (ADR 0066, item 4): **input**, **display**, **compact**, **cell**, for e-mail address, name, phone number, postal address and date, in every style of `fs-components-styles`. A program never draws a field; it names a `Kind`, a `Mode` and a `Look`.

## Pattern map

| Where | Pattern | Why |
|---|---|---|
| `field.rs` | Strategy (trait `BasicField`) | what a type is: parts, rule, joined and short forms; written once per type, views written once for all |
| `kinds/*` | Strategy | one implementation per type; the rules come from the standard (`EmailAddress`, `PhoneNumber` of `fs-interfaces` `values`; JSContact `Name`/`Address`; RFC 3339 `full-date`) |
| `kinds/kind.rs` | Registry | `Kind` names a type, `Kind::field()` gives it |
| `suggest.rs` | Strategy (trait `Suggest`) | the command "suggest" of the type description: the program answers from its data, the input only asks |
| `problem.rs` | Value object | a code with a translation key; sentences are the program's (ADR 0057) |
| `view/look.rs` | Adapter | style data in, Blinc colours, text, icons out |
| `view/{display,compact,cell}.rs` | Adapter | Blinc elements from the type and the look |
| `view/input.rs`, `suggestions.rs` | Adapter + Observer | Blinc `text_input` per part (not `blinc_cn`'s: its heights are fixed at 32/40/48) plus a live area under the parts, outside the fields so a redraw never takes the keyboard |

## State

| Part | State |
|---|---|
| E-mail (local part, `@` fixed between, domain), name, phone (country, number), address, date (year, month, day) | done, 20 components, drawn and measured in 4 styles x light/dark |
| E-mail suggestions | done: the input asks `Suggest`, shows chips, a click fills the part. The program-side command is not wired (the type description of Thread T does not exist yet): `SuggestFromList` is the only implementation |
| `@` without the keyboard problem | an `@` typed or pasted into the first part is spread over both (`settle`), the focus goes on to the domain |
| Value types for name, address, date | here, local to this crate (provisional); move to the types crate of Thread T when it is tagged |
| Per-language order and format (date, name, address) | **Unsupported**: ISO date, Western name order, street/postcode/locality order; the words crate has no date formatting yet |
| Focus by Tab between parts, sizes by style | parts are normal text inputs; the style gives their height, radius and colours (tested: the control height equals the style's) |
| Surfaces | `Look::surface` draws the chrome/overlay surface of a style: flat, glass (measured: blurs), **Backdrop (Mica) is Unsupported**: drawn as the flat tint, Blinc has no compositor blur (git-bug in the Blinc clone). The frontend places the surface, components do not paint a ground |
| Focus and hover look of the input | **partly Unsupported**: Blinc draws hover and focus of a text input with its own theme (`ua_styles`), so the style's focus colour and ring (`Palette::focus`, `Effects::focus`) are not applied to the inputs yet. Text and idle colours are the style's. Possible cause of the "white text while typing" hand test (not reproduced in the live probe, not confirmed) |
| Pasting a whole value into one part | e-mail and date spread it (`settle`); name, phone, address do not (Unsupported) |
| Rules of the standard types in `fs-types` (Thread T, `fs-interfaces` v0.0.4: `BasicRules` check/suggest, `Date`, `Name`, `Address`) | **not used yet**: this crate still has its own date, name and address rules and the `Suggest` list. To do: delegate rules to `fs-types`, keep only the parts and the views here |

## Handtest errors of the input (2026-11-10)

- AltGr+Q gives "2": cause found in Blinc (`blinc_platform_desktop` `convert_key` folds `@` into `Key::Num2`, `blinc_app` `keyboard.rs` writes `2` back for it). Recorded in the Blinc clone as a git-bug entry with the proof; this crate cannot repair it: `settle` acts only on an `@` that arrives (typed with a layout that gives it, or pasted). **AltGr and layout keys are Unsupported until Blinc is fixed**: typing `ada@x.org` on a German layout silently gives the local part `ada2x.org`, and `+` becomes `=` in the phone country part (`tools/live-probe/scnD-keys.txt`).
- White text while typing and sluggishness: **open, not reproduced, not fixed here.** The only measurement is the headless test `a_key_costs_little` (under 50 ms per key), which does not reproduce the slowness seen by hand; the live probe (`tools/live-probe`) did not show white text. Blinc's own focus/hover theme is a possible cause (see the focus row above). Thread B measures typing latency in Blinc.

## Field bundle 1a: link and many-line text (2026-11-19)

| Part | What it is |
|---|---|
| `Kind::Link` (block `field-link`, basic type `link`) | One part, the address. Only `http`, `https`, `mailto`, `tel` are links (`javascript:`, `data:`, `file:`, `ftp:` are refused: `Problem::LinkScheme`); no scheme, no host, spaces and control characters each have their own problem. The display and the compact line draw a good link as a link: role `link`, attribute `href`, underlined, a tab stop; a link that is not good is plain text and opens nothing. `display_with`/`compact_with` take what follows the link (`Follow`); the default opens the system's program (`open_external`) |
| Link ink | `link_ink(palette)`: the accent as text moved toward the text colour in steps of 5 per cent until it reads 4.5:1 on the ground AND the panel. Admin, Simple and Windows keep their token; Apple's own blue (3.5:1) is darkened/lightened. Told by underline and colour, never colour alone |
| Link key | The window turns Enter AND Space into a click on the focused element (`Key::Activate`), so Space follows too (WAI-ARIA `link`: Enter only). **Limit, not fixable at this layer**; tested as it is |
| `Kind::Multiline` (block `field-multiline`, basic type `multilineText`) | One part, typed into a Blinc text area (`BasicField::multiline`, `FieldInput` holds `areas` instead of `data`); the stored text has `\n` line ends, no blank lines or spaces at its end; the compact line and the cell show the first line with text and a mark ` \u2026` when more follows. Control characters other than line break and tab are a problem |
| Not in this bundle | the switch exists in `controls/` (`fs-components-fields::controls::Switch`, nothing rebuilt); number (spinbutton) and colour follow in 1b/1c: the catalog has no word for `integer`/`whole` yet and the colour has no basic type |

### Bundle 1a, nachgetragen (keys, ARIA, words, pictures)

| Part | What it is |
|---|---|
| Many-line input, keyboard | The Blinc text area is the one tab stop: typing, arrows, Home/End, Enter (a line break, it never submits), Tab leaves. Role and `aria-multiline` are the text area's own (Blinc 3.2 `text_area` sets no ARIA attribute: the role `textbox` is the element's default, `aria-multiline` is **not set**, **Limit**: nothing in this crate can add an attribute to a Blinc `TextArea`; the field id `<name>-text` names it for tests and automation) |
| Link, keyboard | Tab to it, Enter follows once; Space follows too (the window gives Enter and Space as one `Key::Activate`); a click follows once |
| Words (ADR 0057) | The words are the program's; the keys are stable: `field.<key>`, `label.<key>.<part>`, `hint.<key>.<part>` and `problem.*` of `Problem::key`. Link: `field.link`, `label.link.url`, `hint.link.url`, `problem.link.character`, `.no-scheme`, `.scheme`, `.no-host`. Many-line text: `field.multiline`, `label.multiline.text`, `hint.multiline.text`, `problem.multiline.control-character`. There is no word file (MF2 or other) in this repository for any field; the program ships de and en (MF2) and the tests use `KeysAsText` |
| Pictures | `tests/f1_pictures.rs` draws link display and many-line input in Admin, light and dark, and keeps the PNGs when `FS_COMP_SHOTS` names a directory |

## Field bundle 1b: yes/no and whole numbers (2026-11-19)

| Part | What it is |
|---|---|
| Basic types | `fs-types` v0.0.4 has `flag` (19), `integer` (17) and `whole` (20); the rules are its own (`Flag`, `Integer`, `Whole`: optional sign, ASCII digits, `i64` / `u64`, canonical decimal, `yes`/`no` read), not copies. The catalog now has their words (`basic_key`: `flag`, `integer`, `whole`; `BASICS` 16 -> 19); `duration` is bundle 3. `colour` has NO basic type: no `Kind` for it (decision with the owner) |
| `Kind::Flag` (block `field-flag`) | One part: `true`, `false` or empty (not answered; a form shows the switch off and gives no value until it is flipped). The input is the existing `Switch` of the controls (role `switch`, `aria-checked`, Space/Enter/click flip once, tokens `--switch-*`); `Entry::Switch` in `BasicField::entry`. The display says the value in words (`value.flag.true`, `value.flag.false` through `Say`); the compact line and the cell have no `Say` and draw a check or a cross (`BasicField::mark`, `view/shown.rs`) |
| `Kind::Integer`, `Kind::Whole` (blocks `field-integer`, `field-whole`) | One part, a Blinc `text_input` with `InputType::Integer` (digits and a sign) and the stepper hook: the arrow keys step by 1 (`kinds::stepped`: from nothing at 0, `whole` never below 0, the end of the 64-bit range stays, no wrap). `FieldInput::step(delta)` does the same from code. The element around the field has the role `spinbutton` (id `<name>-spin`), `aria-valuemin` 0 for `whole`, `aria-valuenow` = the text when the form was drawn |
| Why not `cn::number_input` | Blinc 3.2 `number_input` holds an `f64` (a 64-bit integer above 2^53 would be rounded: tested with 2^63-1), has three fixed heights (the styles need their own) and sets no role. Its parts that matter are used: `InputType::Integer`, `on_step` |
| Limits | (1) Blinc's stepper hook consumes the keys `+`, `-` and `=` as steps, so a minus cannot be TYPED for `integer`; a negative number comes by stepping down (arrow down or `-`) or by paste. (2) The role sits on the wrapper because a Blinc `text_input` takes no attribute; `aria-valuenow` is not live. (3) No visible plus/minus buttons and no wheel stepping: only the keys. Both are requests for Blinc (a typed sign next to a stepper hook; attributes on `text_input`) |
| Many-line and number inks | A text area has no colour setter in Blinc: `area_styles()` (one class per style and scheme, concrete palette colours) must be installed with the other sheets; the Admin window does |
| Words | `field.flag`, `label.flag.flag` (the words of the switch), `value.flag.true`, `value.flag.false`, `problem.flag.bad`; `field.integer`, `field.whole`, `label.integer.number`, `label.whole.number`, `hint.integer.number`, `hint.whole.number`, `problem.number.out-of-range`, `problem.not-a-number` |

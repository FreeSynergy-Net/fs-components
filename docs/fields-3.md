# Field bundle 3: date-time, duration, time zone

What Blinc 3.2 (upstream `cd3e196e`) carries, and why the fields are not the Blinc pickers.

| Field | Blinc 3.2 | Here |
|---|---|---|
| Date | `blinc_cn::date_picker`: a calendar popover, value `Option<CalendarDay>` (a day, no offset), role `dialog` on the popover | `Kind::Date` (already there) |
| Clock time | `blinc_cn::time_picker`: hour, minute, second as separate text inputs, value `TimeOfDay` ("a wall-clock time without a date or time zone") | not built: `fs-types` has no basic type for a clock time alone (question to the strategy) |
| Date and time | none (two separate pieces) | `Kind::DateTime`: RFC 3339 `date-time`, with offset |
| Duration | none | `Kind::Duration`: RFC 8984 / ISO 8601 subset, a number per unit |
| Time zone | none (no zone data) | `zone` module over the IANA database (`chrono-tz`), IANA name shown |

**Why the segment frame (`BasicField`) and not the Blinc pickers.** Neither picker can carry what the standards of the types need: the date picker returns a day without clock or offset, the time picker a time "without a date or time zone", so an RFC 3339 `date-time` (offset required, `Z` or `+01:00`) and an RFC 8984 duration (`P1DT2H30M`, weeks alone) cannot be written or read back through them; and our field frame already gives the four roles (input, display, compact, cell), the problem sentences (ADR 0097) and the rule from the contract crate (`fs_types::DateTime`, `Duration`). The Blinc pieces under the pickers (`text_input`) are what our input already uses.

**Times in the data (ADR rules of the data conventions).** The stored `dateTime` keeps the offset the person wrote; `kinds::utc_text` gives the same instant in UTC (`Z`) for a program that compares or stores instants. A zone is an IANA name (`docs/de/data/conventions.md`: "`timezone` muss eine IANA-Kennung sein"); a duration is an ISO 8601 duration, not a number of minutes (`docs/de/data/fields.md`). No conflict with the documentation was found.

**Time zone.** `zone::names()` lists the database as compiled in (`zone::DATABASE_VERSION`, tzdata 2025b with chrono-tz 0.10.4); `zone::check` asks the form of `fs_types::Zone::iana` and then whether the database has the name (a custom zone `/...` is not taken: it is only valid with its object). What a reader sees is the IANA name, never empty; the CLDR names ("Central European Time") are a follow-up, no CLDR package is in the stack. New in the lock: `chrono-tz` 0.10.4 (MIT OR Apache-2.0), `phf` 0.12.1 and `phf_shared` 0.12.1 (MIT), `siphasher` 1.0.4 (MIT OR Apache-2.0); `chrono` was in the lock already. `cargo deny check licenses bans sources`: licenses ok, sources ok; bans fail as before this bundle (wildcard path dependencies of our own crates, same on `main`).

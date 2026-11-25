//! The rows of the form `checklist@^1.1.0` read into the sections of a task list.
//!
//! Pattern: Adapter (rows by channel in, sections of [`TaskLine`] out) + Value object ([`Rows`]). It knows channels
//! (`label`, `done`, `sub`, `facts`, `group`, `order`), never the fields of a module; the words of a group come from the
//! host (`task.group.<name>`), the module's name for it is the fallback.

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::said_or;

use super::chip::{Chip, Tone};
use super::line::{TaskLine, Tick};
use super::list::Section;

/// The command of the form that ticks a row (writes `done`).
pub const TOGGLE: &str = "toggle";
/// The command of the form that opens a row (writes nothing).
pub const OPEN: &str = "open";

/// What the form says, ready to draw.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rows {
    /// The sections in the order a group first appears; rows without a group stand first in a section with no title.
    pub sections: Vec<Section>,
    /// `(key, done)` of every row, in the order of the list.
    pub done: Vec<(String, bool)>,
}

impl Rows {
    /// Whether the row `key` is ticked (`None` = no such row).
    #[must_use]
    pub fn is_done(&self, key: &str) -> Option<bool> {
        self.done.iter().find(|(k, _)| k == key).map(|(_, d)| *d)
    }

    /// The keys of all lines, in the order they stand on the screen.
    #[must_use]
    pub fn keys(&self) -> Vec<String> {
        self.sections
            .iter()
            .flat_map(|s| s.lines.iter().map(|l| l.key.clone()))
            .collect()
    }
}

/// Read `input`. `say` gives the sentence of a key (or the key itself); `can_toggle` says whether the person may tick a row
/// (the host's answer; a row that may not is drawn locked).
pub fn read_rows(
    input: &FormInput,
    say: &dyn Fn(&str) -> String,
    can_toggle: &dyn Fn(&str) -> bool,
) -> Rows {
    let mut rows: Vec<&FormRow> = input.rows.iter().collect();
    // `order` (a number) puts rows in order; a row without one, or with a text, stands after those that have one.
    rows.sort_by(|a, b| order_of(a).total_cmp(&order_of(b)));
    let mut out = Rows::default();
    let mut groups: Vec<Option<String>> = Vec::new();
    for row in rows {
        let done = row.get("done") == Some("true");
        let mut line = TaskLine::new(&row.key, row.get("label").unwrap_or(&row.key));
        line.sub = row.get("sub").map(str::to_owned);
        line.checked = done;
        if !can_toggle(&row.key) {
            line.tick = Tick::Locked;
        }
        line.facts = row
            .all("facts")
            .iter()
            .map(|v| Chip::new(v.trim(), Tone::Neutral))
            .collect();
        let group = row.get("group").map(str::to_owned);
        let at = groups.iter().position(|g| *g == group).unwrap_or_else(|| {
            let title = group.as_deref().map_or_else(String::new, |g| {
                said_or(
                    say(&format!("task.group.{g}")),
                    &format!("task.group.{g}"),
                    g,
                )
            });
            groups.push(group.clone());
            out.sections
                .push(Section::new(&title, Tone::Neutral, Vec::new()));
            groups.len() - 1
        });
        out.sections[at].lines.push(line);
        out.done.push((row.key.clone(), done));
    }
    // rows without a group stand first
    if let Some(none) = groups.iter().position(Option::is_none) {
        out.sections[..=none].rotate_right(1);
    }
    set_slots(&mut out.sections);
    out
}

fn order_of(row: &FormRow) -> f64 {
    row.get("order")
        .and_then(|o| o.trim().parse::<f64>().ok())
        .filter(|o| o.is_finite())
        .unwrap_or(f64::MAX)
}

/// Pixels a fact needs: its letters and the padding of a chip (the sizes of the sheet).
#[allow(clippy::cast_precision_loss, reason = "a chip is a few letters")]
fn chip_width(text: &str) -> f32 {
    text.chars().count() as f32 * 7.5 + 20.0
}

/// Every line of every section gets the same slot widths: slot `i` is as wide as the widest fact `i` of any line, so the
/// facts of the lines stand in columns and the right edge is the same everywhere.
fn set_slots(sections: &mut [Section]) {
    let count = sections
        .iter()
        .flat_map(|s| &s.lines)
        .map(|l| l.facts.len())
        .max()
        .unwrap_or(0);
    let widths: Vec<f32> = (0..count)
        .map(|i| {
            sections
                .iter()
                .flat_map(|s| &s.lines)
                .filter_map(|l| l.facts.get(i))
                .map(|c| chip_width(&c.text))
                .fold(0.0, f32::max)
        })
        .collect();
    for line in sections.iter_mut().flat_map(|s| &mut s.lines) {
        line.slots.clone_from(&widths);
        // a line with fewer facts than the others keeps its empty slots
        line.facts.resize(count, Chip::new("", Tone::Neutral));
    }
}

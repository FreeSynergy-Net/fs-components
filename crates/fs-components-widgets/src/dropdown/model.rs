//! The lines of a dropdown from the command model (M3): the dropdown reads the same resolved items as the title-bar menu.
//!
//! Pattern: Adapter (resolved items of `fs-components-commands` to [`Entry`]). A dropdown has one level, so a submenu is
//! written out below its own separator; a row of symbols becomes ordinary lines; a placeholder is skipped. A locked command is
//! shown, not chosen, and says why (never greyed out silently): the reason follows the label in brackets. Two separators in a row
//! never appear (a skipped line cannot leave one behind). The one `run` receives the id of the command that is chosen.

use std::sync::Arc;

use fs_components_commands::{Command, CommandId, Resolved, TextKey};

use super::panel::Entry;

/// The entries of a dropdown for `items`; `say` gives the text of a key (the program's MF2 texts), `run` is called with the id of a
/// chosen command.
pub fn entries_from_model(
    items: &[Resolved],
    say: &dyn Fn(&TextKey) -> String,
    run: &Arc<dyn Fn(&CommandId) + Send + Sync>,
) -> Vec<Entry> {
    let mut out = Vec::new();
    write(items, say, run, &mut out);
    while matches!(out.last(), Some(Entry::Separator)) {
        out.pop();
    }
    out
}

fn write(
    items: &[Resolved],
    say: &dyn Fn(&TextKey) -> String,
    run: &Arc<dyn Fn(&CommandId) + Send + Sync>,
    out: &mut Vec<Entry>,
) {
    for item in items {
        match item {
            Resolved::Command(command) => out.push(line(command, say, run)),
            Resolved::IconRow(row) => out.extend(row.iter().map(|c| line(c, say, run))),
            Resolved::Separator => separator(out),
            Resolved::Submenu { children, .. } => {
                separator(out);
                write(children, say, run, out);
                separator(out);
            }
            _ => {}
        }
    }
}

/// A separator, but never first and never twice in a row.
fn separator(out: &mut Vec<Entry>) {
    if !matches!(out.last(), None | Some(Entry::Separator)) {
        out.push(Entry::Separator);
    }
}

fn line(
    command: &Command,
    say: &dyn Fn(&TextKey) -> String,
    run: &Arc<dyn Fn(&CommandId) + Send + Sync>,
) -> Entry {
    let label = say(&command.label);
    match command.state.reason() {
        Some(reason) => Entry::Line {
            label: format!("{label} ({})", say(reason)),
            action: None,
        },
        None => {
            let (id, run) = (command.id.clone(), Arc::clone(run));
            Entry::Line {
                label,
                action: Some(Arc::new(move || run(&id))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use fs_components_commands::{Arrangement, Commands, MenuItem, resolve};

    use super::*;

    fn items() -> Vec<Resolved> {
        let list = Commands::new()
            .with(Command::new("open", "t-open"))
            .with(Command::new("save", "t-save").disabled("t-why"))
            .with(Command::new("a", "t-a").icon("x"))
            .with(Command::new("b", "t-b").icon("y"))
            .with(Command::new("zip", "t-zip"));
        let tree = Arrangement::new(
            "m",
            vec![
                MenuItem::command("open"),
                MenuItem::command("save"),
                MenuItem::Separator,
                MenuItem::icon_row("row", vec![MenuItem::command("a"), MenuItem::command("b")]),
                MenuItem::submenu("more", vec![MenuItem::command("zip")]),
                MenuItem::Placeholder("search".into()),
            ],
        );
        resolve(&tree, &list).unwrap()
    }

    fn say(key: &TextKey) -> String {
        format!("<{}>", key.as_str())
    }

    fn shape(entries: &[Entry]) -> Vec<String> {
        entries
            .iter()
            .map(|e| match e {
                Entry::Separator => "-".to_owned(),
                Entry::Line { label, action } => {
                    format!("{label}{}", if action.is_some() { "" } else { " [off]" })
                }
            })
            .collect()
    }

    #[test]
    fn the_lines_follow_the_model() {
        let run: Arc<dyn Fn(&CommandId) + Send + Sync> = Arc::new(|_| {});
        let entries = entries_from_model(&items(), &say, &run);
        assert_eq!(
            shape(&entries),
            [
                "<t-open>",
                "<t-save> (<t-why>) [off]",
                "-",
                "<t-a>",
                "<t-b>",
                "-",
                "<t-zip>"
            ]
        );
    }

    #[test]
    fn a_chosen_line_runs_with_its_id_and_a_locked_one_cannot() {
        let seen = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = Arc::clone(&seen);
        let run: Arc<dyn Fn(&CommandId) + Send + Sync> =
            Arc::new(move |id| sink.lock().unwrap().push(id.as_str().to_owned()));
        let entries = entries_from_model(&items(), &say, &run);
        for entry in &entries {
            if let Entry::Line {
                action: Some(action),
                ..
            } = entry
            {
                action();
            }
        }
        assert_eq!(*seen.lock().unwrap(), ["open", "a", "b", "zip"]);
        assert!(matches!(&entries[1], Entry::Line { action: None, .. }));
    }

    #[test]
    fn no_separator_stands_at_the_end() {
        let list = Commands::new().with(Command::new("a", "a"));
        let tree = Arrangement::new("m", vec![MenuItem::command("a")]);
        let mut resolved = resolve(&tree, &list).unwrap();
        resolved.push(Resolved::Separator);
        let run: Arc<dyn Fn(&CommandId) + Send + Sync> = Arc::new(|_| {});
        assert_eq!(entries_from_model(&resolved, &say, &run).len(), 1);
    }
}

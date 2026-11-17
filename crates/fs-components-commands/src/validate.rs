//! The checks of the model: command list and arrangement tree.
//!
//! Pattern: Visitor-like walk, collecting every problem (nothing stops at the first). A group is no line of its own, so its
//! items count as part of the level around it: a separator at the end of a group followed by one after it is "twice".

use std::collections::HashSet;

use crate::arrangement::{Arrangement, MenuItem};
use crate::command::{Command, CommandId, Toggle};
use crate::commands::Commands;
use crate::parameter::ResultType;
use crate::problem::Problem;

/// The problems of a command list: no id, an id twice, a locked command without a reason, a radio group without exactly one on.
#[must_use]
pub fn validate_commands(commands: &Commands) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    let mut reported = HashSet::new();
    let mut groups: Vec<&str> = Vec::new();
    for command in commands.iter() {
        if command.id.as_str().is_empty() {
            problems.push(Problem::EmptyCommandId);
        } else if !seen.insert(&command.id) && reported.insert(&command.id) {
            problems.push(Problem::DuplicateCommand(command.id.clone()));
        }
        if command
            .state
            .reason()
            .is_some_and(|r| r.as_str().is_empty())
        {
            problems.push(Problem::LockedWithoutReason(command.id.clone()));
        }
        params(command, &mut problems);
        if let Toggle::Radio { group, .. } = &command.toggle
            && !groups.contains(&group.as_str())
        {
            groups.push(group);
        }
    }
    for group in groups {
        let on = commands.radio_on(group).len();
        if on != 1 {
            problems.push(Problem::RadioGroupOn {
                group: group.to_owned(),
                on,
            });
        }
    }
    problems
}

/// The parameters and the result of one command: a name, no name twice, a type, a type that can be filled.
fn params(command: &Command, problems: &mut Vec<Problem>) {
    let mut names = HashSet::new();
    for param in &command.params {
        if param.name.is_empty() || !names.insert(param.name.as_str()) {
            problems.push(Problem::ParamName {
                command: command.id.clone(),
                name: param.name.clone(),
            });
        }
        match &param.ty {
            None => problems.push(Problem::ParamWithoutType {
                command: command.id.clone(),
                name: param.name.clone(),
            }),
            Some(ty) if ty.is_hollow() => problems.push(Problem::ParamHollow {
                command: command.id.clone(),
                name: param.name.clone(),
            }),
            Some(_) => {}
        }
    }
    if let ResultType::Value(ty) = &command.result
        && ty.is_hollow()
    {
        problems.push(Problem::ParamHollow {
            command: command.id.clone(),
            name: "result".to_owned(),
        });
    }
}

/// The problems of an arrangement against the commands: unknown or repeated commands, a separator at an edge or twice in a row,
/// a submenu without a line, a submenu or group without a name.
#[must_use]
pub fn validate_arrangement(arrangement: &Arrangement, commands: &Commands) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut used = HashSet::new();
    level(&arrangement.items, &[], commands, &mut used, &mut problems);
    problems
}

fn level<'a>(
    items: &'a [MenuItem],
    path: &[usize],
    commands: &Commands,
    used: &mut HashSet<&'a CommandId>,
    problems: &mut Vec<Problem>,
) {
    let mut flat = Vec::new();
    flatten(items, path, &mut flat, problems);
    for (position, (at, item)) in flat.iter().enumerate() {
        match item {
            MenuItem::Separator => {
                let edge = position == 0 || position + 1 == flat.len();
                if edge {
                    problems.push(Problem::SeparatorAtEdge { path: at.clone() });
                } else if matches!(flat[position - 1].1, MenuItem::Separator) {
                    problems.push(Problem::SeparatorTwice { path: at.clone() });
                }
            }
            MenuItem::Command(id) => {
                if commands.get(id).is_none() {
                    problems.push(Problem::UnknownCommand {
                        id: id.clone(),
                        path: at.clone(),
                    });
                } else if !used.insert(id) {
                    problems.push(Problem::CommandTwice(id.clone()));
                }
            }
            MenuItem::Submenu {
                label, children, ..
            } => {
                if label.as_str().is_empty() {
                    problems.push(Problem::EmptyName { path: at.clone() });
                }
                let mut inner = Vec::new();
                flatten(children, at, &mut inner, &mut Vec::new());
                if inner.iter().all(|(_, i)| matches!(i, MenuItem::Separator)) {
                    problems.push(Problem::EmptySubmenu { path: at.clone() });
                }
                level(children, at, commands, used, problems);
            }
            MenuItem::Group {
                children,
                inline_icons: true,
                ..
            } => icon_row(children, at, commands, used, problems),
            MenuItem::Group { .. } | MenuItem::Placeholder(_) => {}
        }
    }
}

/// A row of symbols: commands only, each known, used once and with a symbol.
fn icon_row<'a>(
    children: &'a [MenuItem],
    at: &[usize],
    commands: &Commands,
    used: &mut HashSet<&'a CommandId>,
    problems: &mut Vec<Problem>,
) {
    for (index, child) in children.iter().enumerate() {
        let mut here = at.to_vec();
        here.push(index);
        let MenuItem::Command(id) = child else {
            problems.push(Problem::IconRowNotCommand { path: here });
            continue;
        };
        match commands.get(id) {
            None => problems.push(Problem::UnknownCommand {
                id: id.clone(),
                path: here,
            }),
            Some(command) => {
                if !used.insert(id) {
                    problems.push(Problem::CommandTwice(id.clone()));
                }
                if command.icon.as_deref().is_none_or(str::is_empty) {
                    problems.push(Problem::IconRowWithoutIcon(id.clone()));
                }
            }
        }
    }
}

/// The items of a level with the items of its groups in place of the groups, each with its position in the tree.
fn flatten<'a>(
    items: &'a [MenuItem],
    path: &[usize],
    out: &mut Vec<(Vec<usize>, &'a MenuItem)>,
    problems: &mut Vec<Problem>,
) {
    for (index, item) in items.iter().enumerate() {
        let mut at = path.to_vec();
        at.push(index);
        if let MenuItem::Group {
            id,
            children,
            inline_icons: false,
        } = item
        {
            if id.is_empty() {
                problems.push(Problem::EmptyName { path: at.clone() });
            }
            flatten(children, &at, out, problems);
        } else {
            out.push((at, item));
        }
    }
}

//! The commands as building blocks of a workflow (plan 4b): one command, three uses: menu, palette, block in the graph editor.
//!
//! Pattern: Adapter (commands to a description a workflow editor reads) as a pure function. Every registered command is exactly
//! one block; a command that cannot be described (no id, a parameter without a type, a type that cannot be filled) makes the
//! whole export fail with every problem, so no block can silently be missing or half described. The id is the contract: saved
//! workflows refer to it, so renaming one is a migration, never a silent change.

use std::fmt;

use crate::command::{CommandId, TextKey};
use crate::commands::Commands;
use crate::parameter::{Param, ResultType};
use crate::problem::Problem;
use crate::validate::validate_commands;

/// One block: a command as a workflow editor sees it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The stable id of the command.
    pub id: CommandId,
    /// The key of the text of its name.
    pub label: TextKey,
    /// The symbol, if the command has one.
    pub icon: Option<String>,
    /// What it asks for, in order; every parameter has a type.
    pub params: Vec<Param>,
    /// What it gives back.
    pub result: ResultType,
}

impl fmt::Display for Block {
    /// `id(name: type, other?: type) -> result` (`?` marks an optional parameter): one line, stable, for logs and tests.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(", self.id.as_str())?;
        for (index, param) in self.params.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            let ty = param
                .ty
                .as_ref()
                .map_or_else(String::new, crate::parameter::ParamType::name);
            let mark = if param.required { "" } else { "?" };
            write!(f, "{}{mark}: {ty}", param.name)?;
        }
        write!(f, ") -> {}", self.result.name())
    }
}

/// Every command of the list as a block, in the order of the list.
///
/// # Errors
/// All problems of [`validate_commands`] (no id, an id twice, a parameter without a name or a type, ...); nothing is exported then.
pub fn export_blocks(commands: &Commands) -> Result<Vec<Block>, Vec<Problem>> {
    let problems = validate_commands(commands);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(commands
        .iter()
        .map(|c| Block {
            id: c.id.clone(),
            label: c.label.clone(),
            icon: c.icon.clone(),
            params: c.params.clone(),
            result: c.result.clone(),
        })
        .collect())
}

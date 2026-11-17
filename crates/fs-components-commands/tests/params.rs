//! Tests: typed parameters, the result type, and the exporter to workflow blocks (M3, plan 4b).
//!
//! Pattern: Value object tests with fixed expected values (the oracle is written by hand, never computed by the code under test).
//!
//! Mutants (each makes at least one test here red):
//! - `export_blocks` skips validation: `a_command_without_an_id_stops_the_export`, `a_parameter_without_a_type_stops_the_export`;
//! - `export_blocks` drops a command: `every_registered_command_is_one_block`;
//! - `validate_commands` forgets the parameters: `a_parameter_without_a_type_is_reported`, `a_parameter_name_is_needed_and_unique`;
//! - `is_hollow` ignores a choice without options or a list of one: `types_that_cannot_be_filled_are_reported`;
//! - `ParamType::name` wrong for a list: `types_have_stable_names`.

use fs_components_commands::{
    Command, CommandId, Commands, Param, ParamType, Problem, ResultType, export_blocks,
    validate_commands,
};

fn id(name: &str) -> CommandId {
    CommandId::new(name)
}

fn typed() -> Commands {
    Commands::new()
        .with(
            Command::new("group.rename", "group-rename")
                .param(Param::new("name", "param-name").of(ParamType::Text)),
        )
        .with(
            Command::new("group.add-member", "group-add-member")
                .param(Param::new("group", "param-group").of(ParamType::object("group")))
                .param(Param::new("who", "param-who").of(ParamType::Person))
                .param(
                    Param::new("role", "param-role")
                        .of(ParamType::choice(["role-member", "role-admin"]))
                        .optional(),
                )
                .returns(ParamType::Number),
        )
        .with(
            Command::new("file.attach", "file-attach")
                .param(Param::new("files", "param-files").of(ParamType::list(ParamType::File))),
        )
        .with(Command::new("plain", "plain"))
}

#[test]
fn types_have_stable_names() {
    assert_eq!(ParamType::Text.name(), "text");
    assert_eq!(ParamType::Number.name(), "number");
    assert_eq!(ParamType::Person.name(), "person");
    assert_eq!(ParamType::File.name(), "file");
    assert_eq!(ParamType::object("group").name(), "object:group");
    assert_eq!(ParamType::choice(["a", "b"]).name(), "choice(a|b)");
    assert_eq!(ParamType::list(ParamType::File).name(), "list<file>");
    assert_eq!(ResultType::Nothing.name(), "nothing");
    assert_eq!(ResultType::Value(ParamType::Number).name(), "number");
}

#[test]
fn a_typed_list_is_valid() {
    assert_eq!(validate_commands(&typed()), vec![]);
}

#[test]
fn a_parameter_without_a_type_is_reported() {
    let list = Commands::new().with(
        Command::new("a", "a")
            .param(Param::new("x", "x").of(ParamType::Text))
            .param(Param::new("y", "y")),
    );
    assert_eq!(
        validate_commands(&list),
        vec![Problem::ParamWithoutType {
            command: id("a"),
            name: "y".into()
        }]
    );
}

#[test]
fn a_parameter_name_is_needed_and_unique() {
    let list = Commands::new().with(
        Command::new("a", "a")
            .param(Param::new("x", "x").of(ParamType::Text))
            .param(Param::new("x", "x2").of(ParamType::Number))
            .param(Param::new("", "none").of(ParamType::Text)),
    );
    assert_eq!(
        validate_commands(&list),
        vec![
            Problem::ParamName {
                command: id("a"),
                name: "x".into()
            },
            Problem::ParamName {
                command: id("a"),
                name: String::new()
            },
        ]
    );
}

#[test]
fn types_that_cannot_be_filled_are_reported() {
    let list = Commands::new().with(
        Command::new("a", "a")
            .param(Param::new("c", "c").of(ParamType::choice(Vec::<String>::new())))
            .param(Param::new("o", "o").of(ParamType::object("")))
            .param(
                Param::new("l", "l").of(ParamType::list(ParamType::choice(Vec::<String>::new()))),
            )
            .returns(ParamType::object("")),
    );
    let hollow = |name: &str| Problem::ParamHollow {
        command: id("a"),
        name: name.into(),
    };
    assert_eq!(
        validate_commands(&list),
        vec![hollow("c"), hollow("o"), hollow("l"), hollow("result")]
    );
}

#[test]
fn every_registered_command_is_one_block() {
    let blocks = export_blocks(&typed()).unwrap();
    let ids: Vec<&str> = blocks.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(
        ids,
        ["group.rename", "group.add-member", "file.attach", "plain"]
    );
    assert_eq!(blocks.len(), typed().len());
}

#[test]
fn a_block_says_its_parameters_and_result() {
    let lines: Vec<String> = export_blocks(&typed())
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        lines,
        [
            "group.rename(name: text) -> nothing",
            "group.add-member(group: object:group, who: person, role?: choice(role-member|role-admin)) -> number",
            "file.attach(files: list<file>) -> nothing",
            "plain() -> nothing",
        ]
    );
}

#[test]
fn a_command_without_an_id_stops_the_export() {
    let list = typed().with(Command::new("", "nameless"));
    assert_eq!(
        export_blocks(&list).unwrap_err(),
        vec![Problem::EmptyCommandId]
    );
}

#[test]
fn a_parameter_without_a_type_stops_the_export() {
    let list = typed().with(Command::new("late", "late").param(Param::new("p", "p")));
    assert_eq!(
        export_blocks(&list).unwrap_err(),
        vec![Problem::ParamWithoutType {
            command: id("late"),
            name: "p".into()
        }]
    );
}

#[test]
fn the_icon_and_label_are_carried() {
    let list = Commands::new().with(Command::new("edit-copy", "edit-copy-label").icon("edit-copy"));
    let block = &export_blocks(&list).unwrap()[0];
    assert_eq!(block.label.as_str(), "edit-copy-label");
    assert_eq!(block.icon.as_deref(), Some("edit-copy"));
}

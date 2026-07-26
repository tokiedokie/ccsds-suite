use gpui::{App, Div};

use super::property_rows;
use crate::ElementKind;

pub(super) struct CommandMetaDataForm;

impl CommandMetaDataForm {
    pub(super) fn render(
        &self,
        kind: ElementKind,
        metadata: Option<&xtce::CommandMetaDataType>,
        cx: &App,
    ) -> Div {
        let rows = match kind {
            ElementKind::CommandMetaData => {
                let Some(metadata) = metadata else {
                    return property_rows(Vec::new(), cx);
                };
                vec![
                    (
                        "ParameterTypeSet",
                        presence(metadata.parameter_type_set.is_some()),
                    ),
                    ("ParameterSet", presence(metadata.parameter_set.is_some())),
                    (
                        "ArgumentTypeSet",
                        presence(metadata.argument_type_set.is_some()),
                    ),
                    (
                        "MetaCommandSet",
                        presence(metadata.meta_command_set.is_some()),
                    ),
                    (
                        "CommandContainerSet",
                        presence(metadata.command_container_set.is_some()),
                    ),
                    ("StreamSet", presence(metadata.stream_set.is_some())),
                    ("AlgorithmSet", presence(metadata.algorithm_set.is_some())),
                ]
            }
            ElementKind::CommandParameterTypeSet => vec![(
                "Parameter types",
                metadata
                    .and_then(|value| value.parameter_type_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::CommandParameterSet => vec![(
                "Parameters",
                metadata
                    .and_then(|value| value.parameter_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::ArgumentTypeSet => vec![(
                "Argument types",
                metadata
                    .and_then(|value| value.argument_type_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::ArgumentType(index) => {
                let argument_type = metadata
                    .and_then(|value| value.argument_type_set.as_ref())
                    .and_then(|set| set.content.get(index));
                argument_type.map_or_else(Vec::new, |argument_type| {
                    vec![
                        ("Name", argument_type_name(argument_type)),
                        ("Type", argument_type_kind(argument_type).to_owned()),
                    ]
                })
            }
            ElementKind::MetaCommandSet => vec![(
                "Meta commands",
                metadata
                    .and_then(|value| value.meta_command_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::MetaCommand(index) => {
                let command = metadata
                    .and_then(|value| value.meta_command_set.as_ref())
                    .and_then(|set| set.content.get(index));
                command.map_or_else(Vec::new, |command| {
                    vec![
                        ("Name", meta_command_name(command)),
                        ("Type", meta_command_kind(command).to_owned()),
                    ]
                })
            }
            ElementKind::CommandContainerSet => vec![(
                "Command containers",
                metadata
                    .and_then(|value| value.command_container_set.as_ref())
                    .map_or(0, |set| set.command_container.len())
                    .to_string(),
            )],
            ElementKind::CommandStreamSet => vec![(
                "Streams",
                metadata
                    .and_then(|value| value.stream_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::CommandAlgorithmSet => vec![(
                "Algorithms",
                metadata
                    .and_then(|value| value.algorithm_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            _ => Vec::new(),
        };
        property_rows(rows, cx)
    }
}

fn presence(present: bool) -> String {
    if present { "Present" } else { "Not present" }.to_owned()
}

fn argument_type_name(argument_type: &xtce::ArgumentTypeSetTypeContent) -> String {
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => value.name.clone(),
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => value.name.clone(),
    }
}

fn argument_type_kind(argument_type: &xtce::ArgumentTypeSetTypeContent) -> &'static str {
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(_) => "StringArgumentType",
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(_) => "EnumeratedArgumentType",
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(_) => "IntegerArgumentType",
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(_) => "BinaryArgumentType",
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(_) => "FloatArgumentType",
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(_) => "BooleanArgumentType",
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(_) => "RelativeTimeArgumentType",
        xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(_) => "AbsoluteTimeArgumentType",
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(_) => "ArrayArgumentType",
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(_) => "AggregateArgumentType",
    }
}

fn meta_command_name(command: &xtce::MetaCommandSetTypeContent) -> String {
    match command {
        xtce::MetaCommandSetTypeContent::MetaCommand(value) => value.name.clone(),
        xtce::MetaCommandSetTypeContent::MetaCommandRef(value) => value.clone(),
        xtce::MetaCommandSetTypeContent::BlockMetaCommand(value) => value.name.clone(),
    }
}

fn meta_command_kind(command: &xtce::MetaCommandSetTypeContent) -> &'static str {
    match command {
        xtce::MetaCommandSetTypeContent::MetaCommand(_) => "MetaCommand",
        xtce::MetaCommandSetTypeContent::MetaCommandRef(_) => "MetaCommandRef",
        xtce::MetaCommandSetTypeContent::BlockMetaCommand(_) => "BlockMetaCommand",
    }
}

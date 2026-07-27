use gpui::{App, Div, WeakEntity};

use super::{
    collection_summary,
    telemetry_metadata::{algorithm_row, parameter_row, parameter_type_row, stream_row},
};
use crate::{ElementKind, XtceEditor};

pub(super) struct CommandMetaDataForm;

impl CommandMetaDataForm {
    pub(super) fn render(
        &self,
        kind: ElementKind,
        metadata: Option<&xtce::CommandMetaDataType>,
        editor: WeakEntity<XtceEditor>,
        cx: &App,
    ) -> Div {
        match kind {
            ElementKind::CommandMetaData => collection_summary(
                &["Element", "Items"],
                command_sections(metadata),
                vec![
                    None,
                    None,
                    Some(ElementKind::ArgumentTypeSet),
                    Some(ElementKind::MetaCommandSet),
                    Some(ElementKind::CommandContainerSet),
                    Some(ElementKind::CommandStreamSet),
                    Some(ElementKind::CommandAlgorithmSet),
                ],
                editor,
                "Command metadata is not present.",
                cx,
            ),
            ElementKind::CommandParameterTypeSet => {
                let rows = metadata
                    .and_then(|value| value.parameter_type_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(parameter_type_row)
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::CommandParameterType(index)))
                    .collect();
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No command parameter types are defined.",
                    cx,
                )
            }
            ElementKind::CommandParameterSet => {
                let rows = metadata
                    .and_then(|value| value.parameter_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(parameter_row)
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::CommandParameter(index)))
                    .collect();
                collection_summary(
                    &["Name or reference", "Type reference"],
                    rows,
                    targets,
                    editor,
                    "No command parameters are defined.",
                    cx,
                )
            }
            ElementKind::ArgumentTypeSet => {
                let rows = metadata
                    .and_then(|value| value.argument_type_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(argument_type_row)
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::ArgumentType(index)))
                    .collect();
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No argument types are defined.",
                    cx,
                )
            }
            ElementKind::MetaCommandSet => {
                let rows = metadata
                    .and_then(|value| value.meta_command_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(meta_command_row)
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::MetaCommand(index)))
                    .collect();
                collection_summary(
                    &["Type", "Name or reference"],
                    rows,
                    targets,
                    editor,
                    "No meta commands are defined.",
                    cx,
                )
            }
            ElementKind::CommandContainerSet => {
                let rows = metadata
                    .and_then(|value| value.command_container_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.command_container)
                    .map(|container| {
                        vec![
                            container.name.clone(),
                            container.entry_list.content.len().to_string(),
                        ]
                    })
                    .collect::<Vec<_>>();
                let targets = vec![None; rows.len()];
                collection_summary(
                    &["Name", "Entries"],
                    rows,
                    targets,
                    editor,
                    "No command containers are defined.",
                    cx,
                )
            }
            ElementKind::CommandStreamSet => {
                let streams = metadata
                    .and_then(|value| value.stream_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .collect::<Vec<_>>();
                let rows = streams
                    .iter()
                    .map(|stream| stream_row(stream))
                    .collect::<Vec<_>>();
                let targets = streams
                    .iter()
                    .enumerate()
                    .map(|(index, stream)| {
                        matches!(stream, xtce::StreamSetTypeContent::FixedFrameStream(_))
                            .then_some(ElementKind::CommandFixedFrameStream(index))
                    })
                    .collect();
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No command streams are defined.",
                    cx,
                )
            }
            ElementKind::CommandAlgorithmSet => {
                let rows = metadata
                    .and_then(|value| value.algorithm_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(algorithm_row)
                    .collect::<Vec<_>>();
                let targets = vec![None; rows.len()];
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No command algorithms are defined.",
                    cx,
                )
            }
            _ => collection_summary(
                &["Element", "Value"],
                Vec::new(),
                Vec::new(),
                editor,
                "No content.",
                cx,
            ),
        }
    }
}

fn command_sections(metadata: Option<&xtce::CommandMetaDataType>) -> Vec<Vec<String>> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    vec![
        section_row(
            "ParameterTypeSet",
            metadata
                .parameter_type_set
                .as_ref()
                .map_or(0, |set| set.content.len()),
        ),
        section_row(
            "ParameterSet",
            metadata
                .parameter_set
                .as_ref()
                .map_or(0, |set| set.content.len()),
        ),
        section_row(
            "ArgumentTypeSet",
            metadata
                .argument_type_set
                .as_ref()
                .map_or(0, |set| set.content.len()),
        ),
        section_row(
            "MetaCommandSet",
            metadata
                .meta_command_set
                .as_ref()
                .map_or(0, |set| set.content.len()),
        ),
        section_row(
            "CommandContainerSet",
            metadata
                .command_container_set
                .as_ref()
                .map_or(0, |set| set.command_container.len()),
        ),
        section_row(
            "StreamSet",
            metadata
                .stream_set
                .as_ref()
                .map_or(0, |set| set.content.len()),
        ),
        section_row(
            "AlgorithmSet",
            metadata
                .algorithm_set
                .as_ref()
                .map_or(0, |set| set.content.len()),
        ),
    ]
}

fn section_row(name: &str, count: usize) -> Vec<String> {
    vec![name.to_owned(), count.to_string()]
}

fn argument_type_row(argument_type: &xtce::ArgumentTypeSetTypeContent) -> Vec<String> {
    match argument_type {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => {
            vec!["StringArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => {
            vec!["EnumeratedArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => {
            vec!["IntegerArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => {
            vec!["BinaryArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => {
            vec!["FloatArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => {
            vec!["BooleanArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => {
            vec!["RelativeTimeArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => {
            vec!["AbsoluteTimeArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => {
            vec!["ArrayArgumentType".to_owned(), value.name.clone()]
        }
        xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => {
            vec!["AggregateArgumentType".to_owned(), value.name.clone()]
        }
    }
}

fn meta_command_row(command: &xtce::MetaCommandSetTypeContent) -> Vec<String> {
    match command {
        xtce::MetaCommandSetTypeContent::MetaCommand(value) => {
            vec!["MetaCommand".to_owned(), value.name.clone()]
        }
        xtce::MetaCommandSetTypeContent::MetaCommandRef(value) => {
            vec!["MetaCommandRef".to_owned(), value.clone()]
        }
        xtce::MetaCommandSetTypeContent::BlockMetaCommand(value) => {
            vec!["BlockMetaCommand".to_owned(), value.name.clone()]
        }
    }
}

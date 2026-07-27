use gpui::{App, Div, WeakEntity};

use super::collection_summary;
use crate::{ElementKind, XtceEditor};

pub(super) struct TelemetryMetaDataForm;

impl TelemetryMetaDataForm {
    pub(super) fn render(
        &self,
        kind: ElementKind,
        metadata: Option<&xtce::TelemetryMetaDataType>,
        editor: WeakEntity<XtceEditor>,
        cx: &App,
    ) -> Div {
        match kind {
            ElementKind::TelemetryMetaData => collection_summary(
                &["Element", "Items"],
                telemetry_sections(metadata),
                vec![
                    None,
                    None,
                    None,
                    Some(ElementKind::MessageSet),
                    None,
                    Some(ElementKind::TelemetryAlgorithmSet),
                ],
                editor,
                "Telemetry metadata is not present.",
                cx,
            ),
            ElementKind::TelemetryParameterTypeSet => {
                let rows = metadata
                    .and_then(|value| value.parameter_type_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(parameter_type_row)
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::TelemetryParameterType(index)))
                    .collect();
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No parameter types are defined.",
                    cx,
                )
            }
            ElementKind::TelemetryParameterSet => {
                let rows = metadata
                    .and_then(|value| value.parameter_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(parameter_row)
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::TelemetryParameter(index)))
                    .collect();
                collection_summary(
                    &["Name or reference", "Type reference"],
                    rows,
                    targets,
                    editor,
                    "No parameters are defined.",
                    cx,
                )
            }
            ElementKind::ContainerSet => {
                let rows = metadata
                    .and_then(|value| value.container_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(|container| match container {
                        xtce::ContainerSetTypeContent::SequenceContainer(value) => {
                            vec!["SequenceContainer".to_owned(), value.name.clone()]
                        }
                    })
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::SequenceContainer(index)))
                    .collect();
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No sequence containers are defined.",
                    cx,
                )
            }
            ElementKind::MessageSet => {
                let rows = metadata
                    .and_then(|value| value.message_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.message)
                    .map(|message| {
                        vec![
                            message.name.clone(),
                            message.container_ref.container_ref.clone(),
                        ]
                    })
                    .collect::<Vec<_>>();
                let targets = (0..rows.len())
                    .map(|index| Some(ElementKind::Message(index)))
                    .collect();
                collection_summary(
                    &["Name", "Container reference"],
                    rows,
                    targets,
                    editor,
                    "No messages are defined.",
                    cx,
                )
            }
            ElementKind::TelemetryStreamSet => {
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
                    .map(|(index, stream)| match stream {
                        xtce::StreamSetTypeContent::FixedFrameStream(_) => {
                            Some(ElementKind::TelemetryFixedFrameStream(index))
                        }
                        xtce::StreamSetTypeContent::VariableFrameStream(_) => {
                            Some(ElementKind::TelemetryVariableFrameStream(index))
                        }
                        xtce::StreamSetTypeContent::CustomStream(_) => None,
                    })
                    .collect();
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No streams are defined.",
                    cx,
                )
            }
            ElementKind::TelemetryAlgorithmSet => {
                let rows = metadata
                    .and_then(|value| value.algorithm_set.as_ref())
                    .into_iter()
                    .flat_map(|set| &set.content)
                    .map(algorithm_row)
                    .collect::<Vec<_>>();
                let targets = no_targets(rows.len());
                collection_summary(
                    &["Type", "Name"],
                    rows,
                    targets,
                    editor,
                    "No algorithms are defined.",
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

fn no_targets(count: usize) -> Vec<Option<ElementKind>> {
    vec![None; count]
}

fn telemetry_sections(metadata: Option<&xtce::TelemetryMetaDataType>) -> Vec<Vec<String>> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    vec![
        vec![
            "ParameterTypeSet".to_owned(),
            metadata
                .parameter_type_set
                .as_ref()
                .map_or(0, |set| set.content.len())
                .to_string(),
        ],
        vec![
            "ParameterSet".to_owned(),
            metadata
                .parameter_set
                .as_ref()
                .map_or(0, |set| set.content.len())
                .to_string(),
        ],
        vec![
            "ContainerSet".to_owned(),
            metadata
                .container_set
                .as_ref()
                .map_or(0, |set| set.content.len())
                .to_string(),
        ],
        vec![
            "MessageSet".to_owned(),
            metadata
                .message_set
                .as_ref()
                .map_or(0, |set| set.message.len())
                .to_string(),
        ],
        vec![
            "StreamSet".to_owned(),
            metadata
                .stream_set
                .as_ref()
                .map_or(0, |set| set.content.len())
                .to_string(),
        ],
        vec![
            "AlgorithmSet".to_owned(),
            metadata
                .algorithm_set
                .as_ref()
                .map_or(0, |set| set.content.len())
                .to_string(),
        ],
    ]
}

pub(super) fn parameter_type_row(
    parameter_type: &xtce::ParameterTypeSetTypeContent,
) -> Vec<String> {
    match parameter_type {
        xtce::ParameterTypeSetTypeContent::StringParameterType(value) => {
            vec!["StringParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => {
            vec!["EnumeratedParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => {
            vec!["IntegerParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => {
            vec!["BinaryParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => {
            vec!["FloatParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => {
            vec!["BooleanParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
            vec!["RelativeTimeParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
            vec!["AbsoluteTimeParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => {
            vec!["ArrayParameterType".to_owned(), value.name.clone()]
        }
        xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => {
            vec!["AggregateParameterType".to_owned(), value.name.clone()]
        }
    }
}

pub(super) fn parameter_row(parameter: &xtce::ParameterSetTypeContent) -> Vec<String> {
    match parameter {
        xtce::ParameterSetTypeContent::Parameter(value) => {
            vec![value.name.clone(), value.parameter_type_ref.clone()]
        }
        xtce::ParameterSetTypeContent::ParameterRef(value) => {
            vec![format!("→ {}", value.parameter_ref), String::new()]
        }
    }
}

pub(super) fn stream_row(stream: &xtce::StreamSetTypeContent) -> Vec<String> {
    match stream {
        xtce::StreamSetTypeContent::FixedFrameStream(value) => {
            vec!["FixedFrameStream".to_owned(), value.name.clone()]
        }
        xtce::StreamSetTypeContent::VariableFrameStream(value) => {
            vec!["VariableFrameStream".to_owned(), value.name.clone()]
        }
        xtce::StreamSetTypeContent::CustomStream(value) => {
            vec!["CustomStream".to_owned(), value.name.clone()]
        }
    }
}

pub(super) fn algorithm_row(algorithm: &xtce::AlgorithmSetTypeContent) -> Vec<String> {
    match algorithm {
        xtce::AlgorithmSetTypeContent::CustomAlgorithm(value) => {
            vec!["CustomAlgorithm".to_owned(), value.name.clone()]
        }
        xtce::AlgorithmSetTypeContent::MathAlgorithm(value) => {
            vec!["MathAlgorithm".to_owned(), value.name.clone()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parameter_row, telemetry_sections};

    #[test]
    fn metadata_summary_reports_each_collection_size() {
        let metadata = xtce::TelemetryMetaDataType {
            parameter_type_set: Some(xtce::ParameterTypeSetType {
                content: Vec::new(),
            }),
            parameter_set: Some(xtce::ParameterSetType {
                content: vec![xtce::ParameterSetTypeContent::ParameterRef(
                    xtce::ParameterRefType {
                        parameter_ref: "/Shared/Mode".to_owned(),
                    },
                )],
            }),
            container_set: None,
            message_set: None,
            stream_set: None,
            algorithm_set: None,
        };

        let rows = telemetry_sections(Some(&metadata));

        assert_eq!(rows[0], vec!["ParameterTypeSet", "0"]);
        assert_eq!(rows[1], vec!["ParameterSet", "1"]);
    }

    #[test]
    fn parameter_reference_summary_includes_the_reference() {
        let row = parameter_row(&xtce::ParameterSetTypeContent::ParameterRef(
            xtce::ParameterRefType {
                parameter_ref: "/Shared/Counter".to_owned(),
            },
        ));

        assert_eq!(row, vec!["→ /Shared/Counter", ""]);
    }
}

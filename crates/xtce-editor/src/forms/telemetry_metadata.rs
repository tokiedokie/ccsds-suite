use gpui::{App, Div};

use super::property_rows;
use crate::ElementKind;

pub(super) struct TelemetryMetaDataForm;

impl TelemetryMetaDataForm {
    pub(super) fn render(
        &self,
        kind: ElementKind,
        metadata: Option<&xtce::TelemetryMetaDataType>,
        cx: &App,
    ) -> Div {
        let rows = match kind {
            ElementKind::TelemetryMetaData => {
                let Some(metadata) = metadata else {
                    return property_rows(Vec::new(), cx);
                };
                vec![
                    (
                        "ParameterTypeSet",
                        presence(metadata.parameter_type_set.is_some()),
                    ),
                    ("ParameterSet", presence(metadata.parameter_set.is_some())),
                    ("ContainerSet", presence(metadata.container_set.is_some())),
                    ("MessageSet", presence(metadata.message_set.is_some())),
                    ("StreamSet", presence(metadata.stream_set.is_some())),
                    ("AlgorithmSet", presence(metadata.algorithm_set.is_some())),
                ]
            }
            ElementKind::TelemetryParameterTypeSet => vec![(
                "Parameter types",
                metadata
                    .and_then(|value| value.parameter_type_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::TelemetryParameterSet => vec![(
                "Parameters",
                metadata
                    .and_then(|value| value.parameter_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::ContainerSet => vec![(
                "Containers",
                metadata
                    .and_then(|value| value.container_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::MessageSet => vec![(
                "Messages",
                metadata
                    .and_then(|value| value.message_set.as_ref())
                    .map_or(0, |set| set.message.len())
                    .to_string(),
            )],
            ElementKind::TelemetryStreamSet => vec![(
                "Streams",
                metadata
                    .and_then(|value| value.stream_set.as_ref())
                    .map_or(0, |set| set.content.len())
                    .to_string(),
            )],
            ElementKind::TelemetryAlgorithmSet => vec![(
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

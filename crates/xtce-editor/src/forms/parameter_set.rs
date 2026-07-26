use gpui::{App, AppContext, Context, Div, Entity, Window};
use gpui_component::input::InputState;

use super::{field, optional_value};
use crate::XtceEditor;

pub(super) struct ParameterSetForm {
    parameters_input: Entity<InputState>,
}

impl ParameterSetForm {
    pub(super) fn new(
        parameter_set: Option<&xtce::ParameterSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        Self {
            parameters_input: cx.new(|cx| {
                InputState::new(window, cx)
                    .multi_line(true)
                    .default_value(encode(parameter_set))
            }),
        }
    }

    pub(super) fn load(
        &self,
        parameter_set: Option<&xtce::ParameterSetType>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        self.parameters_input.update(cx, |input, cx| {
            input.set_value(encode(parameter_set), window, cx);
        });
    }

    pub(super) fn apply_to(&self, parameter_set: &mut xtce::ParameterSetType, cx: &App) {
        let rows = decode(&self.parameters_input.read(cx).value());
        let mut existing = std::mem::take(&mut parameter_set.content).into_iter();
        parameter_set.content = rows
            .into_iter()
            .map(|row| update_or_create(row, existing.next()))
            .collect();
    }

    pub(super) fn render(&self, cx: &App) -> Div {
        field(
            "Parameters",
            "Parameter | name | type ref | initial value | short description; or ParameterRef | ref",
            &self.parameters_input,
            cx,
        )
    }
}

enum ParameterRow {
    Parameter {
        name: String,
        parameter_type_ref: String,
        initial_value: Option<String>,
        short_description: Option<String>,
    },
    ParameterRef {
        parameter_ref: String,
    },
}

fn encode(parameter_set: Option<&xtce::ParameterSetType>) -> String {
    parameter_set
        .into_iter()
        .flat_map(|set| &set.content)
        .map(|parameter| match parameter {
            xtce::ParameterSetTypeContent::Parameter(parameter) => format!(
                "Parameter | {} | {} | {} | {}",
                parameter.name,
                parameter.parameter_type_ref,
                parameter.initial_value.as_deref().unwrap_or_default(),
                parameter.short_description.as_deref().unwrap_or_default()
            ),
            xtce::ParameterSetTypeContent::ParameterRef(parameter) => {
                format!("ParameterRef | {}", parameter.parameter_ref)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode(value: &str) -> Vec<ParameterRow> {
    value
        .lines()
        .filter_map(|line| {
            let mut columns = line.splitn(5, " | ").map(str::trim);
            match columns.next()? {
                "Parameter" => Some(ParameterRow::Parameter {
                    name: columns.next()?.to_owned(),
                    parameter_type_ref: columns.next()?.to_owned(),
                    initial_value: optional_value(columns.next()?.to_owned()),
                    short_description: optional_value(
                        columns.next().unwrap_or_default().to_owned(),
                    ),
                }),
                "ParameterRef" => Some(ParameterRow::ParameterRef {
                    parameter_ref: columns.next()?.to_owned(),
                }),
                _ => None,
            }
        })
        .collect()
}

fn update_or_create(
    row: ParameterRow,
    existing: Option<xtce::ParameterSetTypeContent>,
) -> xtce::ParameterSetTypeContent {
    match (row, existing) {
        (
            ParameterRow::Parameter {
                name,
                parameter_type_ref,
                initial_value,
                short_description,
            },
            Some(xtce::ParameterSetTypeContent::Parameter(mut parameter)),
        ) => {
            parameter.name = name;
            parameter.parameter_type_ref = parameter_type_ref;
            parameter.initial_value = initial_value;
            parameter.short_description = short_description;
            xtce::ParameterSetTypeContent::Parameter(parameter)
        }
        (
            ParameterRow::Parameter {
                name,
                parameter_type_ref,
                initial_value,
                short_description,
            },
            _,
        ) => xtce::ParameterSetTypeContent::Parameter(xtce::ParameterType {
            short_description,
            name,
            parameter_type_ref,
            initial_value,
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            parameter_properties: None,
        }),
        (
            ParameterRow::ParameterRef { parameter_ref },
            Some(xtce::ParameterSetTypeContent::ParameterRef(mut parameter)),
        ) => {
            parameter.parameter_ref = parameter_ref;
            xtce::ParameterSetTypeContent::ParameterRef(parameter)
        }
        (ParameterRow::ParameterRef { parameter_ref }, _) => {
            xtce::ParameterSetTypeContent::ParameterRef(xtce::ParameterRefType { parameter_ref })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ParameterRow, decode, update_or_create};

    #[test]
    fn decodes_parameters_and_parameter_references() {
        let rows = decode(
            "Parameter | Mode | ModeType | SAFE | Current mode\nParameterRef | /Shared/Counter",
        );

        assert_eq!(rows.len(), 2);
        match &rows[0] {
            ParameterRow::Parameter {
                name,
                parameter_type_ref,
                initial_value,
                ..
            } => {
                assert_eq!(name, "Mode");
                assert_eq!(parameter_type_ref, "ModeType");
                assert_eq!(initial_value.as_deref(), Some("SAFE"));
            }
            ParameterRow::ParameterRef { .. } => panic!("expected a Parameter row"),
        }
        match &rows[1] {
            ParameterRow::ParameterRef { parameter_ref } => {
                assert_eq!(parameter_ref, "/Shared/Counter");
            }
            ParameterRow::Parameter { .. } => panic!("expected a ParameterRef row"),
        }
    }

    #[test]
    fn updating_a_parameter_preserves_fields_not_shown_in_the_form() {
        let existing = xtce::ParameterSetTypeContent::Parameter(xtce::ParameterType {
            short_description: Some("Old description".to_owned()),
            name: "OldName".to_owned(),
            parameter_type_ref: "OldType".to_owned(),
            initial_value: None,
            long_description: Some("Keep this value".to_owned()),
            alias_set: None,
            ancillary_data_set: None,
            parameter_properties: None,
        });
        let updated = update_or_create(
            ParameterRow::Parameter {
                name: "NewName".to_owned(),
                parameter_type_ref: "NewType".to_owned(),
                initial_value: Some("1".to_owned()),
                short_description: Some("New description".to_owned()),
            },
            Some(existing),
        );

        match updated {
            xtce::ParameterSetTypeContent::Parameter(parameter) => {
                assert_eq!(parameter.name, "NewName");
                assert_eq!(
                    parameter.long_description.as_deref(),
                    Some("Keep this value")
                );
            }
            xtce::ParameterSetTypeContent::ParameterRef(_) => {
                panic!("expected a Parameter")
            }
        }
    }
}

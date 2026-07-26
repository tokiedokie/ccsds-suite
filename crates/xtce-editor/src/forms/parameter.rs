use gpui::{App, AppContext, Context, Div, Entity, ParentElement, Styled, Window};
use gpui_component::{
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use super::{field, optional_value};
use crate::XtceEditor;

pub(super) struct ParameterForm {
    name_input: Entity<InputState>,
    parameter_type_ref_input: Entity<InputState>,
    initial_value_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    parameter_ref_input: Entity<InputState>,
}

impl ParameterForm {
    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(gpui::px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn new(
        parameter: Option<&xtce::ParameterSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Self {
        let values = ParameterValues::from_parameter(parameter);
        let name_input = input(&values.name, false, window, cx);
        cx.subscribe(&name_input, |_, _, _: &InputEvent, cx| cx.notify())
            .detach();
        Self {
            name_input,
            parameter_type_ref_input: input(&values.parameter_type_ref, false, window, cx),
            initial_value_input: input(&values.initial_value, false, window, cx),
            short_description_input: input(&values.short_description, false, window, cx),
            long_description_input: input(&values.long_description, true, window, cx),
            parameter_ref_input: input(&values.parameter_ref, false, window, cx),
        }
    }

    pub(super) fn load(
        &self,
        parameter: Option<&xtce::ParameterSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) {
        let values = ParameterValues::from_parameter(parameter);
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.parameter_type_ref_input, values.parameter_type_ref),
            (&self.initial_value_input, values.initial_value),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.parameter_ref_input, values.parameter_ref),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
    }

    pub(super) fn apply_to(&self, parameter: &mut xtce::ParameterSetTypeContent, cx: &App) {
        ParameterValues {
            name: value(&self.name_input, cx),
            parameter_type_ref: value(&self.parameter_type_ref_input, cx),
            initial_value: value(&self.initial_value_input, cx),
            short_description: value(&self.short_description_input, cx),
            long_description: value(&self.long_description_input, cx),
            parameter_ref: value(&self.parameter_ref_input, cx),
        }
        .apply_to(parameter);
    }

    pub(super) fn render(
        &self,
        parameter: Option<&xtce::ParameterSetTypeContent>,
        cx: &App,
    ) -> Div {
        match parameter {
            Some(xtce::ParameterSetTypeContent::Parameter(_)) => v_flex()
                .gap_5()
                .child(h_flex().gap_4().items_start().child(field(
                    "Parameter type reference",
                    "Required",
                    &self.parameter_type_ref_input,
                    cx,
                )))
                .child(
                    h_flex()
                        .gap_4()
                        .items_start()
                        .child(field(
                            "Initial value",
                            "Optional",
                            &self.initial_value_input,
                            cx,
                        ))
                        .child(field(
                            "Short description",
                            "Optional",
                            &self.short_description_input,
                            cx,
                        )),
                )
                .child(field(
                    "Long description",
                    "Optional",
                    &self.long_description_input,
                    cx,
                )),
            Some(xtce::ParameterSetTypeContent::ParameterRef(_)) => field(
                "Parameter reference",
                "Required",
                &self.parameter_ref_input,
                cx,
            ),
            None => v_flex(),
        }
    }
}

struct ParameterValues {
    name: String,
    parameter_type_ref: String,
    initial_value: String,
    short_description: String,
    long_description: String,
    parameter_ref: String,
}

impl ParameterValues {
    fn from_parameter(parameter: Option<&xtce::ParameterSetTypeContent>) -> Self {
        match parameter {
            Some(xtce::ParameterSetTypeContent::Parameter(parameter)) => Self {
                name: parameter.name.clone(),
                parameter_type_ref: parameter.parameter_type_ref.clone(),
                initial_value: parameter.initial_value.clone().unwrap_or_default(),
                short_description: parameter.short_description.clone().unwrap_or_default(),
                long_description: parameter.long_description.clone().unwrap_or_default(),
                parameter_ref: String::new(),
            },
            Some(xtce::ParameterSetTypeContent::ParameterRef(parameter)) => Self {
                name: String::new(),
                parameter_type_ref: String::new(),
                initial_value: String::new(),
                short_description: String::new(),
                long_description: String::new(),
                parameter_ref: parameter.parameter_ref.clone(),
            },
            None => Self {
                name: String::new(),
                parameter_type_ref: String::new(),
                initial_value: String::new(),
                short_description: String::new(),
                long_description: String::new(),
                parameter_ref: String::new(),
            },
        }
    }

    fn apply_to(&self, parameter: &mut xtce::ParameterSetTypeContent) {
        match parameter {
            xtce::ParameterSetTypeContent::Parameter(parameter) => {
                parameter.name.clone_from(&self.name);
                parameter
                    .parameter_type_ref
                    .clone_from(&self.parameter_type_ref);
                parameter.initial_value = optional_value(self.initial_value.clone());
                parameter.short_description = optional_value(self.short_description.clone());
                parameter.long_description = optional_value(self.long_description.clone());
            }
            xtce::ParameterSetTypeContent::ParameterRef(parameter) => {
                parameter.parameter_ref.clone_from(&self.parameter_ref);
            }
        }
    }
}

fn input(
    value: &str,
    multi_line: bool,
    window: &mut Window,
    cx: &mut Context<XtceEditor>,
) -> Entity<InputState> {
    cx.new(|cx| {
        InputState::new(window, cx)
            .multi_line(multi_line)
            .default_value(value.to_owned())
    })
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::ParameterValues;

    #[test]
    fn applying_form_values_preserves_nested_parameter_metadata() {
        let mut parameter = xtce::ParameterSetTypeContent::Parameter(xtce::ParameterType {
            short_description: None,
            name: "OldName".to_owned(),
            parameter_type_ref: "OldType".to_owned(),
            initial_value: None,
            long_description: None,
            alias_set: Some(xtce::AliasSetType {
                alias: vec![xtce::AliasType {
                    name_space: "operations".to_owned(),
                    alias: "MODE".to_owned(),
                }],
            }),
            ancillary_data_set: None,
            parameter_properties: None,
        });
        ParameterValues {
            name: "Mode".to_owned(),
            parameter_type_ref: "ModeType".to_owned(),
            initial_value: "SAFE".to_owned(),
            short_description: "Current mode".to_owned(),
            long_description: "Current operational mode".to_owned(),
            parameter_ref: String::new(),
        }
        .apply_to(&mut parameter);

        match parameter {
            xtce::ParameterSetTypeContent::Parameter(parameter) => {
                assert_eq!(parameter.name, "Mode");
                assert_eq!(parameter.initial_value.as_deref(), Some("SAFE"));
                assert_eq!(
                    parameter.alias_set.expect("alias set").alias[0].alias,
                    "MODE"
                );
            }
            xtce::ParameterSetTypeContent::ParameterRef(_) => panic!("expected a Parameter"),
        }
    }
}

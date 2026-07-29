use gpui::{
    App, AppContext, Context, Div, Entity, ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme, StyledExt, h_flex,
    input::{InputEvent, InputState},
    v_flex,
};

use super::{
    alias_set::AliasSetForm,
    ancillary_data_set::AncillaryDataSetForm,
    field, optional_value,
    rpn_operation::{RpnOperationEntry, RpnOperationForm},
};
use crate::XtceEditor;

pub(super) struct MathAlgorithmForm {
    present: bool,
    name_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    operation_name_input: Entity<InputState>,
    operation_short_description_input: Entity<InputState>,
    output_parameter_ref_input: Entity<InputState>,
    operation_entries: Entity<RpnOperationForm>,
    trigger_set_name_input: Entity<InputState>,
    trigger_rate_input: Entity<InputState>,
    triggers_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    operation_ancillary_data_set: AncillaryDataSetForm,
    _subscriptions: Vec<Subscription>,
}

impl MathAlgorithmForm {
    pub(super) fn new(
        algorithm: Option<&xtce::AlgorithmSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let algorithm = math_algorithm(algorithm);
        let values = AlgorithmValues::from_algorithm(algorithm);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| Self {
            present: algorithm.is_some(),
            name_input,
            short_description_input: input(&values.short_description, false, window, cx),
            long_description_input: input(&values.long_description, true, window, cx),
            operation_name_input: input(&values.operation_name, false, window, cx),
            operation_short_description_input: input(
                &values.operation_short_description,
                false,
                window,
                cx,
            ),
            output_parameter_ref_input: input(&values.output_parameter_ref, false, window, cx),
            operation_entries: RpnOperationForm::new(values.operation_entries, window, cx),
            trigger_set_name_input: input(&values.trigger_set_name, false, window, cx),
            trigger_rate_input: input(&values.trigger_rate, false, window, cx),
            triggers_input: input(&values.triggers, true, window, cx),
            alias_set: AliasSetForm::new(
                algorithm.and_then(|algorithm| algorithm.alias_set.as_ref()),
                window,
                cx,
            ),
            ancillary_data_set: AncillaryDataSetForm::new(
                algorithm.and_then(|algorithm| algorithm.ancillary_data_set.as_ref()),
                window,
                cx,
            ),
            operation_ancillary_data_set: AncillaryDataSetForm::new(
                algorithm.and_then(operation_ancillary_data_set),
                window,
                cx,
            ),
            _subscriptions: vec![name_subscription],
        })
    }

    pub(super) fn load(
        &mut self,
        algorithm: Option<&xtce::AlgorithmSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let algorithm = math_algorithm(algorithm);
        let values = AlgorithmValues::from_algorithm(algorithm);
        self.present = algorithm.is_some();
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.operation_name_input, values.operation_name),
            (
                &self.operation_short_description_input,
                values.operation_short_description,
            ),
            (
                &self.output_parameter_ref_input,
                values.output_parameter_ref,
            ),
            (&self.trigger_set_name_input, values.trigger_set_name),
            (&self.trigger_rate_input, values.trigger_rate),
            (&self.triggers_input, values.triggers),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        self.operation_entries.update(cx, |form, cx| {
            form.load(values.operation_entries, window, cx);
        });
        self.alias_set.load(
            algorithm.and_then(|algorithm| algorithm.alias_set.as_ref()),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            algorithm.and_then(|algorithm| algorithm.ancillary_data_set.as_ref()),
            window,
            cx,
        );
        self.operation_ancillary_data_set.load(
            algorithm.and_then(operation_ancillary_data_set),
            window,
            cx,
        );
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn render_name_editor(&self, cx: &App) -> Div {
        super::name_editor(&self.name_input, cx)
    }

    pub(super) fn apply_to(&self, algorithm: &mut xtce::AlgorithmSetTypeContent, cx: &App) {
        let xtce::AlgorithmSetTypeContent::MathAlgorithm(algorithm) = algorithm else {
            return;
        };
        algorithm.name = value(&self.name_input, cx);
        algorithm.short_description = optional_value(value(&self.short_description_input, cx));
        algorithm.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut algorithm.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut algorithm.ancillary_data_set, cx);

        let operation = &mut algorithm.math_operation;
        operation.name = optional_value(value(&self.operation_name_input, cx));
        operation.short_description =
            optional_value(value(&self.operation_short_description_input, cx));
        operation.output_parameter_ref = value(&self.output_parameter_ref_input, cx);
        let mut ancillary_data_set = take_operation_ancillary_data_set(&mut operation.content);
        self.operation_ancillary_data_set
            .apply_to_option(&mut ancillary_data_set, cx);
        let entries = self
            .operation_entries
            .read(cx)
            .entries(cx)
            .into_iter()
            .map(triggered_math_content)
            .collect();
        let trigger_set = decode_triggers(
            &value(&self.trigger_set_name_input, cx),
            &value(&self.trigger_rate_input, cx),
            &value(&self.triggers_input, cx),
        );
        operation.content = operation_content(ancillary_data_set, entries, trigger_set);
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        if !self.present {
            return v_flex().child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("The selected MathAlgorithm is not present."),
            );
        }
        v_flex()
            .w_full()
            .gap_5()
            .child(field(
                "Short description",
                "Optional",
                &self.short_description_input,
                cx,
            ))
            .child(field(
                "Long description",
                "Optional",
                &self.long_description_input,
                cx,
            ))
            .child(
                v_flex()
                    .w_full()
                    .gap_4()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().muted.opacity(0.18))
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().text_lg().font_semibold().child("Math operation"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        "RPN expression and output defined by this MathOperation.",
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Operation name",
                                "Optional",
                                &self.operation_name_input,
                                cx,
                            ))
                            .child(field(
                                "Output parameter reference",
                                "Required",
                                &self.output_parameter_ref_input,
                                cx,
                            )),
                    )
                    .child(field(
                        "Operation short description",
                        "Optional",
                        &self.operation_short_description_input,
                        cx,
                    ))
                    .child(self.operation_entries.clone())
                    .child(
                        v_flex()
                            .gap_3()
                            .child(div().text_sm().font_medium().child("Operation metadata"))
                            .child(self.operation_ancillary_data_set.render(cx)),
                    ),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(div().text_sm().font_medium().child("Trigger set"))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field("Name", "Optional", &self.trigger_set_name_input, cx))
                            .child(field(
                                "Trigger rate",
                                "Optional; defaults to 1",
                                &self.trigger_rate_input,
                                cx,
                            )),
                    )
                    .child(field(
                        "Triggers",
                        "One per line: parameter | ref, container | ref, or periodic | seconds",
                        &self.triggers_input,
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(div().text_sm().font_medium().child("Algorithm metadata"))
                    .child(self.alias_set.render(cx))
                    .child(self.ancillary_data_set.render(cx)),
            )
    }
}

impl Render for MathAlgorithmForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_form(cx)
    }
}

struct AlgorithmValues {
    name: String,
    short_description: String,
    long_description: String,
    operation_name: String,
    operation_short_description: String,
    output_parameter_ref: String,
    operation_entries: Vec<RpnOperationEntry>,
    trigger_set_name: String,
    trigger_rate: String,
    triggers: String,
}

impl AlgorithmValues {
    fn from_algorithm(algorithm: Option<&xtce::MathAlgorithmType>) -> Self {
        let operation = algorithm.map(|algorithm| &algorithm.math_operation);
        let trigger_set = operation.and_then(operation_trigger_set);
        Self {
            name: algorithm
                .map(|algorithm| algorithm.name.clone())
                .unwrap_or_default(),
            short_description: algorithm
                .and_then(|algorithm| algorithm.short_description.clone())
                .unwrap_or_default(),
            long_description: algorithm
                .and_then(|algorithm| algorithm.long_description.clone())
                .unwrap_or_default(),
            operation_name: operation
                .and_then(|operation| operation.name.clone())
                .unwrap_or_default(),
            operation_short_description: operation
                .and_then(|operation| operation.short_description.clone())
                .unwrap_or_default(),
            output_parameter_ref: operation
                .map(|operation| operation.output_parameter_ref.clone())
                .unwrap_or_default(),
            operation_entries: rpn_entries_from_operation(operation),
            trigger_set_name: trigger_set
                .and_then(|set| set.name.clone())
                .unwrap_or_default(),
            trigger_rate: trigger_set
                .map(|set| set.trigger_rate.to_string())
                .unwrap_or_else(|| xtce::TriggerSetType::default_trigger_rate().to_string()),
            triggers: encode_triggers(trigger_set),
        }
    }
}

pub(crate) fn default_math_algorithm(name: String) -> xtce::AlgorithmSetTypeContent {
    xtce::AlgorithmSetTypeContent::MathAlgorithm(xtce::MathAlgorithmType {
        short_description: None,
        name,
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        math_operation: xtce::TriggeredMathOperationType {
            name: None,
            short_description: None,
            output_parameter_ref: String::new(),
            content: vec![xtce::TriggeredMathOperationTypeContent::ValueOperand(
                "0".to_owned(),
            )],
        },
    })
}

fn math_algorithm(
    algorithm: Option<&xtce::AlgorithmSetTypeContent>,
) -> Option<&xtce::MathAlgorithmType> {
    match algorithm {
        Some(xtce::AlgorithmSetTypeContent::MathAlgorithm(algorithm)) => Some(algorithm),
        _ => None,
    }
}

fn operation_ancillary_data_set(
    algorithm: &xtce::MathAlgorithmType,
) -> Option<&xtce::AncillaryDataSetType> {
    algorithm
        .math_operation
        .content
        .iter()
        .find_map(|content| match content {
            xtce::TriggeredMathOperationTypeContent::AncillaryDataSet(value) => Some(value),
            _ => None,
        })
}

fn take_operation_ancillary_data_set(
    content: &mut Vec<xtce::TriggeredMathOperationTypeContent>,
) -> Option<xtce::AncillaryDataSetType> {
    std::mem::take(content)
        .into_iter()
        .find_map(|content| match content {
            xtce::TriggeredMathOperationTypeContent::AncillaryDataSet(value) => Some(value),
            _ => None,
        })
}

fn operation_trigger_set(
    operation: &xtce::TriggeredMathOperationType,
) -> Option<&xtce::TriggerSetType> {
    operation.content.iter().find_map(|content| match content {
        xtce::TriggeredMathOperationTypeContent::TriggerSet(value) => Some(value),
        _ => None,
    })
}

fn rpn_entries_from_operation(
    operation: Option<&xtce::TriggeredMathOperationType>,
) -> Vec<RpnOperationEntry> {
    operation
        .into_iter()
        .flat_map(|operation| &operation.content)
        .filter_map(|entry| match entry {
            xtce::TriggeredMathOperationTypeContent::ValueOperand(value) => {
                Some(RpnOperationEntry::Value(value.clone()))
            }
            xtce::TriggeredMathOperationTypeContent::ThisParameterOperand(value) => {
                Some(RpnOperationEntry::ThisParameter(value.clone()))
            }
            xtce::TriggeredMathOperationTypeContent::Operator(value) => {
                Some(RpnOperationEntry::Operator(value.clone()))
            }
            xtce::TriggeredMathOperationTypeContent::ParameterInstanceRefOperand(value) => {
                Some(RpnOperationEntry::ParameterInstance {
                    parameter_ref: value.parameter_ref.clone(),
                    instance: value.instance,
                    use_calibrated_value: value.use_calibrated_value,
                })
            }
            xtce::TriggeredMathOperationTypeContent::AncillaryDataSet(_)
            | xtce::TriggeredMathOperationTypeContent::TriggerSet(_) => None,
        })
        .collect()
}

fn triggered_math_content(entry: RpnOperationEntry) -> xtce::TriggeredMathOperationTypeContent {
    match entry {
        RpnOperationEntry::Value(value) => {
            xtce::TriggeredMathOperationTypeContent::ValueOperand(value)
        }
        RpnOperationEntry::ThisParameter(value) => {
            xtce::TriggeredMathOperationTypeContent::ThisParameterOperand(value)
        }
        RpnOperationEntry::Operator(value) => {
            xtce::TriggeredMathOperationTypeContent::Operator(value)
        }
        RpnOperationEntry::ParameterInstance {
            parameter_ref,
            instance,
            use_calibrated_value,
        } => xtce::TriggeredMathOperationTypeContent::ParameterInstanceRefOperand(
            xtce::ParameterInstanceRefType {
                parameter_ref,
                instance,
                use_calibrated_value,
            },
        ),
        RpnOperationEntry::ArgumentInstance { .. } => {
            unreachable!("argument operands are not enabled for math algorithms")
        }
    }
}

fn operation_content(
    ancillary_data_set: Option<xtce::AncillaryDataSetType>,
    mut entries: Vec<xtce::TriggeredMathOperationTypeContent>,
    trigger_set: Option<xtce::TriggerSetType>,
) -> Vec<xtce::TriggeredMathOperationTypeContent> {
    let mut content = Vec::new();
    if let Some(ancillary_data_set) = ancillary_data_set {
        content.push(xtce::TriggeredMathOperationTypeContent::AncillaryDataSet(
            ancillary_data_set,
        ));
    }
    if entries.is_empty() {
        entries.push(xtce::TriggeredMathOperationTypeContent::ValueOperand(
            "0".to_owned(),
        ));
    }
    content.append(&mut entries);
    if let Some(trigger_set) = trigger_set {
        content.push(xtce::TriggeredMathOperationTypeContent::TriggerSet(
            trigger_set,
        ));
    }
    content
}

fn encode_triggers(set: Option<&xtce::TriggerSetType>) -> String {
    set.into_iter()
        .flat_map(|set| &set.content)
        .map(|trigger| match trigger {
            xtce::TriggerSetTypeContent::OnParameterUpdateTrigger(trigger) => {
                format!("parameter | {}", trigger.parameter_ref)
            }
            xtce::TriggerSetTypeContent::OnContainerUpdateTrigger(trigger) => {
                format!("container | {}", trigger.container_ref)
            }
            xtce::TriggerSetTypeContent::OnPeriodicRateTrigger(trigger) => {
                format!("periodic | {}", trigger.fire_rate_in_seconds)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_triggers(name: &str, rate: &str, value: &str) -> Option<xtce::TriggerSetType> {
    let content = value
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(2, '|').map(str::trim);
            let kind = fields.next()?;
            let value = fields.next()?;
            match kind {
                "parameter" if !value.is_empty() => {
                    Some(xtce::TriggerSetTypeContent::OnParameterUpdateTrigger(
                        xtce::OnParameterUpdateTriggerType {
                            parameter_ref: value.to_owned(),
                        },
                    ))
                }
                "container" if !value.is_empty() => {
                    Some(xtce::TriggerSetTypeContent::OnContainerUpdateTrigger(
                        xtce::OnContainerUpdateTriggerType {
                            container_ref: value.to_owned(),
                        },
                    ))
                }
                "periodic" => value.parse().ok().map(|fire_rate_in_seconds| {
                    xtce::TriggerSetTypeContent::OnPeriodicRateTrigger(
                        xtce::OnPeriodicRateTriggerType {
                            fire_rate_in_seconds,
                        },
                    )
                }),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    (!content.is_empty()).then(|| xtce::TriggerSetType {
        name: optional_value(name.trim().to_owned()),
        trigger_rate: rate
            .trim()
            .parse()
            .unwrap_or_else(|_| xtce::TriggerSetType::default_trigger_rate()),
        content,
    })
}

fn input(
    value: &str,
    multiline: bool,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<InputState> {
    cx.new(|cx| {
        let input = InputState::new(window, cx).default_value(value.to_owned());
        if multiline {
            input.auto_grow(super::MULTILINE_MIN_ROWS, super::MULTILINE_MAX_ROWS)
        } else {
            input
        }
    })
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        decode_triggers, default_math_algorithm, encode_triggers, rpn_entries_from_operation,
        triggered_math_content,
    };
    use crate::forms::rpn_operation::RpnOperationEntry;

    #[test]
    fn math_operation_entries_round_trip_in_rpn_order() {
        let entries = vec![
            RpnOperationEntry::ParameterInstance {
                parameter_ref: "P1".to_owned(),
                instance: 1,
                use_calibrated_value: false,
            },
            RpnOperationEntry::Value("2".to_owned()),
            RpnOperationEntry::Operator("*".to_owned()),
        ];
        let operation = xtce::TriggeredMathOperationType {
            name: None,
            short_description: None,
            output_parameter_ref: "OUT".to_owned(),
            content: entries
                .clone()
                .into_iter()
                .map(triggered_math_content)
                .collect(),
        };
        assert_eq!(rpn_entries_from_operation(Some(&operation)), entries);
    }

    #[test]
    fn math_operation_triggers_round_trip() {
        let triggers = decode_triggers("main", "2", "container | C1\nperiodic | 0.5").unwrap();
        assert_eq!(
            encode_triggers(Some(&triggers)),
            "container | C1\nperiodic | 0.5"
        );
        assert_eq!(triggers.trigger_rate, 2);
    }

    #[test]
    fn new_math_algorithm_has_a_valid_initial_operand() {
        let xtce::AlgorithmSetTypeContent::MathAlgorithm(algorithm) =
            default_math_algorithm("MathAlgorithm1".to_owned())
        else {
            panic!("expected math algorithm");
        };
        assert_eq!(algorithm.name, "MathAlgorithm1");
        assert!(matches!(
            algorithm.math_operation.content.as_slice(),
            [xtce::TriggeredMathOperationTypeContent::ValueOperand(value)] if value == "0"
        ));
    }
}

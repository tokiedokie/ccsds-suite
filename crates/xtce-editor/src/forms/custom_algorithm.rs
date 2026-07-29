use gpui::{
    App, AppContext, Context, Div, Entity, ParentElement, Render, Styled, Subscription, Window,
    div, px,
};
use gpui_component::{
    ActiveTheme, IndexPath, StyledExt, h_flex,
    input::{Input, InputEvent, InputState},
    select::SelectState,
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm, field, impl_select_item,
    optional_value,
};
use crate::XtceEditor;

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

pub(super) struct CustomAlgorithmForm {
    present: bool,
    name_input: Entity<InputState>,
    thread_select: Entity<SelectState<Vec<BooleanChoice>>>,
    trigger_container_input: Entity<InputState>,
    priority_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    language_input: Entity<InputState>,
    algorithm_text_input: Entity<InputState>,
    external_algorithms_input: Entity<InputState>,
    inputs_input: Entity<InputState>,
    outputs_input: Entity<InputState>,
    trigger_set_name_input: Entity<InputState>,
    trigger_rate_input: Entity<InputState>,
    triggers_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    _subscriptions: Vec<Subscription>,
}

impl CustomAlgorithmForm {
    pub(super) fn new(
        algorithm: Option<&xtce::AlgorithmSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let algorithm = custom_algorithm(algorithm);
        let values = AlgorithmValues::from_algorithm(algorithm);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| Self {
            present: algorithm.is_some(),
            name_input,
            thread_select: select(BooleanChoice::VARIANTS, values.thread, window, cx),
            trigger_container_input: input(&values.trigger_container, false, window, cx),
            priority_input: input(&values.priority, false, window, cx),
            short_description_input: input(&values.short_description, false, window, cx),
            long_description_input: input(&values.long_description, true, window, cx),
            language_input: input(&values.language, false, window, cx),
            algorithm_text_input: input(&values.algorithm_text, true, window, cx),
            external_algorithms_input: input(&values.external_algorithms, true, window, cx),
            inputs_input: input(&values.inputs, true, window, cx),
            outputs_input: input(&values.outputs, true, window, cx),
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
            _subscriptions: vec![name_subscription],
        })
    }

    pub(super) fn load(
        &mut self,
        algorithm: Option<&xtce::AlgorithmSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let algorithm = custom_algorithm(algorithm);
        let values = AlgorithmValues::from_algorithm(algorithm);
        self.present = algorithm.is_some();
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.trigger_container_input, values.trigger_container),
            (&self.priority_input, values.priority),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.language_input, values.language),
            (&self.algorithm_text_input, values.algorithm_text),
            (&self.external_algorithms_input, values.external_algorithms),
            (&self.inputs_input, values.inputs),
            (&self.outputs_input, values.outputs),
            (&self.trigger_set_name_input, values.trigger_set_name),
            (&self.trigger_rate_input, values.trigger_rate),
            (&self.triggers_input, values.triggers),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        sync_select(&self.thread_select, values.thread, window, cx);
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
        cx.notify();
    }

    pub(super) fn name(&self, cx: &App) -> String {
        value(&self.name_input, cx)
    }

    pub(super) fn render_name_editor(&self) -> Div {
        v_flex()
            .w_full()
            .max_w(px(520.))
            .child(Input::new(&self.name_input))
    }

    pub(super) fn apply_to(&self, algorithm: &mut xtce::AlgorithmSetTypeContent, cx: &App) {
        let xtce::AlgorithmSetTypeContent::CustomAlgorithm(algorithm) = algorithm else {
            return;
        };
        algorithm.name = value(&self.name_input, cx);
        algorithm.thread =
            selected_value(&self.thread_select, BooleanChoice::False, cx) == BooleanChoice::True;
        algorithm.trigger_container = optional_value(value(&self.trigger_container_input, cx));
        algorithm.priority = optional_parse(&value(&self.priority_input, cx));
        algorithm.short_description = optional_value(value(&self.short_description_input, cx));
        algorithm.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut algorithm.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut algorithm.ancillary_data_set, cx);
        algorithm.algorithm_text = decode_algorithm_text(
            value(&self.language_input, cx),
            value(&self.algorithm_text_input, cx),
        );
        algorithm.external_algorithm_set =
            decode_external_algorithms(&value(&self.external_algorithms_input, cx));
        algorithm.input_set = decode_inputs(&value(&self.inputs_input, cx));
        algorithm.output_set = decode_outputs(&value(&self.outputs_input, cx));
        algorithm.trigger_set = decode_triggers(
            &value(&self.trigger_set_name_input, cx),
            &value(&self.trigger_rate_input, cx),
            &value(&self.triggers_input, cx),
        );
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        if !self.present {
            return v_flex().child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("The selected CustomAlgorithm is not present."),
            );
        }
        v_flex()
            .w_full()
            .gap_5()
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(select_field("Thread", "Required", &self.thread_select))
                    .child(field(
                        "Trigger container",
                        "Optional",
                        &self.trigger_container_input,
                        cx,
                    ))
                    .child(field(
                        "Priority",
                        "Optional integer",
                        &self.priority_input,
                        cx,
                    )),
            )
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
                    .gap_3()
                    .child(div().text_sm().font_medium().child("Algorithm"))
                    .child(field(
                        "Language",
                        "Optional; defaults to pseudo",
                        &self.language_input,
                        cx,
                    ))
                    .child(field(
                        "Algorithm text",
                        "Optional",
                        &self.algorithm_text_input,
                        cx,
                    )),
            )
            .child(field(
                "External algorithms",
                "One per line: implementation name | algorithm location",
                &self.external_algorithms_input,
                cx,
            ))
            .child(field(
                "Inputs",
                "One per line: parameter | ref | instance | calibrated | input name, or constant | name | value",
                &self.inputs_input,
                cx,
            ))
            .child(field(
                "Outputs",
                "One per line: parameter ref | output name",
                &self.outputs_input,
                cx,
            ))
            .child(
                v_flex()
                    .gap_3()
                    .child(div().text_sm().font_medium().child("Trigger set"))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Name",
                                "Optional",
                                &self.trigger_set_name_input,
                                cx,
                            ))
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
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
    }
}

impl Render for CustomAlgorithmForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_form(cx)
    }
}

struct AlgorithmValues {
    name: String,
    thread: BooleanChoice,
    trigger_container: String,
    priority: String,
    short_description: String,
    long_description: String,
    language: String,
    algorithm_text: String,
    external_algorithms: String,
    inputs: String,
    outputs: String,
    trigger_set_name: String,
    trigger_rate: String,
    triggers: String,
}

impl AlgorithmValues {
    fn from_algorithm(algorithm: Option<&xtce::InputOutputTriggerAlgorithmType>) -> Self {
        let trigger_set = algorithm.and_then(|algorithm| algorithm.trigger_set.as_ref());
        Self {
            name: algorithm
                .map(|algorithm| algorithm.name.clone())
                .unwrap_or_default(),
            thread: if algorithm.is_some_and(|algorithm| algorithm.thread) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            trigger_container: algorithm
                .and_then(|algorithm| algorithm.trigger_container.clone())
                .unwrap_or_default(),
            priority: algorithm
                .and_then(|algorithm| algorithm.priority)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            short_description: algorithm
                .and_then(|algorithm| algorithm.short_description.clone())
                .unwrap_or_default(),
            long_description: algorithm
                .and_then(|algorithm| algorithm.long_description.clone())
                .unwrap_or_default(),
            language: algorithm
                .and_then(|algorithm| algorithm.algorithm_text.as_ref())
                .map(|text| text.language.clone())
                .unwrap_or_else(xtce::AlgorithmTextType::default_language),
            algorithm_text: algorithm
                .and_then(|algorithm| algorithm.algorithm_text.as_ref())
                .map(|text| text.content.clone())
                .unwrap_or_default(),
            external_algorithms: encode_external_algorithms(
                algorithm.and_then(|algorithm| algorithm.external_algorithm_set.as_ref()),
            ),
            inputs: encode_inputs(algorithm.and_then(|algorithm| algorithm.input_set.as_ref())),
            outputs: encode_outputs(algorithm.and_then(|algorithm| algorithm.output_set.as_ref())),
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

pub(crate) fn default_custom_algorithm(name: String) -> xtce::AlgorithmSetTypeContent {
    xtce::AlgorithmSetTypeContent::CustomAlgorithm(xtce::InputOutputTriggerAlgorithmType {
        short_description: None,
        name,
        thread: xtce::InputOutputTriggerAlgorithmType::default_thread(),
        trigger_container: None,
        priority: None,
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        algorithm_text: None,
        external_algorithm_set: None,
        input_set: None,
        output_set: None,
        trigger_set: None,
    })
}

fn custom_algorithm(
    algorithm: Option<&xtce::AlgorithmSetTypeContent>,
) -> Option<&xtce::InputOutputTriggerAlgorithmType> {
    match algorithm {
        Some(xtce::AlgorithmSetTypeContent::CustomAlgorithm(algorithm)) => Some(algorithm),
        _ => None,
    }
}

pub(super) fn decode_algorithm_text(
    language: String,
    content: String,
) -> Option<xtce::AlgorithmTextType> {
    (!content.is_empty()).then(|| xtce::AlgorithmTextType {
        language: if language.is_empty() {
            xtce::AlgorithmTextType::default_language()
        } else {
            language
        },
        content,
    })
}

pub(super) fn encode_external_algorithms(set: Option<&xtce::ExternalAlgorithmSetType>) -> String {
    set.into_iter()
        .flat_map(|set| &set.external_algorithm)
        .map(|algorithm| {
            format!(
                "{} | {}",
                algorithm.implementation_name, algorithm.algorithm_location
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn decode_external_algorithms(value: &str) -> Option<xtce::ExternalAlgorithmSetType> {
    let external_algorithm = value
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(2, '|').map(str::trim);
            let implementation_name = fields.next()?.to_owned();
            let algorithm_location = fields.next()?.to_owned();
            (!implementation_name.is_empty() && !algorithm_location.is_empty()).then_some(
                xtce::ExternalAlgorithmType {
                    implementation_name,
                    algorithm_location,
                },
            )
        })
        .collect::<Vec<_>>();
    (!external_algorithm.is_empty())
        .then_some(xtce::ExternalAlgorithmSetType { external_algorithm })
}

pub(super) fn encode_inputs(set: Option<&xtce::InputSetType>) -> String {
    set.into_iter()
        .flat_map(|set| &set.content)
        .map(|input| match input {
            xtce::InputSetTypeContent::InputParameterInstanceRef(input) => format!(
                "parameter | {} | {} | {} | {}",
                input.parameter_ref,
                input.instance,
                input.use_calibrated_value,
                input.input_name.as_deref().unwrap_or_default()
            ),
            xtce::InputSetTypeContent::Constant(input) => {
                format!("constant | {} | {}", input.constant_name, input.value)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn decode_inputs(value: &str) -> Option<xtce::InputSetType> {
    let content = value
        .lines()
        .filter_map(|line| {
            let fields = line.split('|').map(str::trim).collect::<Vec<_>>();
            match fields.first().copied()? {
                "parameter" if fields.len() >= 2 && !fields[1].is_empty() => {
                    Some(xtce::InputSetTypeContent::InputParameterInstanceRef(
                        xtce::InputParameterInstanceRefType {
                            parameter_ref: fields[1].to_owned(),
                            instance: fields
                                .get(2)
                                .and_then(|value| value.parse().ok())
                                .unwrap_or_else(
                                    xtce::InputParameterInstanceRefType::default_instance,
                                ),
                            use_calibrated_value: fields
                                .get(3)
                                .and_then(|value| value.parse().ok())
                                .unwrap_or_else(
                                    xtce::InputParameterInstanceRefType::default_use_calibrated_value,
                                ),
                            input_name: fields
                                .get(4)
                                .map(|value| (*value).to_owned())
                                .filter(|value| !value.is_empty()),
                        },
                    ))
                }
                "constant" if fields.len() >= 3 && !fields[1].is_empty() => {
                    Some(xtce::InputSetTypeContent::Constant(xtce::ConstantType {
                        constant_name: fields[1].to_owned(),
                        value: fields[2].to_owned(),
                    }))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    (!content.is_empty()).then_some(xtce::InputSetType { content })
}

fn encode_outputs(set: Option<&xtce::OutputSetType>) -> String {
    set.into_iter()
        .flat_map(|set| &set.output_parameter_ref)
        .map(|output| {
            format!(
                "{} | {}",
                output.parameter_ref,
                output.output_name.as_deref().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_outputs(value: &str) -> Option<xtce::OutputSetType> {
    let output_parameter_ref = value
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(2, '|').map(str::trim);
            let parameter_ref = fields.next()?.to_owned();
            let output_name = fields
                .next()
                .map(str::to_owned)
                .filter(|value| !value.is_empty());
            (!parameter_ref.is_empty()).then_some(xtce::OutputParameterRefType {
                parameter_ref,
                output_name,
            })
        })
        .collect::<Vec<_>>();
    (!output_parameter_ref.is_empty()).then_some(xtce::OutputSetType {
        output_parameter_ref,
    })
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
            input.auto_grow(4, 20)
        } else {
            input
        }
    })
}

fn select<T>(
    choices: &[T],
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<T>>>
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    let selected_index = choices
        .iter()
        .position(|choice| *choice == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            choices.to_vec(),
            Some(IndexPath::default().row(selected_index)),
            window,
            cx,
        )
    })
}

fn sync_select<T>(
    select: &Entity<SelectState<Vec<T>>>,
    selected: T,
    window: &mut Window,
    cx: &mut impl AppContext,
) where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select.update(cx, |select, cx| {
        select.set_selected_value(&selected, window, cx);
    });
}

fn selected_value<T>(select: &Entity<SelectState<Vec<T>>>, fallback: T, cx: &App) -> T
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(fallback)
}

fn select_field<T>(
    label: &'static str,
    hint: &'static str,
    select: &Entity<SelectState<Vec<T>>>,
) -> Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    super::select_field(label, hint, select)
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn optional_parse<T: std::str::FromStr>(value: &str) -> Option<T> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.parse().ok()).flatten()
}

#[cfg(test)]
mod tests {
    use super::{
        decode_external_algorithms, decode_inputs, decode_outputs, decode_triggers,
        default_custom_algorithm, encode_external_algorithms, encode_inputs, encode_outputs,
        encode_triggers,
    };

    #[test]
    fn custom_algorithm_compact_collections_round_trip() {
        let external = decode_external_algorithms("impl | /opt/algorithm").unwrap();
        assert_eq!(
            encode_external_algorithms(Some(&external)),
            "impl | /opt/algorithm"
        );

        let inputs =
            decode_inputs("parameter | P1 | 2 | false | input\nconstant | C1 | 42").unwrap();
        assert_eq!(
            encode_inputs(Some(&inputs)),
            "parameter | P1 | 2 | false | input\nconstant | C1 | 42"
        );

        let outputs = decode_outputs("P2 | result").unwrap();
        assert_eq!(encode_outputs(Some(&outputs)), "P2 | result");

        let triggers = decode_triggers("main", "2", "parameter | P1\nperiodic | 0.5").unwrap();
        assert_eq!(
            encode_triggers(Some(&triggers)),
            "parameter | P1\nperiodic | 0.5"
        );
        assert_eq!(triggers.name.as_deref(), Some("main"));
        assert_eq!(triggers.trigger_rate, 2);
    }

    #[test]
    fn new_custom_algorithm_uses_xtce_thread_default() {
        let xtce::AlgorithmSetTypeContent::CustomAlgorithm(algorithm) =
            default_custom_algorithm("CustomAlgorithm1".to_owned())
        else {
            panic!("expected custom algorithm");
        };
        assert_eq!(algorithm.name, "CustomAlgorithm1");
        assert!(!algorithm.thread);
    }
}

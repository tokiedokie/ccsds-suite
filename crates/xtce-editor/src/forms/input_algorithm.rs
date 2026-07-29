use gpui::{App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{ActiveTheme, input::InputState, v_flex};

use super::{
    alias_set::AliasSetForm,
    ancillary_data_set::AncillaryDataSetForm,
    custom_algorithm::{
        decode_algorithm_text, decode_external_algorithms, decode_inputs,
        encode_external_algorithms, encode_inputs,
    },
    field, optional_value,
};

pub(super) struct InputAlgorithmForm {
    name: Entity<InputState>,
    short_description: Entity<InputState>,
    long_description: Entity<InputState>,
    language: Entity<InputState>,
    algorithm_text: Entity<InputState>,
    external_algorithms: Entity<InputState>,
    inputs: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
}

impl InputAlgorithmForm {
    pub(super) fn new(
        algorithm: Option<&xtce::InputAlgorithmType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = InputAlgorithmValues::from_algorithm(algorithm);
        cx.new(move |cx| Self {
            name: input(&values.name, false, window, cx),
            short_description: input(&values.short_description, false, window, cx),
            long_description: input(&values.long_description, true, window, cx),
            language: input(&values.language, false, window, cx),
            algorithm_text: input(&values.algorithm_text, true, window, cx),
            external_algorithms: input(&values.external_algorithms, true, window, cx),
            inputs: input(&values.inputs, true, window, cx),
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
        })
    }

    pub(super) fn load(
        &mut self,
        algorithm: Option<&xtce::InputAlgorithmType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = InputAlgorithmValues::from_algorithm(algorithm);
        for (input, value) in [
            (&self.name, values.name),
            (&self.short_description, values.short_description),
            (&self.long_description, values.long_description),
            (&self.language, values.language),
            (&self.algorithm_text, values.algorithm_text),
            (&self.external_algorithms, values.external_algorithms),
            (&self.inputs, values.inputs),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
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

    pub(super) fn algorithm(&self, cx: &App) -> xtce::InputAlgorithmType {
        let mut alias_set = None;
        self.alias_set.apply_to_option(&mut alias_set, cx);
        let mut ancillary_data_set = None;
        self.ancillary_data_set
            .apply_to_option(&mut ancillary_data_set, cx);
        xtce::InputAlgorithmType {
            name: value(&self.name, cx),
            short_description: optional_value(value(&self.short_description, cx)),
            long_description: optional_value(value(&self.long_description, cx)),
            alias_set,
            ancillary_data_set,
            algorithm_text: decode_algorithm_text(
                value(&self.language, cx),
                value(&self.algorithm_text, cx),
            ),
            external_algorithm_set: decode_external_algorithms(&value(
                &self.external_algorithms,
                cx,
            )),
            input_set: decode_inputs(&value(&self.inputs, cx)),
        }
    }

    fn render_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
            .child(
                v_flex()
                    .p_3()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(super::section_title("Algorithm identity"))
                    .child(field("Name", "Required", &self.name, cx))
                    .child(field(
                        "Short description",
                        "Optional",
                        &self.short_description,
                        cx,
                    ))
                    .child(field(
                        "Long description",
                        "Optional",
                        &self.long_description,
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .p_3()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(super::section_title("Implementation"))
                    .child(field(
                        "Language",
                        "Optional; defaults to pseudo",
                        &self.language,
                        cx,
                    ))
                    .child(field(
                        "Algorithm text",
                        "Optional",
                        &self.algorithm_text,
                        cx,
                    ))
                    .child(field(
                        "External algorithms",
                        "One per line: implementation name | algorithm location",
                        &self.external_algorithms,
                        cx,
                    )),
            )
            .child(field(
                "Inputs",
                "One per line: parameter | ref | instance | calibrated | input name, or constant | name | value",
                &self.inputs,
                cx,
            ))
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
    }
}

impl Render for InputAlgorithmForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_form(cx)
    }
}

struct InputAlgorithmValues {
    name: String,
    short_description: String,
    long_description: String,
    language: String,
    algorithm_text: String,
    external_algorithms: String,
    inputs: String,
}

impl InputAlgorithmValues {
    fn from_algorithm(algorithm: Option<&xtce::InputAlgorithmType>) -> Self {
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
        }
    }
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
    use super::InputAlgorithmValues;

    #[test]
    fn custom_match_algorithm_values_include_text_and_inputs() {
        let algorithm = xtce::InputAlgorithmType {
            short_description: None,
            name: "validPacket".to_owned(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            algorithm_text: Some(xtce::AlgorithmTextType {
                language: "python".to_owned(),
                content: "return packet_ok".to_owned(),
            }),
            external_algorithm_set: None,
            input_set: Some(xtce::InputSetType {
                content: vec![xtce::InputSetTypeContent::Constant(xtce::ConstantType {
                    constant_name: "limit".to_owned(),
                    value: "5".to_owned(),
                })],
            }),
        };
        let values = InputAlgorithmValues::from_algorithm(Some(&algorithm));
        assert_eq!(values.name, "validPacket");
        assert_eq!(values.language, "python");
        assert_eq!(values.inputs, "constant | limit | 5");
    }
}

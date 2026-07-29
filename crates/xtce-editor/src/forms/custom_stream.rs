use gpui::{
    App, AppContext, Context, Div, Entity, ParentElement, Render, Styled, Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme, IndexPath, h_flex,
    input::{InputEvent, InputState},
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
enum PcmChoice {
    #[strum(serialize = "NRZL")]
    Nrzl,
    #[strum(serialize = "NRZM")]
    Nrzm,
    #[strum(serialize = "NRZS")]
    Nrzs,
    BiPhaseL,
    BiPhaseM,
    BiPhaseS,
}
impl_select_item!(PcmChoice);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

pub(super) struct CustomStreamForm {
    present: bool,
    name_input: Entity<InputState>,
    bit_rate_input: Entity<InputState>,
    pcm_select: Entity<SelectState<Vec<PcmChoice>>>,
    inverted_select: Entity<SelectState<Vec<BooleanChoice>>>,
    encoded_stream_ref_input: Entity<InputState>,
    decoded_stream_ref_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    encoding_name_input: Entity<InputState>,
    encoding_short_description_input: Entity<InputState>,
    encoding_long_description_input: Entity<InputState>,
    encoding_language_input: Entity<InputState>,
    encoding_text_input: Entity<InputState>,
    decoding_name_input: Entity<InputState>,
    decoding_thread_select: Entity<SelectState<Vec<BooleanChoice>>>,
    decoding_short_description_input: Entity<InputState>,
    decoding_long_description_input: Entity<InputState>,
    decoding_language_input: Entity<InputState>,
    decoding_text_input: Entity<InputState>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    _subscriptions: Vec<Subscription>,
}

impl CustomStreamForm {
    pub(super) fn new(
        stream: Option<&xtce::StreamSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let stream = custom_stream(stream);
        let values = StreamValues::from_stream(stream);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        cx.new(move |cx| Self {
            present: stream.is_some(),
            name_input,
            bit_rate_input: input(&values.bit_rate, false, window, cx),
            pcm_select: select(PcmChoice::VARIANTS, values.pcm, window, cx),
            inverted_select: select(BooleanChoice::VARIANTS, values.inverted, window, cx),
            encoded_stream_ref_input: input(&values.encoded_stream_ref, false, window, cx),
            decoded_stream_ref_input: input(&values.decoded_stream_ref, false, window, cx),
            short_description_input: input(&values.short_description, false, window, cx),
            long_description_input: input(&values.long_description, true, window, cx),
            encoding_name_input: input(&values.encoding_name, false, window, cx),
            encoding_short_description_input: input(
                &values.encoding_short_description,
                false,
                window,
                cx,
            ),
            encoding_long_description_input: input(
                &values.encoding_long_description,
                true,
                window,
                cx,
            ),
            encoding_language_input: input(&values.encoding_language, false, window, cx),
            encoding_text_input: input(&values.encoding_text, true, window, cx),
            decoding_name_input: input(&values.decoding_name, false, window, cx),
            decoding_thread_select: select(
                BooleanChoice::VARIANTS,
                values.decoding_thread,
                window,
                cx,
            ),
            decoding_short_description_input: input(
                &values.decoding_short_description,
                false,
                window,
                cx,
            ),
            decoding_long_description_input: input(
                &values.decoding_long_description,
                true,
                window,
                cx,
            ),
            decoding_language_input: input(&values.decoding_language, false, window, cx),
            decoding_text_input: input(&values.decoding_text, true, window, cx),
            alias_set: AliasSetForm::new(
                stream.and_then(|stream| stream.alias_set.as_ref()),
                window,
                cx,
            ),
            ancillary_data_set: AncillaryDataSetForm::new(
                stream.and_then(|stream| stream.ancillary_data_set.as_ref()),
                window,
                cx,
            ),
            _subscriptions: vec![name_subscription],
        })
    }

    pub(super) fn load(
        &mut self,
        stream: Option<&xtce::StreamSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let stream = custom_stream(stream);
        let values = StreamValues::from_stream(stream);
        self.present = stream.is_some();
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.bit_rate_input, values.bit_rate),
            (&self.encoded_stream_ref_input, values.encoded_stream_ref),
            (&self.decoded_stream_ref_input, values.decoded_stream_ref),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.encoding_name_input, values.encoding_name),
            (
                &self.encoding_short_description_input,
                values.encoding_short_description,
            ),
            (
                &self.encoding_long_description_input,
                values.encoding_long_description,
            ),
            (&self.encoding_language_input, values.encoding_language),
            (&self.encoding_text_input, values.encoding_text),
            (&self.decoding_name_input, values.decoding_name),
            (
                &self.decoding_short_description_input,
                values.decoding_short_description,
            ),
            (
                &self.decoding_long_description_input,
                values.decoding_long_description,
            ),
            (&self.decoding_language_input, values.decoding_language),
            (&self.decoding_text_input, values.decoding_text),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        sync_select(&self.pcm_select, values.pcm, window, cx);
        sync_select(&self.inverted_select, values.inverted, window, cx);
        sync_select(
            &self.decoding_thread_select,
            values.decoding_thread,
            window,
            cx,
        );
        self.alias_set.load(
            stream.and_then(|stream| stream.alias_set.as_ref()),
            window,
            cx,
        );
        self.ancillary_data_set.load(
            stream.and_then(|stream| stream.ancillary_data_set.as_ref()),
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

    pub(super) fn apply_to(&self, stream: &mut xtce::StreamSetTypeContent, cx: &App) {
        let xtce::StreamSetTypeContent::CustomStream(stream) = stream else {
            return;
        };
        stream.name = value(&self.name_input, cx);
        stream.bit_rate_in_bps = optional_parse(&value(&self.bit_rate_input, cx));
        stream.pcm_type = pcm_value(selected_value(&self.pcm_select, PcmChoice::Nrzl, cx));
        stream.inverted =
            selected_value(&self.inverted_select, BooleanChoice::False, cx) == BooleanChoice::True;
        stream.encoded_stream_ref = value(&self.encoded_stream_ref_input, cx);
        stream.decoded_stream_ref = value(&self.decoded_stream_ref_input, cx);
        stream.short_description = optional_value(value(&self.short_description_input, cx));
        stream.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut stream.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut stream.ancillary_data_set, cx);

        apply_algorithm_text(
            &mut stream.encoding_algorithm.algorithm_text,
            value(&self.encoding_language_input, cx),
            value(&self.encoding_text_input, cx),
        );
        stream.encoding_algorithm.name = value(&self.encoding_name_input, cx);
        stream.encoding_algorithm.short_description =
            optional_value(value(&self.encoding_short_description_input, cx));
        stream.encoding_algorithm.long_description =
            optional_value(value(&self.encoding_long_description_input, cx));

        apply_algorithm_text(
            &mut stream.decoding_algorithm.algorithm_text,
            value(&self.decoding_language_input, cx),
            value(&self.decoding_text_input, cx),
        );
        stream.decoding_algorithm.name = value(&self.decoding_name_input, cx);
        stream.decoding_algorithm.thread =
            selected_value(&self.decoding_thread_select, BooleanChoice::False, cx)
                == BooleanChoice::True;
        stream.decoding_algorithm.short_description =
            optional_value(value(&self.decoding_short_description_input, cx));
        stream.decoding_algorithm.long_description =
            optional_value(value(&self.decoding_long_description_input, cx));
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        if !self.present {
            return v_flex().child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("The selected CustomStream is not present."),
            );
        }
        v_flex()
            .w_full()
            .gap_5()
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(select_field("PCM type", "Required", &self.pcm_select))
                    .child(select_field("Inverted", "Required", &self.inverted_select))
                    .child(field(
                        "Bit rate (bps)",
                        "Optional",
                        &self.bit_rate_input,
                        cx,
                    )),
            )
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(field(
                        "Encoded stream reference",
                        "Required",
                        &self.encoded_stream_ref_input,
                        cx,
                    ))
                    .child(field(
                        "Decoded stream reference",
                        "Required",
                        &self.decoded_stream_ref_input,
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
                    .child(super::section_title("Encoding algorithm"))
                    .child(field("Name", "Required", &self.encoding_name_input, cx))
                    .child(field(
                        "Short description",
                        "Optional",
                        &self.encoding_short_description_input,
                        cx,
                    ))
                    .child(field(
                        "Long description",
                        "Optional",
                        &self.encoding_long_description_input,
                        cx,
                    ))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Algorithm language",
                                "Optional; defaults to pseudo",
                                &self.encoding_language_input,
                                cx,
                            ))
                            .child(field(
                                "Algorithm text",
                                "Optional",
                                &self.encoding_text_input,
                                cx,
                            )),
                    ),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(super::section_title("Decoding algorithm"))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field("Name", "Required", &self.decoding_name_input, cx))
                            .child(select_field(
                                "Thread",
                                "Required",
                                &self.decoding_thread_select,
                            )),
                    )
                    .child(field(
                        "Short description",
                        "Optional",
                        &self.decoding_short_description_input,
                        cx,
                    ))
                    .child(field(
                        "Long description",
                        "Optional",
                        &self.decoding_long_description_input,
                        cx,
                    ))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Algorithm language",
                                "Optional; defaults to pseudo",
                                &self.decoding_language_input,
                                cx,
                            ))
                            .child(field(
                                "Algorithm text",
                                "Optional",
                                &self.decoding_text_input,
                                cx,
                            )),
                    ),
            )
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
    }
}

impl Render for CustomStreamForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_form(cx)
    }
}

struct StreamValues {
    name: String,
    bit_rate: String,
    pcm: PcmChoice,
    inverted: BooleanChoice,
    encoded_stream_ref: String,
    decoded_stream_ref: String,
    short_description: String,
    long_description: String,
    encoding_name: String,
    encoding_short_description: String,
    encoding_long_description: String,
    encoding_language: String,
    encoding_text: String,
    decoding_name: String,
    decoding_thread: BooleanChoice,
    decoding_short_description: String,
    decoding_long_description: String,
    decoding_language: String,
    decoding_text: String,
}

impl StreamValues {
    fn from_stream(stream: Option<&xtce::CustomStreamType>) -> Self {
        let encoding = stream.map(|stream| &stream.encoding_algorithm);
        let decoding = stream.map(|stream| &stream.decoding_algorithm);
        Self {
            name: stream.map(|stream| stream.name.clone()).unwrap_or_default(),
            bit_rate: stream
                .and_then(|stream| stream.bit_rate_in_bps)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            pcm: stream
                .map(|stream| pcm_choice(&stream.pcm_type))
                .unwrap_or(PcmChoice::Nrzl),
            inverted: if stream.is_some_and(|stream| stream.inverted) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            encoded_stream_ref: stream
                .map(|stream| stream.encoded_stream_ref.clone())
                .unwrap_or_default(),
            decoded_stream_ref: stream
                .map(|stream| stream.decoded_stream_ref.clone())
                .unwrap_or_default(),
            short_description: stream
                .and_then(|stream| stream.short_description.clone())
                .unwrap_or_default(),
            long_description: stream
                .and_then(|stream| stream.long_description.clone())
                .unwrap_or_default(),
            encoding_name: encoding
                .map(|algorithm| algorithm.name.clone())
                .unwrap_or_else(|| "EncodingAlgorithm".to_owned()),
            encoding_short_description: encoding
                .and_then(|algorithm| algorithm.short_description.clone())
                .unwrap_or_default(),
            encoding_long_description: encoding
                .and_then(|algorithm| algorithm.long_description.clone())
                .unwrap_or_default(),
            encoding_language: encoding
                .and_then(|algorithm| algorithm.algorithm_text.as_ref())
                .map(|text| text.language.clone())
                .unwrap_or_else(xtce::AlgorithmTextType::default_language),
            encoding_text: encoding
                .and_then(|algorithm| algorithm.algorithm_text.as_ref())
                .map(|text| text.content.clone())
                .unwrap_or_default(),
            decoding_name: decoding
                .map(|algorithm| algorithm.name.clone())
                .unwrap_or_else(|| "DecodingAlgorithm".to_owned()),
            decoding_thread: if decoding.is_some_and(|algorithm| algorithm.thread) {
                BooleanChoice::True
            } else {
                BooleanChoice::False
            },
            decoding_short_description: decoding
                .and_then(|algorithm| algorithm.short_description.clone())
                .unwrap_or_default(),
            decoding_long_description: decoding
                .and_then(|algorithm| algorithm.long_description.clone())
                .unwrap_or_default(),
            decoding_language: decoding
                .and_then(|algorithm| algorithm.algorithm_text.as_ref())
                .map(|text| text.language.clone())
                .unwrap_or_else(xtce::AlgorithmTextType::default_language),
            decoding_text: decoding
                .and_then(|algorithm| algorithm.algorithm_text.as_ref())
                .map(|text| text.content.clone())
                .unwrap_or_default(),
        }
    }
}

pub(crate) fn default_custom_stream(name: String) -> xtce::StreamSetTypeContent {
    xtce::StreamSetTypeContent::CustomStream(xtce::CustomStreamType {
        short_description: None,
        name,
        bit_rate_in_bps: None,
        pcm_type: xtce::CustomStreamType::default_pcm_type(),
        inverted: xtce::CustomStreamType::default_inverted(),
        encoded_stream_ref: String::new(),
        decoded_stream_ref: String::new(),
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        encoding_algorithm: default_input_algorithm("EncodingAlgorithm"),
        decoding_algorithm: default_input_output_algorithm("DecodingAlgorithm"),
    })
}

fn default_input_algorithm(name: &str) -> xtce::InputAlgorithmType {
    xtce::InputAlgorithmType {
        short_description: None,
        name: name.to_owned(),
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        algorithm_text: None,
        external_algorithm_set: None,
        input_set: None,
    }
}

fn default_input_output_algorithm(name: &str) -> xtce::InputOutputAlgorithmType {
    xtce::InputOutputAlgorithmType {
        short_description: None,
        name: name.to_owned(),
        thread: xtce::InputOutputAlgorithmType::default_thread(),
        long_description: None,
        alias_set: None,
        ancillary_data_set: None,
        algorithm_text: None,
        external_algorithm_set: None,
        input_set: None,
        output_set: None,
    }
}

fn apply_algorithm_text(
    target: &mut Option<xtce::AlgorithmTextType>,
    language: String,
    content: String,
) {
    if content.is_empty() {
        *target = None;
    } else {
        *target = Some(xtce::AlgorithmTextType {
            language: if language.is_empty() {
                xtce::AlgorithmTextType::default_language()
            } else {
                language
            },
            content,
        });
    }
}

fn custom_stream(stream: Option<&xtce::StreamSetTypeContent>) -> Option<&xtce::CustomStreamType> {
    match stream {
        Some(xtce::StreamSetTypeContent::CustomStream(stream)) => Some(stream),
        _ => None,
    }
}

fn pcm_choice(value: &xtce::PcmType) -> PcmChoice {
    match value {
        xtce::PcmType::Nrzl => PcmChoice::Nrzl,
        xtce::PcmType::Nrzm => PcmChoice::Nrzm,
        xtce::PcmType::Nrzs => PcmChoice::Nrzs,
        xtce::PcmType::BiPhaseL => PcmChoice::BiPhaseL,
        xtce::PcmType::BiPhaseM => PcmChoice::BiPhaseM,
        xtce::PcmType::BiPhaseS => PcmChoice::BiPhaseS,
    }
}

fn pcm_value(value: PcmChoice) -> xtce::PcmType {
    match value {
        PcmChoice::Nrzl => xtce::PcmType::Nrzl,
        PcmChoice::Nrzm => xtce::PcmType::Nrzm,
        PcmChoice::Nrzs => xtce::PcmType::Nrzs,
        PcmChoice::BiPhaseL => xtce::PcmType::BiPhaseL,
        PcmChoice::BiPhaseM => xtce::PcmType::BiPhaseM,
        PcmChoice::BiPhaseS => xtce::PcmType::BiPhaseS,
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
    use super::default_custom_stream;

    #[test]
    fn new_custom_stream_has_empty_required_stream_references() {
        let xtce::StreamSetTypeContent::CustomStream(stream) =
            default_custom_stream("CustomStream1".to_owned())
        else {
            panic!("expected custom stream");
        };
        assert!(stream.encoded_stream_ref.is_empty());
        assert!(stream.decoded_stream_ref.is_empty());
        assert_eq!(stream.encoding_algorithm.name, "EncodingAlgorithm");
        assert_eq!(stream.decoding_algorithm.name, "DecodingAlgorithm");
        assert!(!stream.decoding_algorithm.thread);
    }

    #[test]
    fn custom_stream_defaults_use_xtce_pcm_and_inversion_defaults() {
        let xtce::StreamSetTypeContent::CustomStream(stream) =
            default_custom_stream("CustomStream1".to_owned())
        else {
            panic!("expected custom stream");
        };
        assert!(matches!(stream.pcm_type, xtce::PcmType::Nrzl));
        assert!(!stream.inverted);
    }
}

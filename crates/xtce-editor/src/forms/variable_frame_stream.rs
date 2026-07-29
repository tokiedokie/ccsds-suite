use gpui::{
    App, AppContext, Context, Div, Entity, ParentElement, Render, Styled, Subscription, Window,
    div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, IndexPath, StyledExt, h_flex,
    input::{Input, InputEvent, InputState},
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    alias_set::AliasSetForm, ancillary_data_set::AncillaryDataSetForm, field, impl_select_item,
    input_algorithm::InputAlgorithmForm, optional_value,
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

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ReferenceKind {
    ContainerRef,
    ServiceRef,
}
impl_select_item!(ReferenceKind);

#[derive(Clone, Copy, Debug, Display, EnumString, VariantArray, PartialEq, Eq)]
enum FlagBitChoice {
    #[strum(serialize = "ones")]
    Ones,
    #[strum(serialize = "zeros")]
    Zeros,
}
impl_select_item!(FlagBitChoice);

pub(super) struct VariableFrameStreamForm {
    present: bool,
    name_input: Entity<InputState>,
    bit_rate_input: Entity<InputState>,
    pcm_select: Entity<SelectState<Vec<PcmChoice>>>,
    inverted_select: Entity<SelectState<Vec<BooleanChoice>>>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    reference_kind_select: Entity<SelectState<Vec<ReferenceKind>>>,
    reference_input: Entity<InputState>,
    stream_ref_input: Entity<InputState>,
    verify_to_lock_input: Entity<InputState>,
    check_to_lock_input: Entity<InputState>,
    max_bit_errors_input: Entity<InputState>,
    flag_size_input: Entity<InputState>,
    flag_bit_select: Entity<SelectState<Vec<FlagBitChoice>>>,
    auto_invert_present: bool,
    bad_frames_to_auto_invert_input: Entity<InputState>,
    invert_algorithm_present: bool,
    invert_algorithm: Entity<InputAlgorithmForm>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    _subscriptions: Vec<Subscription>,
}

impl VariableFrameStreamForm {
    pub(super) fn new(
        stream: Option<&xtce::StreamSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let stream = variable_stream(stream);
        let values = StreamValues::from_stream(stream);
        let name_input = input(&values.name, false, window, cx);
        let name_subscription = cx.subscribe(&name_input, |editor, _, _: &InputEvent, cx| {
            editor.refresh_tree(cx);
            cx.notify();
        });
        let invert_algorithm = InputAlgorithmForm::new(invert_algorithm(stream), window, cx);
        cx.new(move |cx| {
            let reference_kind_select =
                select(ReferenceKind::VARIANTS, values.reference_kind, window, cx);
            let reference_subscription = cx.subscribe(
                &reference_kind_select,
                |_, _, _: &SelectEvent<Vec<ReferenceKind>>, cx| cx.notify(),
            );
            Self {
                present: stream.is_some(),
                name_input,
                bit_rate_input: input(&values.bit_rate, false, window, cx),
                pcm_select: select(PcmChoice::VARIANTS, values.pcm, window, cx),
                inverted_select: select(BooleanChoice::VARIANTS, values.inverted, window, cx),
                short_description_input: input(&values.short_description, false, window, cx),
                long_description_input: input(&values.long_description, true, window, cx),
                reference_kind_select,
                reference_input: input(&values.reference, false, window, cx),
                stream_ref_input: input(&values.stream_ref, false, window, cx),
                verify_to_lock_input: input(&values.verify_to_lock, false, window, cx),
                check_to_lock_input: input(&values.check_to_lock, false, window, cx),
                max_bit_errors_input: input(&values.max_bit_errors, false, window, cx),
                flag_size_input: input(&values.flag_size, false, window, cx),
                flag_bit_select: select(FlagBitChoice::VARIANTS, values.flag_bit, window, cx),
                auto_invert_present: values.auto_invert_present,
                bad_frames_to_auto_invert_input: input(
                    &values.bad_frames_to_auto_invert,
                    false,
                    window,
                    cx,
                ),
                invert_algorithm_present: values.invert_algorithm_present,
                invert_algorithm,
                alias_set: AliasSetForm::new(alias_set(stream), window, cx),
                ancillary_data_set: AncillaryDataSetForm::new(
                    ancillary_data_set(stream),
                    window,
                    cx,
                ),
                _subscriptions: vec![name_subscription, reference_subscription],
            }
        })
    }

    pub(super) fn load(
        &mut self,
        stream: Option<&xtce::StreamSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let stream = variable_stream(stream);
        let values = StreamValues::from_stream(stream);
        self.present = stream.is_some();
        self.auto_invert_present = values.auto_invert_present;
        self.invert_algorithm_present = values.invert_algorithm_present;
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.bit_rate_input, values.bit_rate),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.reference_input, values.reference),
            (&self.stream_ref_input, values.stream_ref),
            (&self.verify_to_lock_input, values.verify_to_lock),
            (&self.check_to_lock_input, values.check_to_lock),
            (&self.max_bit_errors_input, values.max_bit_errors),
            (&self.flag_size_input, values.flag_size),
            (
                &self.bad_frames_to_auto_invert_input,
                values.bad_frames_to_auto_invert,
            ),
        ] {
            input.update(cx, |input, cx| input.set_value(value, window, cx));
        }
        sync_select(&self.pcm_select, values.pcm, window, cx);
        sync_select(&self.inverted_select, values.inverted, window, cx);
        sync_select(
            &self.reference_kind_select,
            values.reference_kind,
            window,
            cx,
        );
        sync_select(&self.flag_bit_select, values.flag_bit, window, cx);
        self.alias_set.load(alias_set(stream), window, cx);
        self.ancillary_data_set
            .load(ancillary_data_set(stream), window, cx);
        self.invert_algorithm.update(cx, |form, cx| {
            form.load(invert_algorithm(stream), window, cx);
        });
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

    pub(super) fn apply_to(&self, stream: &mut xtce::StreamSetTypeContent, cx: &App) {
        let xtce::StreamSetTypeContent::VariableFrameStream(stream) = stream else {
            return;
        };
        stream.name = value(&self.name_input, cx);
        stream.bit_rate_in_bps = optional_parse(&value(&self.bit_rate_input, cx));
        stream.pcm_type = pcm_value(selected_value(&self.pcm_select, PcmChoice::Nrzl, cx));
        stream.inverted =
            selected_value(&self.inverted_select, BooleanChoice::False, cx) == BooleanChoice::True;
        stream.short_description = optional_value(value(&self.short_description_input, cx));

        let mut content = StreamContent::take(&mut stream.content);
        content.long_description = optional_value(value(&self.long_description_input, cx));
        self.alias_set.apply_to_option(&mut content.alias_set, cx);
        self.ancillary_data_set
            .apply_to_option(&mut content.ancillary_data_set, cx);
        content.reference = Some(
            match selected_value(&self.reference_kind_select, ReferenceKind::ContainerRef, cx) {
                ReferenceKind::ContainerRef => StreamReference::Container(xtce::ContainerRefType {
                    container_ref: value(&self.reference_input, cx),
                }),
                ReferenceKind::ServiceRef => StreamReference::Service(xtce::ServiceRefType {
                    service_ref: value(&self.reference_input, cx),
                }),
            },
        );
        content.stream_ref = optional_value(value(&self.stream_ref_input, cx))
            .map(|stream_ref| xtce::StreamRefType { stream_ref });

        let sync = content
            .sync_strategy
            .get_or_insert_with(default_sync_strategy);
        update_i64(
            &mut sync.verify_to_lock_good_frames,
            &self.verify_to_lock_input,
            cx,
        );
        update_i64(
            &mut sync.check_to_lock_good_frames,
            &self.check_to_lock_input,
            cx,
        );
        update_i64(
            &mut sync.max_bit_errors_in_sync_pattern,
            &self.max_bit_errors_input,
            cx,
        );
        update_i64(&mut sync.flag.flag_size_in_bits, &self.flag_size_input, cx);
        sync.flag.flag_bit_type = flag_bit_value(selected_value(
            &self.flag_bit_select,
            FlagBitChoice::Ones,
            cx,
        ));
        if self.auto_invert_present {
            let auto_invert = sync
                .auto_invert
                .get_or_insert_with(|| xtce::AutoInvertType {
                    bad_frames_to_auto_invert:
                        xtce::AutoInvertType::default_bad_frames_to_auto_invert(),
                    invert_algorithm: None,
                });
            update_i64(
                &mut auto_invert.bad_frames_to_auto_invert,
                &self.bad_frames_to_auto_invert_input,
                cx,
            );
            auto_invert.invert_algorithm = self
                .invert_algorithm_present
                .then(|| self.invert_algorithm.read(cx).algorithm(cx));
        } else {
            sync.auto_invert = None;
        }
        stream.content = content.into_content();
    }

    fn render_form(&self, cx: &mut Context<Self>) -> Div {
        if !self.present {
            return v_flex().child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("The selected VariableFrameStream is not present."),
            );
        }
        let reference_kind =
            selected_value(&self.reference_kind_select, ReferenceKind::ContainerRef, cx);
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
            .child(field(
                "Short description",
                "Optional",
                &self.short_description_input,
                cx,
            ))
            .child(
                v_flex()
                    .gap_3()
                    .child(div().text_sm().font_medium().child("Frame source"))
                    .child(select_field(
                        "Reference target type",
                        "Required",
                        &self.reference_kind_select,
                    ))
                    .child(field(
                        match reference_kind {
                            ReferenceKind::ContainerRef => "Container reference",
                            ReferenceKind::ServiceRef => "Service reference",
                        },
                        "Required",
                        &self.reference_input,
                        cx,
                    ))
                    .child(field(
                        "Connecting stream reference",
                        "Optional",
                        &self.stream_ref_input,
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(div().text_sm().font_medium().child("Synchronization"))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Verify-to-lock frames",
                                "Optional; defaults to 4",
                                &self.verify_to_lock_input,
                                cx,
                            ))
                            .child(field(
                                "Check-to-lock frames",
                                "Optional; defaults to 1",
                                &self.check_to_lock_input,
                                cx,
                            ))
                            .child(field(
                                "Maximum sync bit errors",
                                "Optional; defaults to 0",
                                &self.max_bit_errors_input,
                                cx,
                            )),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Flag size (bits)",
                                "Optional; defaults to 6",
                                &self.flag_size_input,
                                cx,
                            ))
                            .child(select_field(
                                "Flag bit type",
                                "Required",
                                &self.flag_bit_select,
                            )),
                    ),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(div().text_sm().font_medium().child("Auto invert"))
                            .child(if self.auto_invert_present {
                                super::section_remove_button("remove-variable-frame-auto-invert")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.auto_invert_present = false;
                                        cx.notify();
                                    }))
                            } else {
                                super::section_add_button("add-variable-frame-auto-invert")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.auto_invert_present = true;
                                        cx.notify();
                                    }))
                            }),
                    )
                    .when(self.auto_invert_present, |section| {
                        section
                            .child(field(
                                "Bad frames to auto-invert",
                                "Optional; defaults to 1024",
                                &self.bad_frames_to_auto_invert_input,
                                cx,
                            ))
                            .child(
                                v_flex()
                                    .gap_3()
                                    .child(
                                        h_flex()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .child("Invert algorithm"),
                                            )
                                            .child(if self.invert_algorithm_present {
                                                super::section_remove_button(
                                                    "remove-variable-frame-invert-algorithm",
                                                )
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.invert_algorithm_present = false;
                                                    cx.notify();
                                                }))
                                            } else {
                                                super::section_add_button(
                                                    "add-variable-frame-invert-algorithm",
                                                )
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.invert_algorithm_present = true;
                                                    cx.notify();
                                                }))
                                            }),
                                    )
                                    .when(self.invert_algorithm_present, |form| {
                                        form.child(self.invert_algorithm.clone())
                                    }),
                            )
                    }),
            )
            .child(field(
                "Long description",
                "Optional",
                &self.long_description_input,
                cx,
            ))
            .child(self.alias_set.render(cx))
            .child(self.ancillary_data_set.render(cx))
    }
}

impl Render for VariableFrameStreamForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_form(cx)
    }
}

struct StreamValues {
    name: String,
    bit_rate: String,
    pcm: PcmChoice,
    inverted: BooleanChoice,
    short_description: String,
    long_description: String,
    reference_kind: ReferenceKind,
    reference: String,
    stream_ref: String,
    verify_to_lock: String,
    check_to_lock: String,
    max_bit_errors: String,
    flag_size: String,
    flag_bit: FlagBitChoice,
    auto_invert_present: bool,
    bad_frames_to_auto_invert: String,
    invert_algorithm_present: bool,
}

impl StreamValues {
    fn from_stream(stream: Option<&xtce::VariableFrameStreamType>) -> Self {
        let reference = stream.and_then(frame_reference);
        let sync = stream.and_then(sync_strategy);
        let auto_invert = sync.and_then(|sync| sync.auto_invert.as_ref());
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
            short_description: stream
                .and_then(|stream| stream.short_description.clone())
                .unwrap_or_default(),
            long_description: stream
                .and_then(|stream| {
                    stream.content.iter().find_map(|content| match content {
                        xtce::VariableFrameStreamTypeContent::LongDescription(value) => {
                            Some(value.clone())
                        }
                        _ => None,
                    })
                })
                .unwrap_or_default(),
            reference_kind: match reference {
                Some(StreamReferenceRef::Service(_)) => ReferenceKind::ServiceRef,
                _ => ReferenceKind::ContainerRef,
            },
            reference: reference.map(StreamReferenceRef::value).unwrap_or_default(),
            stream_ref: stream
                .and_then(|stream| {
                    stream.content.iter().find_map(|content| match content {
                        xtce::VariableFrameStreamTypeContent::StreamRef(value) => {
                            Some(value.stream_ref.clone())
                        }
                        _ => None,
                    })
                })
                .unwrap_or_default(),
            verify_to_lock: sync
                .map(|sync| sync.verify_to_lock_good_frames.to_string())
                .unwrap_or_else(|| "4".to_owned()),
            check_to_lock: sync
                .map(|sync| sync.check_to_lock_good_frames.to_string())
                .unwrap_or_else(|| "1".to_owned()),
            max_bit_errors: sync
                .map(|sync| sync.max_bit_errors_in_sync_pattern.to_string())
                .unwrap_or_else(|| "0".to_owned()),
            flag_size: sync
                .map(|sync| sync.flag.flag_size_in_bits.to_string())
                .unwrap_or_else(|| "6".to_owned()),
            flag_bit: sync
                .map(|sync| flag_bit_choice(&sync.flag.flag_bit_type))
                .unwrap_or(FlagBitChoice::Ones),
            auto_invert_present: auto_invert.is_some(),
            bad_frames_to_auto_invert: auto_invert
                .map(|value| value.bad_frames_to_auto_invert.to_string())
                .unwrap_or_else(|| "1024".to_owned()),
            invert_algorithm_present: auto_invert
                .is_some_and(|value| value.invert_algorithm.is_some()),
        }
    }
}

enum StreamReference {
    Container(xtce::ContainerRefType),
    Service(xtce::ServiceRefType),
}

enum StreamReferenceRef<'a> {
    Container(&'a xtce::ContainerRefType),
    Service(&'a xtce::ServiceRefType),
}

impl StreamReferenceRef<'_> {
    fn value(self) -> String {
        match self {
            Self::Container(value) => value.container_ref.clone(),
            Self::Service(value) => value.service_ref.clone(),
        }
    }
}

struct StreamContent {
    long_description: Option<String>,
    alias_set: Option<xtce::AliasSetType>,
    ancillary_data_set: Option<xtce::AncillaryDataSetType>,
    reference: Option<StreamReference>,
    stream_ref: Option<xtce::StreamRefType>,
    sync_strategy: Option<xtce::VariableFrameSyncStrategyType>,
}

impl StreamContent {
    fn take(content: &mut Vec<xtce::VariableFrameStreamTypeContent>) -> Self {
        let mut result = Self {
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            reference: None,
            stream_ref: None,
            sync_strategy: None,
        };
        for item in std::mem::take(content) {
            match item {
                xtce::VariableFrameStreamTypeContent::LongDescription(value) => {
                    result.long_description = Some(value)
                }
                xtce::VariableFrameStreamTypeContent::AliasSet(value) => {
                    result.alias_set = Some(value)
                }
                xtce::VariableFrameStreamTypeContent::AncillaryDataSet(value) => {
                    result.ancillary_data_set = Some(value)
                }
                xtce::VariableFrameStreamTypeContent::ContainerRef(value) => {
                    result.reference = Some(StreamReference::Container(value))
                }
                xtce::VariableFrameStreamTypeContent::ServiceRef(value) => {
                    result.reference = Some(StreamReference::Service(value))
                }
                xtce::VariableFrameStreamTypeContent::StreamRef(value) => {
                    result.stream_ref = Some(value)
                }
                xtce::VariableFrameStreamTypeContent::SyncStrategy(value) => {
                    result.sync_strategy = Some(value)
                }
            }
        }
        result
    }

    fn into_content(self) -> Vec<xtce::VariableFrameStreamTypeContent> {
        let mut content = Vec::new();
        if let Some(value) = self.long_description {
            content.push(xtce::VariableFrameStreamTypeContent::LongDescription(value));
        }
        if let Some(value) = self.alias_set {
            content.push(xtce::VariableFrameStreamTypeContent::AliasSet(value));
        }
        if let Some(value) = self.ancillary_data_set {
            content.push(xtce::VariableFrameStreamTypeContent::AncillaryDataSet(
                value,
            ));
        }
        if let Some(value) = self.reference {
            content.push(match value {
                StreamReference::Container(value) => {
                    xtce::VariableFrameStreamTypeContent::ContainerRef(value)
                }
                StreamReference::Service(value) => {
                    xtce::VariableFrameStreamTypeContent::ServiceRef(value)
                }
            });
        }
        if let Some(value) = self.stream_ref {
            content.push(xtce::VariableFrameStreamTypeContent::StreamRef(value));
        }
        if let Some(value) = self.sync_strategy {
            content.push(xtce::VariableFrameStreamTypeContent::SyncStrategy(value));
        }
        content
    }
}

fn variable_stream(
    stream: Option<&xtce::StreamSetTypeContent>,
) -> Option<&xtce::VariableFrameStreamType> {
    match stream {
        Some(xtce::StreamSetTypeContent::VariableFrameStream(stream)) => Some(stream),
        _ => None,
    }
}

fn alias_set(stream: Option<&xtce::VariableFrameStreamType>) -> Option<&xtce::AliasSetType> {
    stream.and_then(|stream| {
        stream.content.iter().find_map(|content| match content {
            xtce::VariableFrameStreamTypeContent::AliasSet(value) => Some(value),
            _ => None,
        })
    })
}

fn ancillary_data_set(
    stream: Option<&xtce::VariableFrameStreamType>,
) -> Option<&xtce::AncillaryDataSetType> {
    stream.and_then(|stream| {
        stream.content.iter().find_map(|content| match content {
            xtce::VariableFrameStreamTypeContent::AncillaryDataSet(value) => Some(value),
            _ => None,
        })
    })
}

fn frame_reference(stream: &xtce::VariableFrameStreamType) -> Option<StreamReferenceRef<'_>> {
    stream.content.iter().find_map(|content| match content {
        xtce::VariableFrameStreamTypeContent::ContainerRef(value) => {
            Some(StreamReferenceRef::Container(value))
        }
        xtce::VariableFrameStreamTypeContent::ServiceRef(value) => {
            Some(StreamReferenceRef::Service(value))
        }
        _ => None,
    })
}

fn sync_strategy(
    stream: &xtce::VariableFrameStreamType,
) -> Option<&xtce::VariableFrameSyncStrategyType> {
    stream.content.iter().find_map(|content| match content {
        xtce::VariableFrameStreamTypeContent::SyncStrategy(value) => Some(value),
        _ => None,
    })
}

fn invert_algorithm(
    stream: Option<&xtce::VariableFrameStreamType>,
) -> Option<&xtce::InputAlgorithmType> {
    stream
        .and_then(sync_strategy)
        .and_then(|sync| sync.auto_invert.as_ref())
        .and_then(|auto_invert| auto_invert.invert_algorithm.as_ref())
}

pub(crate) fn default_variable_frame_stream(name: String) -> xtce::StreamSetTypeContent {
    xtce::StreamSetTypeContent::VariableFrameStream(xtce::VariableFrameStreamType {
        short_description: None,
        name,
        bit_rate_in_bps: None,
        pcm_type: xtce::VariableFrameStreamType::default_pcm_type(),
        inverted: xtce::VariableFrameStreamType::default_inverted(),
        content: vec![
            xtce::VariableFrameStreamTypeContent::ContainerRef(xtce::ContainerRefType {
                container_ref: String::new(),
            }),
            xtce::VariableFrameStreamTypeContent::SyncStrategy(default_sync_strategy()),
        ],
    })
}

fn default_sync_strategy() -> xtce::VariableFrameSyncStrategyType {
    xtce::VariableFrameSyncStrategyType {
        verify_to_lock_good_frames:
            xtce::VariableFrameSyncStrategyType::default_verify_to_lock_good_frames(),
        check_to_lock_good_frames:
            xtce::VariableFrameSyncStrategyType::default_check_to_lock_good_frames(),
        max_bit_errors_in_sync_pattern:
            xtce::VariableFrameSyncStrategyType::default_max_bit_errors_in_sync_pattern(),
        auto_invert: None,
        flag: xtce::FlagType {
            flag_size_in_bits: xtce::FlagType::default_flag_size_in_bits(),
            flag_bit_type: xtce::FlagType::default_flag_bit_type(),
        },
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

fn flag_bit_choice(value: &xtce::FlagBitType) -> FlagBitChoice {
    match value {
        xtce::FlagBitType::Ones => FlagBitChoice::Ones,
        xtce::FlagBitType::Zeros => FlagBitChoice::Zeros,
    }
}

fn flag_bit_value(value: FlagBitChoice) -> xtce::FlagBitType {
    match value {
        FlagBitChoice::Ones => xtce::FlagBitType::Ones,
        FlagBitChoice::Zeros => xtce::FlagBitType::Zeros,
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

fn update_i64(target: &mut i64, input: &Entity<InputState>, cx: &App) {
    if let Ok(value) = value(input, cx).trim().parse() {
        *target = value;
    }
}

#[cfg(test)]
mod tests {
    use super::{default_variable_frame_stream, invert_algorithm};

    #[test]
    fn new_variable_frame_stream_uses_the_schema_flag_defaults() {
        let xtce::StreamSetTypeContent::VariableFrameStream(stream) =
            default_variable_frame_stream("VariableFrameStream1".to_owned())
        else {
            unreachable!()
        };

        assert_eq!(stream.name, "VariableFrameStream1");
        let sync = stream.content.iter().find_map(|content| match content {
            xtce::VariableFrameStreamTypeContent::SyncStrategy(value) => Some(value),
            _ => None,
        });
        let sync = sync.expect("sync strategy");
        assert_eq!(sync.flag.flag_size_in_bits, 6);
        assert!(matches!(sync.flag.flag_bit_type, xtce::FlagBitType::Ones));
    }

    #[test]
    fn configured_invert_algorithm_is_exposed_to_the_form() {
        let xtce::StreamSetTypeContent::VariableFrameStream(mut stream) =
            default_variable_frame_stream("Frames".to_owned())
        else {
            unreachable!()
        };
        let sync = stream.content.iter_mut().find_map(|content| match content {
            xtce::VariableFrameStreamTypeContent::SyncStrategy(sync) => Some(sync),
            _ => None,
        });
        sync.expect("sync strategy").auto_invert = Some(xtce::AutoInvertType {
            bad_frames_to_auto_invert: 10,
            invert_algorithm: Some(test_algorithm("invert")),
        });

        assert_eq!(
            invert_algorithm(Some(&stream)).map(|algorithm| algorithm.name.as_str()),
            Some("invert")
        );
    }

    fn test_algorithm(name: &str) -> xtce::InputAlgorithmType {
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
}

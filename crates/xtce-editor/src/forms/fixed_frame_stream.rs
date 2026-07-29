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

pub(super) struct FixedFrameStreamForm {
    present: bool,
    name_input: Entity<InputState>,
    bit_rate_input: Entity<InputState>,
    pcm_select: Entity<SelectState<Vec<PcmChoice>>>,
    inverted_select: Entity<SelectState<Vec<BooleanChoice>>>,
    sync_aperture_input: Entity<InputState>,
    frame_length_input: Entity<InputState>,
    short_description_input: Entity<InputState>,
    long_description_input: Entity<InputState>,
    reference_kind_select: Entity<SelectState<Vec<ReferenceKind>>>,
    reference_input: Entity<InputState>,
    stream_ref_input: Entity<InputState>,
    verify_to_lock_input: Entity<InputState>,
    check_to_lock_input: Entity<InputState>,
    max_bit_errors_input: Entity<InputState>,
    sync_pattern_input: Entity<InputState>,
    pattern_location_input: Entity<InputState>,
    pattern_mask_input: Entity<InputState>,
    mask_length_input: Entity<InputState>,
    pattern_length_input: Entity<InputState>,
    auto_invert_present: bool,
    bad_frames_to_auto_invert_input: Entity<InputState>,
    invert_algorithm_present: bool,
    invert_algorithm: Entity<InputAlgorithmForm>,
    alias_set: AliasSetForm,
    ancillary_data_set: AncillaryDataSetForm,
    _subscriptions: Vec<Subscription>,
}

impl FixedFrameStreamForm {
    pub(super) fn new(
        stream: Option<&xtce::StreamSetTypeContent>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let stream = fixed_stream(stream);
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
                sync_aperture_input: input(&values.sync_aperture, false, window, cx),
                frame_length_input: input(&values.frame_length, false, window, cx),
                short_description_input: input(&values.short_description, false, window, cx),
                long_description_input: input(&values.long_description, true, window, cx),
                reference_kind_select,
                reference_input: input(&values.reference, false, window, cx),
                stream_ref_input: input(&values.stream_ref, false, window, cx),
                verify_to_lock_input: input(&values.verify_to_lock, false, window, cx),
                check_to_lock_input: input(&values.check_to_lock, false, window, cx),
                max_bit_errors_input: input(&values.max_bit_errors, false, window, cx),
                sync_pattern_input: input(&values.sync_pattern, false, window, cx),
                pattern_location_input: input(&values.pattern_location, false, window, cx),
                pattern_mask_input: input(&values.pattern_mask, false, window, cx),
                mask_length_input: input(&values.mask_length, false, window, cx),
                pattern_length_input: input(&values.pattern_length, false, window, cx),
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
        let stream = fixed_stream(stream);
        let values = StreamValues::from_stream(stream);
        self.present = stream.is_some();
        self.auto_invert_present = values.auto_invert_present;
        self.invert_algorithm_present = values.invert_algorithm_present;
        for (input, value) in [
            (&self.name_input, values.name),
            (&self.bit_rate_input, values.bit_rate),
            (&self.sync_aperture_input, values.sync_aperture),
            (&self.frame_length_input, values.frame_length),
            (&self.short_description_input, values.short_description),
            (&self.long_description_input, values.long_description),
            (&self.reference_input, values.reference),
            (&self.stream_ref_input, values.stream_ref),
            (&self.verify_to_lock_input, values.verify_to_lock),
            (&self.check_to_lock_input, values.check_to_lock),
            (&self.max_bit_errors_input, values.max_bit_errors),
            (&self.sync_pattern_input, values.sync_pattern),
            (&self.pattern_location_input, values.pattern_location),
            (&self.pattern_mask_input, values.pattern_mask),
            (&self.mask_length_input, values.mask_length),
            (&self.pattern_length_input, values.pattern_length),
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
        let xtce::StreamSetTypeContent::FixedFrameStream(stream) = stream else {
            return;
        };
        stream.name = value(&self.name_input, cx);
        stream.bit_rate_in_bps = optional_parse(&value(&self.bit_rate_input, cx));
        stream.pcm_type = pcm_value(selected_value(&self.pcm_select, PcmChoice::Nrzl, cx));
        stream.inverted =
            selected_value(&self.inverted_select, BooleanChoice::False, cx) == BooleanChoice::True;
        update_i64(
            &mut stream.sync_aperture_in_bits,
            &self.sync_aperture_input,
            cx,
        );
        update_i64(
            &mut stream.frame_length_in_bits,
            &self.frame_length_input,
            cx,
        );
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
        sync.sync_pattern.pattern = value(&self.sync_pattern_input, cx);
        update_i64(
            &mut sync.sync_pattern.bit_location_from_start_of_container,
            &self.pattern_location_input,
            cx,
        );
        sync.sync_pattern.mask = optional_value(value(&self.pattern_mask_input, cx));
        sync.sync_pattern.mask_length_in_bits = optional_parse(&value(&self.mask_length_input, cx));
        update_i64(
            &mut sync.sync_pattern.pattern_length_in_bits,
            &self.pattern_length_input,
            cx,
        );
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
                    .child("The selected FixedFrameStream is not present."),
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
                    .child(select_field("Inverted", "Required", &self.inverted_select)),
            )
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .child(field(
                        "Bit rate (bps)",
                        "Optional",
                        &self.bit_rate_input,
                        cx,
                    ))
                    .child(field(
                        "Frame length (bits)",
                        "Required",
                        &self.frame_length_input,
                        cx,
                    ))
                    .child(field(
                        "Sync aperture (bits)",
                        "Optional; defaults to 0",
                        &self.sync_aperture_input,
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
                    .child(field(
                        "Sync pattern",
                        "Required hexadecimal byte string",
                        &self.sync_pattern_input,
                        cx,
                    ))
                    .child(
                        h_flex()
                            .gap_4()
                            .items_start()
                            .child(field(
                                "Pattern length (bits)",
                                "Required",
                                &self.pattern_length_input,
                                cx,
                            ))
                            .child(field(
                                "Pattern bit location",
                                "Optional; defaults to 0",
                                &self.pattern_location_input,
                                cx,
                            ))
                            .child(field(
                                "Mask length (bits)",
                                "Optional",
                                &self.mask_length_input,
                                cx,
                            )),
                    )
                    .child(field(
                        "Pattern mask",
                        "Optional hexadecimal byte string",
                        &self.pattern_mask_input,
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .justify_between()
                            .child(div().text_sm().font_medium().child("Auto invert"))
                            .child(if self.auto_invert_present {
                                super::section_remove_button("remove-fixed-frame-auto-invert")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.auto_invert_present = false;
                                        cx.notify();
                                    }))
                            } else {
                                super::section_add_button("add-fixed-frame-auto-invert").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.auto_invert_present = true;
                                        cx.notify();
                                    }),
                                )
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
                                                    "remove-fixed-frame-invert-algorithm",
                                                )
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.invert_algorithm_present = false;
                                                    cx.notify();
                                                }))
                                            } else {
                                                super::section_add_button(
                                                    "add-fixed-frame-invert-algorithm",
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

impl Render for FixedFrameStreamForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.render_form(cx)
    }
}

struct StreamValues {
    name: String,
    bit_rate: String,
    pcm: PcmChoice,
    inverted: BooleanChoice,
    sync_aperture: String,
    frame_length: String,
    short_description: String,
    long_description: String,
    reference_kind: ReferenceKind,
    reference: String,
    stream_ref: String,
    verify_to_lock: String,
    check_to_lock: String,
    max_bit_errors: String,
    sync_pattern: String,
    pattern_location: String,
    pattern_mask: String,
    mask_length: String,
    pattern_length: String,
    auto_invert_present: bool,
    bad_frames_to_auto_invert: String,
    invert_algorithm_present: bool,
}

impl StreamValues {
    fn from_stream(stream: Option<&xtce::FixedFrameStreamType>) -> Self {
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
            sync_aperture: stream
                .map(|stream| stream.sync_aperture_in_bits.to_string())
                .unwrap_or_else(|| "0".to_owned()),
            frame_length: stream
                .map(|stream| stream.frame_length_in_bits.to_string())
                .unwrap_or_default(),
            short_description: stream
                .and_then(|stream| stream.short_description.clone())
                .unwrap_or_default(),
            long_description: stream
                .and_then(|stream| {
                    stream.content.iter().find_map(|content| match content {
                        xtce::FixedFrameStreamTypeContent::LongDescription(value) => {
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
                        xtce::FixedFrameStreamTypeContent::StreamRef(value) => {
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
            sync_pattern: sync
                .map(|sync| sync.sync_pattern.pattern.clone())
                .unwrap_or_default(),
            pattern_location: sync
                .map(|sync| {
                    sync.sync_pattern
                        .bit_location_from_start_of_container
                        .to_string()
                })
                .unwrap_or_else(|| "0".to_owned()),
            pattern_mask: sync
                .and_then(|sync| sync.sync_pattern.mask.clone())
                .unwrap_or_default(),
            mask_length: sync
                .and_then(|sync| sync.sync_pattern.mask_length_in_bits)
                .map(|value| value.to_string())
                .unwrap_or_default(),
            pattern_length: sync
                .map(|sync| sync.sync_pattern.pattern_length_in_bits.to_string())
                .unwrap_or_default(),
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
    sync_strategy: Option<xtce::FixedFrameSyncStrategyType>,
}

impl StreamContent {
    fn take(content: &mut Vec<xtce::FixedFrameStreamTypeContent>) -> Self {
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
                xtce::FixedFrameStreamTypeContent::LongDescription(value) => {
                    result.long_description = Some(value)
                }
                xtce::FixedFrameStreamTypeContent::AliasSet(value) => {
                    result.alias_set = Some(value)
                }
                xtce::FixedFrameStreamTypeContent::AncillaryDataSet(value) => {
                    result.ancillary_data_set = Some(value)
                }
                xtce::FixedFrameStreamTypeContent::ContainerRef(value) => {
                    result.reference = Some(StreamReference::Container(value))
                }
                xtce::FixedFrameStreamTypeContent::ServiceRef(value) => {
                    result.reference = Some(StreamReference::Service(value))
                }
                xtce::FixedFrameStreamTypeContent::StreamRef(value) => {
                    result.stream_ref = Some(value)
                }
                xtce::FixedFrameStreamTypeContent::SyncStrategy(value) => {
                    result.sync_strategy = Some(value)
                }
            }
        }
        result
    }

    fn into_content(self) -> Vec<xtce::FixedFrameStreamTypeContent> {
        let mut content = Vec::new();
        if let Some(value) = self.long_description {
            content.push(xtce::FixedFrameStreamTypeContent::LongDescription(value));
        }
        if let Some(value) = self.alias_set {
            content.push(xtce::FixedFrameStreamTypeContent::AliasSet(value));
        }
        if let Some(value) = self.ancillary_data_set {
            content.push(xtce::FixedFrameStreamTypeContent::AncillaryDataSet(value));
        }
        if let Some(value) = self.reference {
            content.push(match value {
                StreamReference::Container(value) => {
                    xtce::FixedFrameStreamTypeContent::ContainerRef(value)
                }
                StreamReference::Service(value) => {
                    xtce::FixedFrameStreamTypeContent::ServiceRef(value)
                }
            });
        }
        if let Some(value) = self.stream_ref {
            content.push(xtce::FixedFrameStreamTypeContent::StreamRef(value));
        }
        if let Some(value) = self.sync_strategy {
            content.push(xtce::FixedFrameStreamTypeContent::SyncStrategy(value));
        }
        content
    }
}

fn fixed_stream(
    stream: Option<&xtce::StreamSetTypeContent>,
) -> Option<&xtce::FixedFrameStreamType> {
    match stream {
        Some(xtce::StreamSetTypeContent::FixedFrameStream(stream)) => Some(stream),
        _ => None,
    }
}

fn alias_set(stream: Option<&xtce::FixedFrameStreamType>) -> Option<&xtce::AliasSetType> {
    stream.and_then(|stream| {
        stream.content.iter().find_map(|content| match content {
            xtce::FixedFrameStreamTypeContent::AliasSet(value) => Some(value),
            _ => None,
        })
    })
}

fn ancillary_data_set(
    stream: Option<&xtce::FixedFrameStreamType>,
) -> Option<&xtce::AncillaryDataSetType> {
    stream.and_then(|stream| {
        stream.content.iter().find_map(|content| match content {
            xtce::FixedFrameStreamTypeContent::AncillaryDataSet(value) => Some(value),
            _ => None,
        })
    })
}

fn frame_reference(stream: &xtce::FixedFrameStreamType) -> Option<StreamReferenceRef<'_>> {
    stream.content.iter().find_map(|content| match content {
        xtce::FixedFrameStreamTypeContent::ContainerRef(value) => {
            Some(StreamReferenceRef::Container(value))
        }
        xtce::FixedFrameStreamTypeContent::ServiceRef(value) => {
            Some(StreamReferenceRef::Service(value))
        }
        _ => None,
    })
}

fn sync_strategy(stream: &xtce::FixedFrameStreamType) -> Option<&xtce::FixedFrameSyncStrategyType> {
    stream.content.iter().find_map(|content| match content {
        xtce::FixedFrameStreamTypeContent::SyncStrategy(value) => Some(value),
        _ => None,
    })
}

fn invert_algorithm(
    stream: Option<&xtce::FixedFrameStreamType>,
) -> Option<&xtce::InputAlgorithmType> {
    stream
        .and_then(sync_strategy)
        .and_then(|sync| sync.auto_invert.as_ref())
        .and_then(|auto_invert| auto_invert.invert_algorithm.as_ref())
}

pub(crate) fn default_fixed_frame_stream(name: String) -> xtce::StreamSetTypeContent {
    xtce::StreamSetTypeContent::FixedFrameStream(xtce::FixedFrameStreamType {
        short_description: None,
        name,
        bit_rate_in_bps: None,
        pcm_type: xtce::FixedFrameStreamType::default_pcm_type(),
        inverted: xtce::FixedFrameStreamType::default_inverted(),
        sync_aperture_in_bits: xtce::FixedFrameStreamType::default_sync_aperture_in_bits(),
        frame_length_in_bits: 0,
        content: vec![
            xtce::FixedFrameStreamTypeContent::ContainerRef(xtce::ContainerRefType {
                container_ref: String::new(),
            }),
            xtce::FixedFrameStreamTypeContent::SyncStrategy(default_sync_strategy()),
        ],
    })
}

fn default_sync_strategy() -> xtce::FixedFrameSyncStrategyType {
    xtce::FixedFrameSyncStrategyType {
        verify_to_lock_good_frames:
            xtce::FixedFrameSyncStrategyType::default_verify_to_lock_good_frames(),
        check_to_lock_good_frames:
            xtce::FixedFrameSyncStrategyType::default_check_to_lock_good_frames(),
        max_bit_errors_in_sync_pattern:
            xtce::FixedFrameSyncStrategyType::default_max_bit_errors_in_sync_pattern(),
        auto_invert: None,
        sync_pattern: xtce::SyncPatternType {
            pattern: String::new(),
            bit_location_from_start_of_container:
                xtce::SyncPatternType::default_bit_location_from_start_of_container(),
            mask: None,
            mask_length_in_bits: None,
            pattern_length_in_bits: 0,
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
    use super::{StreamContent, default_fixed_frame_stream, invert_algorithm};

    #[test]
    fn new_fixed_frame_stream_has_required_empty_references() {
        let xtce::StreamSetTypeContent::FixedFrameStream(stream) =
            default_fixed_frame_stream("FixedFrameStream1".to_owned())
        else {
            unreachable!()
        };

        assert_eq!(stream.name, "FixedFrameStream1");
        assert_eq!(stream.frame_length_in_bits, 0);
        assert!(stream.content.iter().any(|content| matches!(
            content,
            xtce::FixedFrameStreamTypeContent::ContainerRef(value)
                if value.container_ref.is_empty()
        )));
        assert!(
            stream.content.iter().any(|content| matches!(
                content,
                xtce::FixedFrameStreamTypeContent::SyncStrategy(_)
            ))
        );
    }

    #[test]
    fn stream_content_rebuilds_in_schema_order() {
        let mut content = vec![
            xtce::FixedFrameStreamTypeContent::SyncStrategy(super::default_sync_strategy()),
            xtce::FixedFrameStreamTypeContent::LongDescription("description".to_owned()),
            xtce::FixedFrameStreamTypeContent::ContainerRef(xtce::ContainerRefType {
                container_ref: "Packets".to_owned(),
            }),
        ];

        let rebuilt = StreamContent::take(&mut content).into_content();

        assert!(matches!(
            rebuilt.first(),
            Some(xtce::FixedFrameStreamTypeContent::LongDescription(_))
        ));
        assert!(matches!(
            rebuilt.last(),
            Some(xtce::FixedFrameStreamTypeContent::SyncStrategy(_))
        ));
    }

    #[test]
    fn configured_invert_algorithm_is_exposed_to_the_form() {
        let xtce::StreamSetTypeContent::FixedFrameStream(mut stream) =
            default_fixed_frame_stream("Frames".to_owned())
        else {
            unreachable!()
        };
        let sync = stream.content.iter_mut().find_map(|content| match content {
            xtce::FixedFrameStreamTypeContent::SyncStrategy(sync) => Some(sync),
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

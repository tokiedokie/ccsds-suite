use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IndexPath, StyledExt, h_flex,
    input::InputState,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{
    discrete_lookup::DiscreteLookupListForm, dynamic_value::DynamicValueForm,
    error_detect_correct::ErrorDetectCorrectForm, field, impl_select_item,
    input_algorithm::InputAlgorithmForm,
};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum SizeKind {
    #[default]
    None,
    Fixed,
    Dynamic,
    #[strum(serialize = "Discrete lookup")]
    DiscreteLookup,
}
impl_select_item!(SizeKind);

pub(super) struct ContainerBinaryEncodingForm {
    present: bool,
    size_kind: SizeKind,
    size_kind_select: Entity<SelectState<Vec<SizeKind>>>,
    size_in_bits: Entity<InputState>,
    dynamic_size: Entity<DynamicValueForm>,
    discrete_size: Entity<DiscreteLookupListForm>,
    error_detection: Entity<ErrorDetectCorrectForm>,
    from_transform_present: bool,
    from_transform: Entity<InputAlgorithmForm>,
    to_transform_present: bool,
    to_transform: Entity<InputAlgorithmForm>,
}

impl ContainerBinaryEncodingForm {
    pub(super) fn new(
        encoding: Option<&xtce::ContainerBinaryDataEncodingType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = EncodingValues::from_encoding(encoding);
        let dynamic_size = DynamicValueForm::new(
            encoding
                .and_then(|encoding| encoding.size_in_bits.as_ref())
                .and_then(|size| match size {
                    xtce::IntegerValueType::DynamicValue(value) => Some(value),
                    _ => None,
                }),
            window,
            cx,
        );
        let discrete_size = DiscreteLookupListForm::new(
            encoding
                .and_then(|encoding| encoding.size_in_bits.as_ref())
                .and_then(|size| match size {
                    xtce::IntegerValueType::DiscreteLookupList(value) => Some(value),
                    _ => None,
                }),
            window,
            cx,
        );
        let error_detection = ErrorDetectCorrectForm::new(
            encoding.and_then(|encoding| encoding.error_detect_correct.as_ref()),
            window,
            cx,
        );
        let from_transform = InputAlgorithmForm::new(
            encoding.and_then(|encoding| encoding.from_binary_transform_algorithm.as_ref()),
            window,
            cx,
        );
        let to_transform = InputAlgorithmForm::new(
            encoding.and_then(|encoding| encoding.to_binary_transform_algorithm.as_ref()),
            window,
            cx,
        );
        cx.new(move |cx| {
            let size_kind_select = size_kind_select(values.size_kind, window, cx);
            let subscription = cx.subscribe_in(
                &size_kind_select,
                window,
                |this: &mut Self, _, event: &SelectEvent<Vec<SizeKind>>, _, cx| {
                    if let SelectEvent::Confirm(Some(kind)) = event {
                        this.size_kind = *kind;
                        cx.notify();
                    }
                },
            );
            subscription.detach();
            Self {
                present: encoding.is_some(),
                size_kind: values.size_kind,
                size_kind_select,
                size_in_bits: cx
                    .new(|cx| InputState::new(window, cx).default_value(values.size_in_bits)),
                dynamic_size,
                discrete_size,
                error_detection,
                from_transform_present: values.from_transform_present,
                from_transform,
                to_transform_present: values.to_transform_present,
                to_transform,
            }
        })
    }

    pub(super) fn load(
        &mut self,
        encoding: Option<&xtce::ContainerBinaryDataEncodingType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let values = EncodingValues::from_encoding(encoding);
        self.present = encoding.is_some();
        self.size_kind = values.size_kind;
        self.from_transform_present = values.from_transform_present;
        self.to_transform_present = values.to_transform_present;
        self.size_in_bits.update(cx, |input, cx| {
            input.set_value(values.size_in_bits, window, cx)
        });
        self.size_kind_select.update(cx, |select, cx| {
            select.set_selected_value(&values.size_kind, window, cx);
        });
        self.dynamic_size.update(cx, |form, cx| {
            form.load(
                encoding
                    .and_then(|encoding| encoding.size_in_bits.as_ref())
                    .and_then(|size| match size {
                        xtce::IntegerValueType::DynamicValue(value) => Some(value),
                        _ => None,
                    }),
                window,
                cx,
            );
        });
        self.discrete_size.update(cx, |form, cx| {
            form.load(
                encoding
                    .and_then(|encoding| encoding.size_in_bits.as_ref())
                    .and_then(|size| match size {
                        xtce::IntegerValueType::DiscreteLookupList(value) => Some(value),
                        _ => None,
                    }),
                window,
                cx,
            );
        });
        self.error_detection.update(cx, |form, cx| {
            form.load(
                encoding.and_then(|encoding| encoding.error_detect_correct.as_ref()),
                window,
                cx,
            );
        });
        self.from_transform.update(cx, |form, cx| {
            form.load(
                encoding.and_then(|encoding| encoding.from_binary_transform_algorithm.as_ref()),
                window,
                cx,
            );
        });
        self.to_transform.update(cx, |form, cx| {
            form.load(
                encoding.and_then(|encoding| encoding.to_binary_transform_algorithm.as_ref()),
                window,
                cx,
            );
        });
        cx.notify();
    }

    pub(super) fn apply_to(
        &self,
        encoding: &mut Option<xtce::ContainerBinaryDataEncodingType>,
        cx: &App,
    ) {
        if !self.present {
            *encoding = None;
            return;
        }
        let encoding = encoding.get_or_insert_with(|| xtce::ContainerBinaryDataEncodingType {
            error_detect_correct: None,
            size_in_bits: None,
            from_binary_transform_algorithm: None,
            to_binary_transform_algorithm: None,
        });
        match self.size_kind {
            SizeKind::None => encoding.size_in_bits = None,
            SizeKind::Fixed => {
                encoding.size_in_bits = self
                    .size_in_bits
                    .read(cx)
                    .value()
                    .trim()
                    .parse()
                    .ok()
                    .map(xtce::IntegerValueType::FixedValue);
            }
            SizeKind::Dynamic => {
                encoding.size_in_bits = Some(xtce::IntegerValueType::DynamicValue(
                    self.dynamic_size.read(cx).value(cx),
                ));
            }
            SizeKind::DiscreteLookup => {
                encoding.size_in_bits = Some(xtce::IntegerValueType::DiscreteLookupList(
                    self.discrete_size.read(cx).value(cx),
                ));
            }
        }
        self.error_detection
            .read(cx)
            .apply_to(&mut encoding.error_detect_correct, cx);
        encoding.from_binary_transform_algorithm = self
            .from_transform_present
            .then(|| self.from_transform.read(cx).algorithm(cx));
        encoding.to_binary_transform_algorithm = self
            .to_transform_present
            .then(|| self.to_transform.read(cx).algorithm(cx));
    }
}

impl Render for ContainerBinaryEncodingForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(div().text_sm().font_medium().child("Binary encoding"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Container-level binary processing"),
                            ),
                    )
                    .child(if self.present {
                        super::section_remove_button("remove-container-binary-encoding").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.present = false;
                                cx.notify();
                            }),
                        )
                    } else {
                        super::section_add_button("add-container-binary-encoding").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.present = true;
                                cx.notify();
                            }),
                        )
                    }),
            )
            .when(self.present, |form| {
                form.child(size_kind_field(&self.size_kind_select))
                    .when(self.size_kind == SizeKind::Fixed, |form| {
                        form.child(field(
                            "Size in bits",
                            "Required fixed size",
                            &self.size_in_bits,
                            cx,
                        ))
                    })
                    .when(self.size_kind == SizeKind::Dynamic, |form| {
                        form.child(self.dynamic_size.clone())
                    })
                    .when(self.size_kind == SizeKind::DiscreteLookup, |form| {
                        form.child(self.discrete_size.clone())
                    })
                    .child(self.error_detection.clone())
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
                                            .child("From-binary transform"),
                                    )
                                    .child(if self.from_transform_present {
                                        super::section_remove_button(
                                            "remove-container-from-binary-transform",
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.from_transform_present = false;
                                                cx.notify();
                                            }),
                                        )
                                    } else {
                                        super::section_add_button(
                                            "add-container-from-binary-transform",
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.from_transform_present = true;
                                                cx.notify();
                                            }),
                                        )
                                    }),
                            )
                            .when(self.from_transform_present, |section| {
                                section.child(self.from_transform.clone())
                            }),
                    )
                    .child(
                        v_flex()
                            .gap_3()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(
                                        div().text_sm().font_medium().child("To-binary transform"),
                                    )
                                    .child(if self.to_transform_present {
                                        super::section_remove_button(
                                            "remove-container-to-binary-transform",
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.to_transform_present = false;
                                                cx.notify();
                                            }),
                                        )
                                    } else {
                                        super::section_add_button(
                                            "add-container-to-binary-transform",
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.to_transform_present = true;
                                                cx.notify();
                                            }),
                                        )
                                    }),
                            )
                            .when(self.to_transform_present, |section| {
                                section.child(self.to_transform.clone())
                            }),
                    )
            })
    }
}

struct EncodingValues {
    size_kind: SizeKind,
    size_in_bits: String,
    from_transform_present: bool,
    to_transform_present: bool,
}

impl EncodingValues {
    fn from_encoding(encoding: Option<&xtce::ContainerBinaryDataEncodingType>) -> Self {
        let size = encoding.and_then(|encoding| encoding.size_in_bits.as_ref());
        Self {
            size_kind: match size {
                None => SizeKind::None,
                Some(xtce::IntegerValueType::FixedValue(_)) => SizeKind::Fixed,
                Some(xtce::IntegerValueType::DynamicValue(_)) => SizeKind::Dynamic,
                Some(xtce::IntegerValueType::DiscreteLookupList(_)) => SizeKind::DiscreteLookup,
            },
            size_in_bits: match size {
                Some(xtce::IntegerValueType::FixedValue(value)) => value.to_string(),
                _ => String::new(),
            },
            from_transform_present: encoding
                .is_some_and(|encoding| encoding.from_binary_transform_algorithm.is_some()),
            to_transform_present: encoding
                .is_some_and(|encoding| encoding.to_binary_transform_algorithm.is_some()),
        }
    }
}

fn size_kind_select(
    selected: SizeKind,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<SelectState<Vec<SizeKind>>> {
    let index = SizeKind::VARIANTS
        .iter()
        .position(|value| *value == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            SizeKind::VARIANTS.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn size_kind_field(select: &Entity<SelectState<Vec<SizeKind>>>) -> gpui::Div {
    v_flex().w_full().child(
        gpui_component::form::field()
            .label("Size in bits")
            .child(Select::new(select).w_full()),
    )
}

#[cfg(test)]
mod tests {
    use super::{EncodingValues, SizeKind};

    #[test]
    fn fixed_binary_size_is_available_to_the_form() {
        let encoding = xtce::ContainerBinaryDataEncodingType {
            error_detect_correct: None,
            size_in_bits: Some(xtce::IntegerValueType::FixedValue(128)),
            from_binary_transform_algorithm: None,
            to_binary_transform_algorithm: None,
        };

        let values = EncodingValues::from_encoding(Some(&encoding));

        assert_eq!(values.size_in_bits, "128");
        assert_eq!(values.size_kind, SizeKind::Fixed);
    }

    #[test]
    fn from_binary_transform_is_available_to_the_form() {
        let encoding = xtce::ContainerBinaryDataEncodingType {
            error_detect_correct: None,
            size_in_bits: None,
            from_binary_transform_algorithm: Some(xtce::InputAlgorithmType {
                short_description: None,
                name: "decodeFrame".to_owned(),
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                algorithm_text: None,
                external_algorithm_set: None,
                input_set: None,
            }),
            to_binary_transform_algorithm: None,
        };

        let values = EncodingValues::from_encoding(Some(&encoding));

        assert!(values.from_transform_present);
    }

    #[test]
    fn to_binary_transform_is_available_to_the_form() {
        let encoding = xtce::ContainerBinaryDataEncodingType {
            error_detect_correct: None,
            size_in_bits: None,
            from_binary_transform_algorithm: None,
            to_binary_transform_algorithm: Some(xtce::InputAlgorithmType {
                short_description: None,
                name: "encodeFrame".to_owned(),
                long_description: None,
                alias_set: None,
                ancillary_data_set: None,
                algorithm_text: None,
                external_algorithm_set: None,
                input_set: None,
            }),
        };

        let values = EncodingValues::from_encoding(Some(&encoding));

        assert!(values.to_transform_present);
    }
}

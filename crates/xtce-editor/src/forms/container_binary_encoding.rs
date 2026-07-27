use gpui::{
    App, AppContext, Context, Div, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, IconName, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputState,
    v_flex,
};

use super::{field, input_algorithm::InputAlgorithmForm};

pub(super) struct ContainerBinaryEncodingForm {
    present: bool,
    size_in_bits: Entity<InputState>,
    preserve_complex_size: bool,
    error_detection_summary: String,
    from_transform_present: bool,
    from_transform: Entity<InputAlgorithmForm>,
    to_transform_summary: String,
}

impl ContainerBinaryEncodingForm {
    pub(super) fn new(
        encoding: Option<&xtce::ContainerBinaryDataEncodingType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = EncodingValues::from_encoding(encoding);
        let from_transform = InputAlgorithmForm::new(
            encoding.and_then(|encoding| encoding.from_binary_transform_algorithm.as_ref()),
            window,
            cx,
        );
        cx.new(move |cx| Self {
            present: encoding.is_some(),
            size_in_bits: cx
                .new(|cx| InputState::new(window, cx).default_value(values.size_in_bits)),
            preserve_complex_size: values.preserve_complex_size,
            error_detection_summary: values.error_detection_summary,
            from_transform_present: values.from_transform_present,
            from_transform,
            to_transform_summary: values.to_transform_summary,
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
        self.preserve_complex_size = values.preserve_complex_size;
        self.error_detection_summary = values.error_detection_summary;
        self.from_transform_present = values.from_transform_present;
        self.to_transform_summary = values.to_transform_summary;
        self.size_in_bits.update(cx, |input, cx| {
            input.set_value(values.size_in_bits, window, cx)
        });
        self.from_transform.update(cx, |form, cx| {
            form.load(
                encoding.and_then(|encoding| encoding.from_binary_transform_algorithm.as_ref()),
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
        let size = self.size_in_bits.read(cx).value().trim().parse::<i64>();
        match size {
            Ok(size) => {
                encoding.size_in_bits = Some(xtce::IntegerValueType::FixedValue(size));
            }
            Err(_) if !self.preserve_complex_size => {
                encoding.size_in_bits = None;
            }
            Err(_) => {}
        }
        encoding.from_binary_transform_algorithm = self
            .from_transform_present
            .then(|| self.from_transform.read(cx).algorithm(cx));
    }
}

impl Render for ContainerBinaryEncodingForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_4()
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
                        Button::new("remove-container-binary-encoding")
                            .small()
                            .danger()
                            .label("Remove binary encoding")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.present = false;
                                cx.notify();
                            }))
                    } else {
                        Button::new("add-container-binary-encoding")
                            .small()
                            .icon(IconName::Plus)
                            .label("Add binary encoding")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.present = true;
                                cx.notify();
                            }))
                    }),
            )
            .when(self.present, |form| {
                form.child(field(
                    "Size in bits",
                    if self.preserve_complex_size {
                        "A dynamic or lookup size is configured; enter a number to replace it"
                    } else {
                        "Optional fixed size"
                    },
                    &self.size_in_bits,
                    cx,
                ))
                .child(
                    v_flex()
                        .gap_2()
                        .child(summary_row(
                            "Error detection/correction",
                            &self.error_detection_summary,
                            cx,
                        ))
                        .child(summary_row(
                            "To-binary transform",
                            &self.to_transform_summary,
                            cx,
                        )),
                )
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .justify_between()
                                .child(div().text_sm().font_medium().child("From-binary transform"))
                                .child(if self.from_transform_present {
                                    Button::new("remove-container-from-binary-transform")
                                        .small()
                                        .danger()
                                        .label("Remove")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.from_transform_present = false;
                                            cx.notify();
                                        }))
                                } else {
                                    Button::new("add-container-from-binary-transform")
                                        .small()
                                        .icon(IconName::Plus)
                                        .label("Add")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.from_transform_present = true;
                                            cx.notify();
                                        }))
                                }),
                        )
                        .when(self.from_transform_present, |section| {
                            section.child(self.from_transform.clone())
                        }),
                )
            })
    }
}

struct EncodingValues {
    size_in_bits: String,
    preserve_complex_size: bool,
    error_detection_summary: String,
    from_transform_present: bool,
    to_transform_summary: String,
}

impl EncodingValues {
    fn from_encoding(encoding: Option<&xtce::ContainerBinaryDataEncodingType>) -> Self {
        let size = encoding.and_then(|encoding| encoding.size_in_bits.as_ref());
        Self {
            size_in_bits: match size {
                Some(xtce::IntegerValueType::FixedValue(value)) => value.to_string(),
                _ => String::new(),
            },
            preserve_complex_size: matches!(
                size,
                Some(
                    xtce::IntegerValueType::DynamicValue(_)
                        | xtce::IntegerValueType::DiscreteLookupList(_)
                )
            ),
            error_detection_summary: presence(
                encoding.and_then(|encoding| encoding.error_detect_correct.as_ref()),
            ),
            from_transform_present: encoding
                .is_some_and(|encoding| encoding.from_binary_transform_algorithm.is_some()),
            to_transform_summary: algorithm_name(
                encoding.and_then(|encoding| encoding.to_binary_transform_algorithm.as_ref()),
            ),
        }
    }
}

fn presence<T>(value: Option<&T>) -> String {
    if value.is_some() {
        "Configured (preserved)".to_owned()
    } else {
        "Not configured".to_owned()
    }
}

fn algorithm_name(value: Option<&xtce::InputAlgorithmType>) -> String {
    value.map_or_else(|| "Not configured".to_owned(), |value| value.name.clone())
}

fn summary_row(label: &'static str, value: &str, cx: &App) -> Div {
    h_flex()
        .justify_between()
        .gap_4()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(div().text_sm().child(value.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::EncodingValues;

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
        assert!(!values.preserve_complex_size);
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
}

use gpui::{
    App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    IconName, IndexPath, Sizable, StyledExt,
    button::Button,
    h_flex,
    input::InputState,
    select::{SelectEvent, SelectState},
    v_flex,
};
use strum::{Display, EnumString, VariantArray};

use super::{field, impl_select_item, input_algorithm::InputAlgorithmForm, optional_value};

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ErrorKind {
    #[default]
    Checksum,
    #[strum(serialize = "CRC")]
    Crc,
    #[strum(serialize = "XOR")]
    Xor,
    Parity,
}
impl_select_item!(ErrorKind);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ReferenceChoice {
    #[default]
    #[strum(serialize = "start")]
    Start,
    #[strum(serialize = "end")]
    End,
}
impl_select_item!(ReferenceChoice);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum BooleanChoice {
    #[default]
    #[strum(serialize = "false")]
    False,
    #[strum(serialize = "true")]
    True,
}
impl_select_item!(BooleanChoice);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum DirectionChoice {
    #[default]
    #[strum(serialize = "most significant bit first")]
    MostSignificantBitFirst,
    #[strum(serialize = "least significant bit first")]
    LeastSignificantBitFirst,
}
impl_select_item!(DirectionChoice);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ParityChoice {
    #[default]
    Even,
    Odd,
}
impl_select_item!(ParityChoice);

#[derive(Clone, Copy, Debug, Default, Display, EnumString, VariantArray, PartialEq, Eq)]
enum ChecksumChoice {
    UnixSum,
    #[default]
    Sum8,
    Sum16,
    Sum24,
    Sum32,
    Fletcher4,
    Fletcher8,
    Fletcher16,
    Fletcher32,
    Adler32,
    Luhn,
    Verhoeff,
    Damm,
    Custom,
}
impl_select_item!(ChecksumChoice);

pub(super) struct ErrorDetectCorrectForm {
    present: bool,
    rows: Vec<Entity<ErrorRow>>,
}

struct ErrorRow {
    kind: ErrorKind,
    kind_select: Entity<SelectState<Vec<ErrorKind>>>,
    bits_from_reference: Entity<InputState>,
    reference: Entity<SelectState<Vec<ReferenceChoice>>>,
    parameter_ref: Entity<InputState>,
    checksum_name: Entity<SelectState<Vec<ChecksumChoice>>>,
    hash_size_in_bits: Entity<InputState>,
    input_algorithm_present: bool,
    input_algorithm: Entity<InputAlgorithmForm>,
    crc_width: Entity<InputState>,
    reflect_data: Entity<SelectState<Vec<BooleanChoice>>>,
    reflect_remainder: Entity<SelectState<Vec<BooleanChoice>>>,
    direction: Entity<SelectState<Vec<DirectionChoice>>>,
    polynomial: Entity<InputState>,
    init_remainder: Entity<InputState>,
    final_xor: Entity<InputState>,
    parity: Entity<SelectState<Vec<ParityChoice>>>,
}

impl ErrorDetectCorrectForm {
    pub(super) fn new(
        value: Option<&xtce::ErrorDetectCorrectType>,
        window: &mut Window,
        cx: &mut impl AppContext,
    ) -> Entity<Self> {
        let values = values(value);
        cx.new(|cx| Self {
            present: value.is_some(),
            rows: values
                .into_iter()
                .map(|(value, algorithm)| new_row(value, algorithm, window, cx))
                .collect(),
        })
    }

    pub(super) fn load(
        &mut self,
        value: Option<&xtce::ErrorDetectCorrectType>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.present = value.is_some();
        self.rows = values(value)
            .into_iter()
            .map(|(value, algorithm)| new_row(value, algorithm, window, cx))
            .collect();
        cx.notify();
    }

    pub(super) fn apply_to(&self, value: &mut Option<xtce::ErrorDetectCorrectType>, cx: &App) {
        *value = self.present.then(|| xtce::ErrorDetectCorrectType {
            content: self
                .rows
                .iter()
                .map(|row| row.read(cx).content(cx))
                .collect(),
        });
    }
}

impl Render for ErrorDetectCorrectForm {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                super::detail_list_card(cx)
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .child(format!("Detection/correction {}", index + 1)),
                            )
                            .child(
                                super::row_remove_button(
                                    format!("remove-error-detect-correct-{index}"),
                                    "Remove error detection/correction entry",
                                )
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        if index < this.rows.len() {
                                            this.rows.remove(index);
                                            cx.notify();
                                        }
                                    },
                                )),
                            ),
                    )
                    .child(row.clone())
            })
            .collect::<Vec<_>>();

        v_flex()
            .w_full()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .child("Error detection/correction"),
                    )
                    .child(if self.present {
                        super::section_remove_button("remove-error-detect-correct").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.present = false;
                                cx.notify();
                            }),
                        )
                    } else {
                        super::section_add_button("add-error-detect-correct").on_click(cx.listener(
                            |this, _, window, cx| {
                                this.present = true;
                                if this.rows.is_empty() {
                                    this.rows.push(new_row(
                                        ErrorValues::default(),
                                        None,
                                        window,
                                        cx,
                                    ));
                                }
                                cx.notify();
                            },
                        ))
                    }),
            )
            .when(self.present, |form| {
                form.children(rows).child(
                    Button::new("add-error-detect-correct-row")
                        .small()
                        .icon(IconName::Plus)
                        .label("Add method")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.rows
                                .push(new_row(ErrorValues::default(), None, window, cx));
                            cx.notify();
                        })),
                )
            })
    }
}

impl ErrorRow {
    fn content(&self, cx: &App) -> xtce::ErrorDetectCorrectTypeContent {
        let bits_from_reference = integer(&self.bits_from_reference, cx);
        let reference = match selected(&self.reference, ReferenceChoice::Start, cx) {
            ReferenceChoice::Start => xtce::ReferencePointType::Start,
            ReferenceChoice::End => xtce::ReferencePointType::End,
        };
        let parameter_ref = optional_value(value(&self.parameter_ref, cx));
        match self.kind {
            ErrorKind::Checksum => {
                xtce::ErrorDetectCorrectTypeContent::Checksum(xtce::ChecksumType {
                    bits_from_reference,
                    reference,
                    name: checksum_type(selected(&self.checksum_name, ChecksumChoice::Sum8, cx)),
                    hash_size_in_bits: optional_integer(&self.hash_size_in_bits, cx),
                    parameter_ref,
                    input_algorithm: self
                        .input_algorithm_present
                        .then(|| self.input_algorithm.read(cx).algorithm(cx)),
                })
            }
            ErrorKind::Crc => xtce::ErrorDetectCorrectTypeContent::Crc(xtce::CrcType {
                width: integer(&self.crc_width, cx),
                reflect_data: boolean(&self.reflect_data, cx),
                reflect_remainder: boolean(&self.reflect_remainder, cx),
                direction: match selected(
                    &self.direction,
                    DirectionChoice::MostSignificantBitFirst,
                    cx,
                ) {
                    DirectionChoice::MostSignificantBitFirst => {
                        xtce::BitOrderType::MostSignificantBitFirst
                    }
                    DirectionChoice::LeastSignificantBitFirst => {
                        xtce::BitOrderType::LeastSignificantBitFirst
                    }
                },
                bits_from_reference,
                reference,
                parameter_ref,
                polynomial: value(&self.polynomial, cx),
                init_remainder: optional_value(value(&self.init_remainder, cx)),
                final_xor: optional_value(value(&self.final_xor, cx)),
            }),
            ErrorKind::Xor => xtce::ErrorDetectCorrectTypeContent::Xor(xtce::XorType {
                bits_from_reference,
                reference,
                parameter_ref,
            }),
            ErrorKind::Parity => xtce::ErrorDetectCorrectTypeContent::Parity(xtce::ParityType {
                type_: match selected(&self.parity, ParityChoice::Even, cx) {
                    ParityChoice::Even => xtce::ParityFormType::Even,
                    ParityChoice::Odd => xtce::ParityFormType::Odd,
                },
                bits_from_reference,
                reference,
                parameter_ref,
            }),
        }
    }
}

impl Render for ErrorRow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_3()
            .child(select_field("Method", &self.kind_select))
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(field(
                        "Bits from reference",
                        "Defaults to 0",
                        &self.bits_from_reference,
                        cx,
                    ))
                    .child(select_field("Reference point", &self.reference))
                    .child(field(
                        "Parameter reference",
                        "Optional result destination",
                        &self.parameter_ref,
                        cx,
                    )),
            )
            .when(self.kind == ErrorKind::Checksum, |form| {
                form.child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(select_field("Checksum", &self.checksum_name))
                        .child(field(
                            "Hash size in bits",
                            "Optional",
                            &self.hash_size_in_bits,
                            cx,
                        )),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .child(div().text_sm().font_medium().child("Input algorithm"))
                        .child(if self.input_algorithm_present {
                            super::section_remove_button("remove-checksum-input-algorithm")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.input_algorithm_present = false;
                                    cx.notify();
                                }))
                        } else {
                            super::section_add_button("add-checksum-input-algorithm").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.input_algorithm_present = true;
                                    cx.notify();
                                }),
                            )
                        }),
                )
                .when(self.input_algorithm_present, |form| {
                    form.child(self.input_algorithm.clone())
                })
            })
            .when(self.kind == ErrorKind::Crc, |form| {
                form.child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(field("Width", "Required", &self.crc_width, cx))
                        .child(select_field("Reflect data", &self.reflect_data))
                        .child(select_field("Reflect remainder", &self.reflect_remainder)),
                )
                .child(select_field("Bit direction", &self.direction))
                .child(field("Polynomial", "Required", &self.polynomial, cx))
                .child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(field(
                            "Initial remainder",
                            "Optional",
                            &self.init_remainder,
                            cx,
                        ))
                        .child(field("Final XOR", "Optional", &self.final_xor, cx)),
                )
            })
            .when(self.kind == ErrorKind::Parity, |form| {
                form.child(select_field("Parity", &self.parity))
            })
    }
}

#[derive(Default)]
struct ErrorValues {
    kind: ErrorKind,
    bits_from_reference: String,
    reference: ReferenceChoice,
    parameter_ref: String,
    checksum_name: ChecksumChoice,
    hash_size_in_bits: String,
    input_algorithm_present: bool,
    crc_width: String,
    reflect_data: bool,
    reflect_remainder: bool,
    direction: DirectionChoice,
    polynomial: String,
    init_remainder: String,
    final_xor: String,
    parity: ParityChoice,
}

fn values(
    value: Option<&xtce::ErrorDetectCorrectType>,
) -> Vec<(ErrorValues, Option<&xtce::InputAlgorithmType>)> {
    value
        .into_iter()
        .flat_map(|value| &value.content)
        .map(|content| {
            let algorithm = match content {
                xtce::ErrorDetectCorrectTypeContent::Checksum(value) => {
                    value.input_algorithm.as_ref()
                }
                _ => None,
            };
            (ErrorValues::from_content(content), algorithm)
        })
        .collect()
}

impl ErrorValues {
    fn from_content(content: &xtce::ErrorDetectCorrectTypeContent) -> Self {
        match content {
            xtce::ErrorDetectCorrectTypeContent::Checksum(value) => Self {
                kind: ErrorKind::Checksum,
                bits_from_reference: value.bits_from_reference.to_string(),
                reference: reference_choice(&value.reference),
                parameter_ref: value.parameter_ref.clone().unwrap_or_default(),
                checksum_name: checksum_choice(&value.name),
                hash_size_in_bits: value
                    .hash_size_in_bits
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                input_algorithm_present: value.input_algorithm.is_some(),
                ..Self::default()
            },
            xtce::ErrorDetectCorrectTypeContent::Crc(value) => Self {
                kind: ErrorKind::Crc,
                bits_from_reference: value.bits_from_reference.to_string(),
                reference: reference_choice(&value.reference),
                parameter_ref: value.parameter_ref.clone().unwrap_or_default(),
                crc_width: value.width.to_string(),
                reflect_data: value.reflect_data,
                reflect_remainder: value.reflect_remainder,
                direction: match value.direction {
                    xtce::BitOrderType::MostSignificantBitFirst => {
                        DirectionChoice::MostSignificantBitFirst
                    }
                    xtce::BitOrderType::LeastSignificantBitFirst => {
                        DirectionChoice::LeastSignificantBitFirst
                    }
                },
                polynomial: value.polynomial.clone(),
                init_remainder: value.init_remainder.clone().unwrap_or_default(),
                final_xor: value.final_xor.clone().unwrap_or_default(),
                ..Self::default()
            },
            xtce::ErrorDetectCorrectTypeContent::Xor(value) => Self {
                kind: ErrorKind::Xor,
                bits_from_reference: value.bits_from_reference.to_string(),
                reference: reference_choice(&value.reference),
                parameter_ref: value.parameter_ref.clone().unwrap_or_default(),
                ..Self::default()
            },
            xtce::ErrorDetectCorrectTypeContent::Parity(value) => Self {
                kind: ErrorKind::Parity,
                bits_from_reference: value.bits_from_reference.to_string(),
                reference: reference_choice(&value.reference),
                parameter_ref: value.parameter_ref.clone().unwrap_or_default(),
                parity: match value.type_ {
                    xtce::ParityFormType::Even => ParityChoice::Even,
                    xtce::ParityFormType::Odd => ParityChoice::Odd,
                },
                ..Self::default()
            },
        }
    }
}

fn new_row(
    values: ErrorValues,
    algorithm: Option<&xtce::InputAlgorithmType>,
    window: &mut Window,
    cx: &mut impl AppContext,
) -> Entity<ErrorRow> {
    let kind_select = select(ErrorKind::VARIANTS, values.kind, window, cx);
    let input_algorithm = InputAlgorithmForm::new(algorithm, window, cx);
    cx.new(move |cx| {
        let subscription = cx.subscribe_in(
            &kind_select,
            window,
            |this: &mut ErrorRow, _, event: &SelectEvent<Vec<ErrorKind>>, _, cx| {
                if let SelectEvent::Confirm(Some(kind)) = event {
                    this.kind = *kind;
                    cx.notify();
                }
            },
        );
        subscription.detach();
        ErrorRow {
            kind: values.kind,
            kind_select,
            bits_from_reference: input(&values.bits_from_reference, window, cx),
            reference: select(ReferenceChoice::VARIANTS, values.reference, window, cx),
            parameter_ref: input(&values.parameter_ref, window, cx),
            checksum_name: select(ChecksumChoice::VARIANTS, values.checksum_name, window, cx),
            hash_size_in_bits: input(&values.hash_size_in_bits, window, cx),
            input_algorithm_present: values.input_algorithm_present,
            input_algorithm,
            crc_width: input(&values.crc_width, window, cx),
            reflect_data: select(
                BooleanChoice::VARIANTS,
                boolean_choice(values.reflect_data),
                window,
                cx,
            ),
            reflect_remainder: select(
                BooleanChoice::VARIANTS,
                boolean_choice(values.reflect_remainder),
                window,
                cx,
            ),
            direction: select(DirectionChoice::VARIANTS, values.direction, window, cx),
            polynomial: input(&values.polynomial, window, cx),
            init_remainder: input(&values.init_remainder, window, cx),
            final_xor: input(&values.final_xor, window, cx),
            parity: select(ParityChoice::VARIANTS, values.parity, window, cx),
        }
    })
}

fn input(value: &str, window: &mut Window, cx: &mut impl AppContext) -> Entity<InputState> {
    cx.new(|cx| InputState::new(window, cx).default_value(value.to_owned()))
}

fn value(input: &Entity<InputState>, cx: &App) -> String {
    input.read(cx).value().to_string()
}

fn integer(input: &Entity<InputState>, cx: &App) -> i64 {
    value(input, cx).trim().parse().unwrap_or_default()
}

fn optional_integer(input: &Entity<InputState>, cx: &App) -> Option<i64> {
    value(input, cx).trim().parse().ok()
}

fn boolean(select: &Entity<SelectState<Vec<BooleanChoice>>>, cx: &App) -> bool {
    selected(select, BooleanChoice::False, cx) == BooleanChoice::True
}

fn boolean_choice(value: bool) -> BooleanChoice {
    if value {
        BooleanChoice::True
    } else {
        BooleanChoice::False
    }
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
    let index = choices
        .iter()
        .position(|choice| *choice == selected)
        .unwrap_or_default();
    cx.new(|cx| {
        SelectState::new(
            choices.to_vec(),
            Some(IndexPath::default().row(index)),
            window,
            cx,
        )
    })
}

fn selected<T>(select: &Entity<SelectState<Vec<T>>>, fallback: T, cx: &App) -> T
where
    T: Clone + Copy + PartialEq + gpui_component::select::SelectItem<Value = T> + 'static,
{
    select
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or(fallback)
}

fn select_field<T>(label: &'static str, select: &Entity<SelectState<Vec<T>>>) -> gpui::Div
where
    T: Clone + PartialEq + gpui_component::select::SelectItem + 'static,
{
    super::select_field(label, "", select)
}

fn reference_choice(value: &xtce::ReferencePointType) -> ReferenceChoice {
    match value {
        xtce::ReferencePointType::Start => ReferenceChoice::Start,
        xtce::ReferencePointType::End => ReferenceChoice::End,
    }
}

fn checksum_choice(value: &xtce::ChecksumTypeNameType) -> ChecksumChoice {
    match value {
        xtce::ChecksumTypeNameType::UnixSum => ChecksumChoice::UnixSum,
        xtce::ChecksumTypeNameType::Sum8 => ChecksumChoice::Sum8,
        xtce::ChecksumTypeNameType::Sum16 => ChecksumChoice::Sum16,
        xtce::ChecksumTypeNameType::Sum24 => ChecksumChoice::Sum24,
        xtce::ChecksumTypeNameType::Sum32 => ChecksumChoice::Sum32,
        xtce::ChecksumTypeNameType::Fletcher4 => ChecksumChoice::Fletcher4,
        xtce::ChecksumTypeNameType::Fletcher8 => ChecksumChoice::Fletcher8,
        xtce::ChecksumTypeNameType::Fletcher16 => ChecksumChoice::Fletcher16,
        xtce::ChecksumTypeNameType::Fletcher32 => ChecksumChoice::Fletcher32,
        xtce::ChecksumTypeNameType::Adler32 => ChecksumChoice::Adler32,
        xtce::ChecksumTypeNameType::Luhn => ChecksumChoice::Luhn,
        xtce::ChecksumTypeNameType::Verhoeff => ChecksumChoice::Verhoeff,
        xtce::ChecksumTypeNameType::Damm => ChecksumChoice::Damm,
        xtce::ChecksumTypeNameType::Custom => ChecksumChoice::Custom,
    }
}

fn checksum_type(value: ChecksumChoice) -> xtce::ChecksumTypeNameType {
    match value {
        ChecksumChoice::UnixSum => xtce::ChecksumTypeNameType::UnixSum,
        ChecksumChoice::Sum8 => xtce::ChecksumTypeNameType::Sum8,
        ChecksumChoice::Sum16 => xtce::ChecksumTypeNameType::Sum16,
        ChecksumChoice::Sum24 => xtce::ChecksumTypeNameType::Sum24,
        ChecksumChoice::Sum32 => xtce::ChecksumTypeNameType::Sum32,
        ChecksumChoice::Fletcher4 => xtce::ChecksumTypeNameType::Fletcher4,
        ChecksumChoice::Fletcher8 => xtce::ChecksumTypeNameType::Fletcher8,
        ChecksumChoice::Fletcher16 => xtce::ChecksumTypeNameType::Fletcher16,
        ChecksumChoice::Fletcher32 => xtce::ChecksumTypeNameType::Fletcher32,
        ChecksumChoice::Adler32 => xtce::ChecksumTypeNameType::Adler32,
        ChecksumChoice::Luhn => xtce::ChecksumTypeNameType::Luhn,
        ChecksumChoice::Verhoeff => xtce::ChecksumTypeNameType::Verhoeff,
        ChecksumChoice::Damm => xtce::ChecksumTypeNameType::Damm,
        ChecksumChoice::Custom => xtce::ChecksumTypeNameType::Custom,
    }
}

#[cfg(test)]
mod tests {
    use super::{ChecksumChoice, ErrorKind, ErrorValues, ParityChoice};

    #[test]
    fn checksum_fields_are_loaded() {
        let content = xtce::ErrorDetectCorrectTypeContent::Checksum(xtce::ChecksumType {
            bits_from_reference: 12,
            reference: xtce::ReferencePointType::End,
            name: xtce::ChecksumTypeNameType::Fletcher16,
            hash_size_in_bits: Some(16),
            parameter_ref: Some("checksum".to_owned()),
            input_algorithm: None,
        });

        let values = ErrorValues::from_content(&content);

        assert_eq!(values.kind, ErrorKind::Checksum);
        assert_eq!(values.checksum_name, ChecksumChoice::Fletcher16);
        assert_eq!(values.hash_size_in_bits, "16");
        assert_eq!(values.parameter_ref, "checksum");
    }

    #[test]
    fn parity_fields_are_loaded() {
        let content = xtce::ErrorDetectCorrectTypeContent::Parity(xtce::ParityType {
            type_: xtce::ParityFormType::Odd,
            bits_from_reference: 0,
            reference: xtce::ReferencePointType::Start,
            parameter_ref: None,
        });

        let values = ErrorValues::from_content(&content);

        assert_eq!(values.kind, ErrorKind::Parity);
        assert_eq!(values.parity, ParityChoice::Odd);
    }
}
